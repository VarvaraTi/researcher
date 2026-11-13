use learning_rules::{
    persistence::{dispatch, render},
    rules::Record,
    Request,
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn changeset_summary(output: &str) -> Value {
    output
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .find(|record| record["kind"] == "changeset-summary")
        .unwrap()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn copy(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}
struct Fixture {
    _dir: tempfile::TempDir,
    ws: PathBuf,
    bundle: PathBuf,
    okf: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("workspace");
        copy(&root().join("tests/fixtures/workspace"), &ws);
        Self {
            bundle: ws.join("courses/synthetic"),
            ws,
            okf: root().join(".local/bin/okf"),
            _dir: dir,
        }
    }
    fn context(&self) -> Value {
        json!({"okf":self.okf,"bundle":self.bundle,"scope_bundles":["learning.learner"]})
    }
    fn cli(&self, args: &[&str]) -> String {
        let out = Command::new(&self.okf)
            .current_dir(&self.ws)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }
    fn request(&self) -> Value {
        let mut r: Value = serde_json::from_str(
            &fs::read_to_string(root().join("tests/fixtures/requests/save-attempt.json")).unwrap(),
        )
        .unwrap();
        r["data"]["context"] = self.context();
        r["data"].clone()
    }
    fn save(&self, data: Value) -> learning_rules::Result<Value> {
        dispatch(
            "save-attempt",
            Request {
                schema_version: 1,
                data,
            },
        )
    }
    fn publish_request(&self, ops: Value) -> Value {
        let changeset = json!({"version":1,"operations":ops});
        let output = self.cli(&[
            "changeset",
            "plan",
            "courses/synthetic",
            "--from",
            &changeset.to_string(),
            "--json",
        ]);
        let summary: Value = changeset_summary(&output);
        json!({"context":self.context(),"changeset":changeset,"expected_digest":summary["base_digest"]})
    }
    fn publish(&self, ops: Value) -> learning_rules::Result<Value> {
        dispatch(
            "publish",
            Request {
                schema_version: 1,
                data: self.publish_request(ops),
            },
        )
    }
    fn inspect(&self) -> Value {
        dispatch(
            "inspect-state",
            Request {
                schema_version: 1,
                data: self.context(),
            },
        )
        .unwrap()["result"]
            .clone()
    }
}
fn create(mut record: Value) -> Value {
    if record["metadata"]["type"] == "Assessment"
        && record["metadata"]["judgment"]["supersedes"].is_null()
    {
        record["metadata"]["judgment"]
            .as_object_mut()
            .unwrap()
            .remove("supersedes");
    }
    let r: Record = serde_json::from_value(record).unwrap();
    json!({"op":"create","path":format!("{}.md",r.id.trim_start_matches('/')),"content":render(&r).unwrap()})
}

#[test]
fn answer_retry_conflict_and_restart_without_assessment() {
    let f = Fixture::new();
    let request = f.request();
    assert_eq!(
        f.save(request.clone()).unwrap()["result"]["save"],
        "created"
    );
    let original = fs::read(f.bundle.join("attempts/submission-001.md")).unwrap();
    assert_eq!(
        f.save(request.clone()).unwrap()["result"]["save"],
        "identical_retry"
    );
    let mut conflicting = request;
    conflicting["record"]["body"] = json!("different answer");
    assert_eq!(f.save(conflicting).unwrap_err().code, "submission_conflict");
    assert_eq!(
        original,
        fs::read(f.bundle.join("attempts/submission-001.md")).unwrap()
    );
    assert_eq!(
        f.inspect()["pending_assessments"],
        json!(["/attempts/submission-001"])
    );
}
#[test]
fn structural_and_reference_gates_prevent_activation() {
    let f = Fixture::new();
    let ops = json!([{"op":"edit","concept":"sessions/one","patch":[{"op":"replace","path":"/session_status","value":"made_up"}]}]);
    assert_eq!(f.publish(ops).unwrap_err().code, "structural_gate");
    assert!(fs::read_to_string(f.bundle.join("sessions/one.md"))
        .unwrap()
        .contains("session_status: active"));
    let ops = json!([{"op":"edit","concept":"objectives/distinct","patch":[{"op":"replace","path":"/topic","value":"/sessions/one"}]}]);
    assert_eq!(f.publish(ops).unwrap_err().code, "structural_gate");
}
#[test]
fn stale_guard_and_failed_patch_preserve_newer_state() {
    let f = Fixture::new();
    let ops = json!([{"op":"edit","concept":"sessions/one","patch":[{"op":"test","path":"/session_status","value":"active"},{"op":"replace","path":"/session_status","value":"paused"}]}]);
    let req = f.publish_request(ops);
    f.cli(&[
        "edit",
        "sessions/one",
        "courses/synthetic",
        "--set",
        "title=intervening",
    ]);
    assert_eq!(
        dispatch(
            "publish",
            Request {
                schema_version: 1,
                data: req
            }
        )
        .unwrap_err()
        .code,
        "stale_guard"
    );
    assert!(fs::read_to_string(f.bundle.join("sessions/one.md"))
        .unwrap()
        .contains("title: intervening"));
}
#[test]
fn original_answers_judgments_and_exercise_versions_are_retained() {
    let f = Fixture::new();
    f.save(f.request()).unwrap();
    assert_eq!(
        f.publish(json!([{"op":"edit","concept":"attempts/submission-001","body":"replacement"}]))
            .unwrap_err()
            .code,
        "immutable_record"
    );
    assert_eq!(
        f.publish(json!([{"op":"edit","concept":"exercises/distinct-v1","body":"changed prompt"}]))
            .unwrap_err()
            .code,
        "immutable_record"
    );
    let r = learning_rules::persistence::parse_record(
        "/exercises/distinct-v2",
        &fs::read_to_string(f.bundle.join("exercises/distinct-v1.md")).unwrap(),
    )
    .unwrap();
    let mut value = serde_json::to_value(r).unwrap();
    value["metadata"]["version"] = json!("2");
    f.publish(json!([create(value)])).unwrap();
    assert!(f.bundle.join("exercises/distinct-v1.md").exists());
    assert!(f.bundle.join("exercises/distinct-v2.md").exists());
}
#[test]
fn correction_and_readiness_resume_preserve_answer() {
    let f = Fixture::new();
    f.save(f.request()).unwrap();
    let judgment = |id: &str, outcome: &str, parent: Value| json!({"id":id,"metadata":{"type":"Assessment","schema_version":1,"title":id,"attempt":"/attempts/submission-001","assessor":"synthetic-agent","judgment":{"id":id,"attempt":"/attempts/submission-001","assessment_status":"resolved","outcome":outcome,"criteria":{"distinctness":true},"supersedes":parent}},"body":"Synthetic feedback.\n"});
    f.publish(json!([create(judgment(
        "/assessments/first",
        "incorrect",
        Value::Null
    ))]))
    .unwrap();
    f.publish(json!([create(judgment(
        "/assessments/corrected",
        "correct",
        json!("/assessments/first")
    ))]))
    .unwrap();
    let readiness = json!({"id":"/readiness/one","metadata":{"type":"ReadinessChoice","schema_version":1,"title":"Continue with uncertainty","session":"/sessions/one","objective":"/objectives/distinct","timestamp":"2026-10-25T12:02:00Z","uncertainty":"Correct answer; transfer not checked","choice":"continue"},"body":"Synthetic learner chooses to continue.\n"});
    f.publish(json!([create(readiness),{"op":"edit","concept":"cheat-sheets/one-v1","patch":[{"op":"replace","path":"/revealed_sections","value":["distinct"]}]},{"op":"edit","concept":"lessons/two-v1","patch":[{"op":"replace","path":"/readiness","value":"ready"}],"body":"Begin with a smaller worked example because the saved readiness choice retains uncertainty. Then offer union discussion. Source: section 2 of /sources/sets-v1.\n"},{"op":"edit","concept":"sessions/one","patch":[{"op":"replace","path":"/current_lesson","value":"/lessons/two-v1"},{"op":"replace","path":"/current_item","value":"discussion"}]}])).unwrap();
    let state = f.inspect();
    assert_eq!(state["pending_assessments"], json!([]));
    assert_eq!(
        state["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["metadata"]["type"] == "Assessment")
            .count(),
        2
    );
    assert!(
        fs::read_to_string(f.bundle.join("attempts/submission-001.md"))
            .unwrap()
            .ends_with("Two, because repeated a counts once.\n")
    );
}
#[test]
fn pending_okf_publication_requires_recovery_before_reconciliation() {
    let f = Fixture::new();
    f.save(f.request()).unwrap();
    fs::create_dir(f.bundle.join(".okf-transaction")).unwrap();
    assert_eq!(f.save(f.request()).unwrap_err().code, "recovery_required");
    f.cli(&["changeset", "recover", "courses/synthetic", "--json"]);
    assert_eq!(
        f.save(f.request()).unwrap()["result"]["save"],
        "identical_retry"
    );
}

#[test]
fn real_journal_rolls_back_partial_publication_and_refuses_intervening_edits() {
    let f = Fixture::new();
    f.save(f.request()).unwrap();
    let path = "sessions/one.md";
    let before = fs::read(f.bundle.join(path)).unwrap();
    let after = b"interrupted publication".to_vec();
    let journal = f.bundle.join(".okf-transaction");
    fs::create_dir(&journal).unwrap();
    fs::write(
        journal.join("manifest.json"),
        serde_json::to_vec(&json!([{"path":path,"before":before,"after":after}])).unwrap(),
    )
    .unwrap();
    fs::write(f.bundle.join(path), b"intervening edit").unwrap();
    let out = Command::new(&f.okf)
        .current_dir(&f.ws)
        .args(["changeset", "recover", "courses/synthetic", "--json"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(fs::read(f.bundle.join(path)).unwrap(), b"intervening edit");
    // Restore the simulated published image, then let the real CLI recover it.
    fs::write(f.bundle.join(path), after).unwrap();
    f.cli(&["changeset", "recover", "courses/synthetic", "--json"]);
    assert_eq!(fs::read(f.bundle.join(path)).unwrap(), before);
    assert_eq!(
        f.inspect()["pending_assessments"],
        json!(["/attempts/submission-001"])
    );
}

#[test]
fn shared_observation_retry_is_independent_of_course_answer() {
    use sha2::{Digest, Sha256};
    let f = Fixture::new();
    f.save(f.request()).unwrap();
    let origin = "learning.synthetic:/attempts/submission-001:distinctness";
    let id = format!("/observations/{:x}", Sha256::digest(origin.as_bytes()));
    let context =
        json!({"okf":f.okf,"bundle":f.ws.join("learner"),"scope_bundles":["learning.synthetic"]});
    let data = json!({"context":context,"record":{"id":id,"metadata":{"type":"LearnerObservation","schema_version":1,"title":"Distinctness evidence","origin_id":origin,"observation":"Explained duplicate handling","scope":"learning.synthetic","observed_at":"2026-10-25T12:03:00Z","author":"synthetic-agent","evidence":["../../courses/synthetic/attempts/submission-001.md"],"sources":[{"id":"original-answer","resource":"../../courses/synthetic/attempts/submission-001.md","bundle_ref":{"id":"learning.synthetic","path":"attempts/submission-001.md"}}]},"body":"Performance observation; no trait claim.\n"}});
    let learner = f.ws.join("learner");
    fs::create_dir(learner.join(".okf-transaction")).unwrap();
    assert_eq!(
        dispatch(
            "save-observation",
            Request {
                schema_version: 1,
                data: data.clone()
            }
        )
        .unwrap_err()
        .code,
        "recovery_required"
    );
    assert!(f.bundle.join("attempts/submission-001.md").exists());
    f.cli(&["changeset", "recover", "learner", "--json"]);
    assert_eq!(
        dispatch(
            "save-observation",
            Request {
                schema_version: 1,
                data: data.clone()
            }
        )
        .unwrap()["result"]["save"],
        "created"
    );
    assert_eq!(
        dispatch(
            "save-observation",
            Request {
                schema_version: 1,
                data
            }
        )
        .unwrap()["result"]["save"],
        "identical_retry"
    );
}

#[test]
fn notes_and_source_snapshots_survive_content_updates() {
    let f = Fixture::new();
    let note = json!({"id":"/notes/personal","metadata":{"type":"Note","schema_version":1,"title":"My words","author":"synthetic-learner","created_at":"2026-10-25T12:00:00Z"},"body":"  My exact words.\n\n"});
    f.publish(json!([create(note),{"op":"edit","concept":"cheat-sheets/one-v1","patch":[{"op":"replace","path":"/personal_additions","value":["/notes/personal"]}]}])).unwrap();
    let original = fs::read(f.bundle.join("notes/personal.md")).unwrap();
    f.publish(json!([{"op":"edit","concept":"cheat-sheets/one-v1","patch":[{"op":"replace","path":"/revealed_sections","value":["distinct"]}]}])).unwrap();
    assert_eq!(
        fs::read(f.bundle.join("notes/personal.md")).unwrap(),
        original
    );
    let error=f.publish(json!([{"op":"put-artifact","path":"references/source.txt","content":"changed","replace":true}])).unwrap_err();
    assert_eq!(error.code, "historical_artifact");
}

#[test]
fn two_courses_share_corrected_profile_but_keep_course_overrides() {
    let f = Fixture::new();
    let second = f.ws.join("courses/second");
    copy(&f.bundle, &second);
    let catalog = f.ws.join("okf-catalog.yaml");
    let mut text = fs::read_to_string(&catalog).unwrap();
    text.push_str("  learning.second:\n    location: {type: directory, path: courses/second}\n");
    fs::write(catalog, text).unwrap();
    let source = fs::read_to_string(f.ws.join("learner/preferences/language-v1.md")).unwrap();
    let mut corrected =
        learning_rules::persistence::parse_record("/preferences/language-v2", &source).unwrap();
    corrected
        .metadata
        .insert("value".into(), json!({"data":"Spanish"}));
    corrected
        .metadata
        .insert("supersedes".into(), json!("/preferences/language-v1"));
    let changes = json!({"version":1,"operations":[create(serde_json::to_value(corrected).unwrap()),{"op":"edit","concept":"preferences/language-v1","patch":[{"op":"test","path":"/preference_status","value":"confirmed"},{"op":"replace","path":"/preference_status","value":"retired"}]},{"op":"edit","concept":"profile","patch":[{"op":"replace","path":"/preferences","value":["/preferences/language-v2"]}]}]});
    let plan = f.cli(&[
        "changeset",
        "plan",
        "learner",
        "--from",
        &changes.to_string(),
        "--json",
    ]);
    let summary: Value = changeset_summary(&plan);
    let context = json!({"okf":f.okf,"bundle":f.ws.join("learner"),"scope_bundles":[]});
    dispatch("publish",Request{schema_version:1,data:json!({"context":context,"changeset":changes,"expected_digest":summary["base_digest"]})}).unwrap();
    let profile = dispatch(
        "inspect-state",
        Request {
            schema_version: 1,
            data: context,
        },
    )
    .unwrap();
    let preference = profile["result"]["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "/preferences/language-v2")
        .unwrap();
    let global = json!({"language":{"value":preference["metadata"]["value"]["data"],"source":"learning.learner:/preferences/language-v2","version":"2"}});
    for (overrides, expected) in [
        (
            json!({"language":{"value":"English","source":"course","version":"1"}}),
            "English",
        ),
        (json!({}), "Spanish"),
    ] {
        let result = learning_rules::calculate(
            "resolve-settings",
            Request {
                schema_version: 1,
                data: json!({"defaults":{},"global":global,"course":overrides,"session":{}}),
            },
        )
        .unwrap();
        assert_eq!(result["result"]["language"]["value"], expected);
    }
    assert_eq!(
        fs::read(f.bundle.join("objectives/distinct.md")).unwrap(),
        fs::read(second.join("objectives/distinct.md")).unwrap()
    );
}

fn study_value(value: &mut Value, slug: &str) {
    match value {
        Value::String(s) if s.starts_with('/') => {
            *s = if s == "/course" {
                format!("/studies/{slug}/study")
            } else {
                format!("/studies/{slug}{s}")
            };
        }
        Value::Array(xs) => xs.iter_mut().for_each(|v| study_value(v, slug)),
        Value::Object(xs) => xs.values_mut().for_each(|v| study_value(v, slug)),
        _ => (),
    }
}
fn article_fixture() -> Fixture {
    let f = Fixture::new();
    let original = f.inspect()["records"].as_array().unwrap().clone();
    for r in &original {
        fs::remove_file(f.bundle.join(format!(
            "{}.md",
            r["id"].as_str().unwrap().trim_start_matches('/')
        )))
        .unwrap();
    }
    fs::copy(
        root().join("schemas/article-ontology.yaml"),
        f.bundle.join("ontology.yaml"),
    )
    .unwrap();
    for slug in ["alpha", "beta"] {
        let dest = f.bundle.join(format!("studies/{slug}/references"));
        fs::create_dir_all(&dest).unwrap();
        fs::copy(
            f.bundle.join("references/source.txt"),
            dest.join("source.txt"),
        )
        .unwrap();
        for old in &original {
            let mut r = old.clone();
            study_value(&mut r, slug);
            let m = r["metadata"].as_object_mut().unwrap();
            m.insert("description".into(), json!("Synthetic article record"));
            if m["type"] == "Course" {
                m.insert("type".into(), json!("Study"));
                m.insert("kind".into(), json!("article"));
                m.insert("domains".into(), json!(["testing"]));
            } else {
                m.remove("course");
                m.insert("study".into(), json!(format!("/studies/{slug}/study")));
                if m["type"] == "CourseSettings" {
                    m.insert("type".into(), json!("StudySettings"));
                    m.insert("learner".into(), json!("../../../../learner/profile.md"));
                }
            }
            if r["body"].as_str().unwrap().trim().is_empty() {
                r["body"] = json!("Synthetic teaching content.\n");
            }
            let r: Record = serde_json::from_value(r).unwrap();
            let path = f
                .bundle
                .join(format!("{}.md", r.id.trim_start_matches('/')));
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, render(&r).unwrap()).unwrap();
        }
    }
    f
}
fn article_context(f: &Fixture, slug: &str) -> Value {
    let mut c = f.context();
    c["study"] = json!(format!("/studies/{slug}/study"));
    c
}
fn article_inspect(f: &Fixture, slug: &str) -> Value {
    dispatch(
        "inspect-state",
        Request {
            schema_version: 1,
            data: article_context(f, slug),
        },
    )
    .unwrap()["result"]
        .clone()
}
fn article_answer(f: &Fixture, slug: &str, sequence: u64) -> Value {
    let mut req = f.request();
    study_value(&mut req["record"], slug);
    req["context"] = article_context(f, slug);
    req["record"]["metadata"]["study"] = json!(format!("/studies/{slug}/study"));
    req["record"]["metadata"]["description"] = json!("Synthetic original answer");
    req["record"]["metadata"]["event"]["sequence"] = json!(sequence);
    let submission = format!("{slug}-001");
    let id = format!("/studies/{slug}/attempts/{submission}");
    req["record"]["id"] = json!(id);
    req["record"]["metadata"]["event"]["id"] = json!(id);
    req["record"]["metadata"]["event"]["submission_id"] = json!(submission);
    req
}
#[test]
fn article_selection_isolates_history_and_blocks_cross_study_answers() {
    let f = article_fixture();
    assert_eq!(
        dispatch(
            "inspect-state",
            Request {
                schema_version: 1,
                data: f.context()
            }
        )
        .unwrap_err()
        .code,
        "study_required"
    );
    let a = article_answer(&f, "alpha", 1);
    f.save(a.clone()).unwrap();
    f.save(article_answer(&f, "beta", 2)).unwrap();
    let alpha = article_inspect(&f, "alpha");
    let beta = article_inspect(&f, "beta");
    assert_eq!(alpha["history"]["attempts"].as_array().unwrap().len(), 1);
    assert_eq!(beta["history"]["attempts"].as_array().unwrap().len(), 1);
    assert_eq!(alpha["next_sequence"], 3);
    assert!(alpha["records"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["id"].as_str().unwrap().starts_with("/studies/alpha/")));
    assert_eq!(f.save(a).unwrap()["result"]["save"], "identical_retry");
    let mut wrong = article_answer(&f, "alpha", 3);
    wrong["record"]["id"] = json!("/studies/alpha/attempts/alpha-002");
    wrong["record"]["metadata"]["event"]["id"] = wrong["record"]["id"].clone();
    wrong["record"]["metadata"]["event"]["submission_id"] = json!("alpha-002");
    wrong["record"]["metadata"]["objective"] = json!("/studies/beta/objectives/distinct");
    wrong["record"]["metadata"]["event"]["objective"] = json!("/studies/beta/objectives/distinct");
    assert_eq!(f.save(wrong).unwrap_err().code, "study_boundary");
    assert_eq!(article_inspect(&f, "beta"), beta);
}
#[test]
fn article_publication_protects_other_studies_and_nested_source_artifacts() {
    let f = article_fixture();
    for (ops, expected) in [
        (
            json!([{"op":"edit","concept":"studies/beta/sessions/one","patch":[{"op":"replace","path":"/current_item","value":"changed"}]}]),
            "study_boundary",
        ),
        (
            json!([{"op":"put-artifact","path":"studies/alpha/references/source.txt","content":"changed","replace":true}]),
            "historical_artifact",
        ),
        (
            json!([{"op":"edit","concept":"studies/alpha/sessions/one","patch":[{"op":"replace","path":"/session_status","value":"completed"}]}]),
            "session_summary",
        ),
    ] {
        let mut req = f.publish_request(ops);
        req["context"] = article_context(&f, "alpha");
        assert_eq!(
            dispatch(
                "publish",
                Request {
                    schema_version: 1,
                    data: req
                }
            )
            .unwrap_err()
            .code,
            expected
        );
    }
}

#[test]
fn article_coverage_is_separate_and_topic_relations_do_not_merge_histories() {
    let f = article_fixture();
    let ops = json!([
      {"op":"edit","concept":"studies/alpha/topics/sets","patch":[{"op":"replace","path":"/related_topics","value":["/studies/beta/topics/sets"]}]},
      {"op":"edit","concept":"studies/alpha/sessions/one","patch":[
        {"op":"replace","path":"/session_status","value":"completed"},
        {"op":"add","path":"/covered_objectives","value":["/studies/alpha/objectives/distinct"]},
        {"op":"add","path":"/understanding_check","value":"not_performed"}
      ],"body":"Discussed the concept; no independent answer was requested.\n"}
    ]);
    let mut req = f.publish_request(ops);
    req["context"] = article_context(&f, "alpha");
    dispatch(
        "publish",
        Request {
            schema_version: 1,
            data: req,
        },
    )
    .unwrap();
    let result = article_inspect(&f, "alpha");
    assert_eq!(result["history"]["attempts"], json!([]));
    assert_eq!(
        result["coverage"][0]["understanding_check"],
        "not_performed"
    );
    assert_eq!(
        article_inspect(&f, "beta")["coverage"][0]["status"],
        "active"
    );
    let mut req =
        f.publish_request(json!([{"op":"edit","concept":"studies/alpha/topics/sets","body":""}]));
    req["context"] = article_context(&f, "alpha");
    assert_eq!(
        dispatch(
            "publish",
            Request {
                schema_version: 1,
                data: req
            }
        )
        .unwrap_err()
        .code,
        "study_content"
    );
}

fn model_audit(f: &Fixture) -> Value {
    dispatch("audit-learner-model",Request{schema_version:1,data:json!({"context":f.context(),"learner_context":{"okf":f.okf,"bundle":f.ws.join("learner"),"scope_bundles":["learning.synthetic"]}})}).unwrap()["result"].clone()
}
#[test]
fn learner_audit_detects_missing_reviews_and_changed_sources_without_mutating_answers() {
    let f = Fixture::new();
    f.cli(&[
        "ontology",
        "apply",
        "learner",
        "--from",
        &format!(
            "@{}",
            root().join("schemas/learner-ontology.yaml").display()
        ),
    ]);
    f.save(f.request()).unwrap();
    let answer = fs::read(f.bundle.join("attempts/submission-001.md")).unwrap();
    let audit = model_audit(&f);
    assert_eq!(audit["review_complete"], false);
    let candidate = audit["pending"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["source_id"] == "learning.synthetic:/course")
        .unwrap();
    let review = json!({"id":candidate["review_id"],"metadata":{"type":"LearnerModelReview","schema_version":1,"title":"Reviewed synthetic course","source_id":candidate["source_id"],"source_digest":candidate["source_digest"],"decision":"not_needed","reason":"Synthetic course creates no new learner assertion.","reviewed_at":"2026-11-01T12:00:00Z","assertions":[]},"body":"No inference about the learner.\n"});
    let changes = json!({"version":1,"operations":[create(review.clone())]});
    let out = f.cli(&[
        "changeset",
        "plan",
        "learner",
        "--from",
        &changes.to_string(),
        "--json",
    ]);
    dispatch("publish",Request{schema_version:1,data:json!({"context":{"okf":f.okf,"bundle":f.ws.join("learner"),"scope_bundles":["learning.synthetic"]},"changeset":changes,"expected_digest":changeset_summary(&out)["base_digest"]})}).unwrap();
    let after = model_audit(&f);
    assert_eq!(
        after["pending"].as_array().unwrap().len() + 1,
        audit["pending"].as_array().unwrap().len()
    );
    assert_eq!(model_audit(&f), after);
    f.cli(&[
        "edit",
        "course",
        "courses/synthetic",
        "--set",
        "goal=Updated synthetic goal",
    ]);
    let changed = model_audit(&f);
    assert_eq!(
        changed["pending"].as_array().unwrap().len(),
        audit["pending"].as_array().unwrap().len()
    );
    let next = changed["pending"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["source_id"] == "learning.synthetic:/course")
        .unwrap();
    assert_ne!(next["review_id"], candidate["review_id"]);
    assert_eq!(
        answer,
        fs::read(f.bundle.join("attempts/submission-001.md")).unwrap()
    );
    let mut bad: Record = serde_json::from_value(review).unwrap();
    bad.metadata.insert("decision".into(), json!("recorded"));
    bad.metadata
        .insert("assertions".into(), json!(["/observations/missing"]));
    assert_eq!(
        learning_rules::learner_model::check(&learning_rules::rules::State {
            complete: true,
            records: vec![bad]
        })
        .unwrap_err()
        .code,
        "model_review"
    );
}
