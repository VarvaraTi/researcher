use learning_rules::{calculate, Request};
use serde_json::{json, Value};

fn run(command: &str, data: Value) -> Value {
    calculate(
        command,
        Request {
            schema_version: 1,
            data,
        },
    )
    .unwrap()["result"]
        .clone()
}
fn error(command: &str, data: Value) -> String {
    calculate(
        command,
        Request {
            schema_version: 1,
            data,
        },
    )
    .unwrap_err()
    .code
}
fn attempt(n: u64, day: u32, session: &str) -> Value {
    json!({"id":format!("a{n}"),"submission_id":format!("sub{n}"),"exercise":"e-v1","session":session,"sequence":n,"timestamp":format!("2026-10-{day:02}T12:00:00Z"),"assistance":"none","objective":"o","evidence_kind":"explanation","is_review":n>1,"independent_eligible":true})
}
fn judgment(n: u64, outcome: &str) -> Value {
    json!({"id":format!("j{n}"),"attempt":format!("a{n}"),"assessment_status":"resolved","outcome":outcome,"criteria":{"explain":true},"supersedes":null})
}
fn history() -> Value {
    json!({"complete":true,"attempts":[attempt(1,1,"s1")],"assessments":[judgment(1,"correct")]})
}
fn objective() -> Value {
    json!({"id":"o","version":"1","required_criteria":["explain"],"allowed_kinds":["explanation"]})
}
fn progress(h: Value) -> Value {
    json!({"history":h,"objectives":[objective()],"required_items":["lesson","task"],"completed_items":["lesson"],"skipped_items":["task"]})
}
fn schedule(h: Value) -> Value {
    json!({"history":h,"objectives":[objective()],"calendars":[{"version":"c1","effective_at":"2026-01-01T00:00:00Z","timezone":"Europe/Warsaw","study_days":[1,2,3,4,5,6,7]}],"now":"2026-10-30T12:00:00Z","policy_version":"review-v1","budget_seconds":1200,"review_percent":25,"review_estimates":{"o":120},"new_work":[{"id":"lesson","seconds":1000}]})
}

#[test]
fn prerequisites_return_cycle_path_and_ignore_unmodeled_related_edges() {
    let e = calculate(
        "check-prerequisites",
        Request {
            schema_version: 1,
            data: json!({"nodes":["b","a"],"prerequisites":[["a","b"],["b","a"]]}),
        },
    )
    .unwrap_err();
    assert_eq!(e.message, "a -> b -> a");
    assert_eq!(
        run(
            "check-prerequisites",
            json!({"nodes":["a","b"],"prerequisites":[["a","b"]]})
        )["valid"],
        true
    );
}
#[test]
fn pending_corrections_branches_cross_attempt_and_cycles() {
    let mut h = history();
    h["assessments"] = json!([]);
    assert_eq!(
        run("resolve-assessments", h.clone())["a1"]["state"],
        "pending"
    );
    h["assessments"] = json!([judgment(1,"incorrect"),{"id":"fix","attempt":"a1","assessment_status":"resolved","outcome":"correct","criteria":{"explain":true},"supersedes":"j1"}]);
    assert_eq!(
        run("resolve-assessments", h.clone())["a1"]["assessment"],
        "fix"
    );
    h["assessments"][1]["supersedes"] = Value::Null;
    assert_eq!(error("resolve-assessments", h.clone()), "assessment_branch");
    h["assessments"][1]["supersedes"] = json!("j1");
    h["assessments"][0]["supersedes"] = json!("fix");
    assert_eq!(
        error("resolve-assessments", h.clone()),
        "supersession_cycle"
    );
    h["assessments"][0]["supersedes"] = Value::Null;
    h["attempts"]
        .as_array_mut()
        .unwrap()
        .push(attempt(2, 2, "s2"));
    h["assessments"][1]["attempt"] = json!("a2");
    assert_eq!(
        error("resolve-assessments", h),
        "cross_attempt_supersession"
    );
}
#[test]
fn preference_scope_and_provenance() {
    let setting = |v| json!({"value":v,"source":"explicit","version":"1"});
    let value = run(
        "resolve-settings",
        json!({"defaults":{"language":setting("en"),"praise":json!({"value":true,"source":"explicit","version":"1"})},"global":{"language":setting("pl")},"course":{"language":setting("fr")},"session":{"language":setting("de")}}),
    );
    assert_eq!(value["language"]["value"], "de");
    assert_eq!(value["language"]["scope"], "session");
    assert_eq!(value["praise"]["scope"], "default");
}
#[test]
fn comparison_rules_are_explicit() {
    assert_eq!(
        run(
            "grade-answer",
            json!({"rule_version":"v1","comparison":{"kind":"trimmed","expected":"two","ascii_case_insensitive":true},"answer":" TWO "})
        )["outcome"],
        "correct"
    );
    assert_eq!(
        run(
            "grade-answer",
            json!({"rule_version":"v1","comparison":{"kind":"numeric","expected":2.0,"absolute_tolerance":0.1},"answer":2.05})
        )["outcome"],
        "correct"
    );
    assert_eq!(
        error(
            "grade-answer",
            json!({"rule_version":"v1","comparison":{"kind":"option_set","expected":["a"],"allowed":["a","b"]},"answer":["c"]})
        ),
        "option_id"
    );
}
#[test]
fn skipped_completion_is_not_evidence_and_readiness_does_not_change_evidence() {
    let mut h = history();
    h["attempts"] = json!([]);
    h["assessments"] = json!([]);
    let p = run("compute-progress", progress(h));
    assert_eq!(p["lesson_complete"], true);
    assert_eq!(p["objectives"]["o"]["state"], "not_started");
}
#[test]
fn assistance_uncertainty_and_failed_review() {
    for assistance in ["hints", "solution", "cheat_sheet", "unknown"] {
        let mut h = history();
        h["attempts"][0]["assistance"] = json!(assistance);
        assert_eq!(
            run("compute-progress", progress(h))["objectives"]["o"]["state"],
            "practicing"
        );
    }
    let mut h = history();
    h["attempts"]
        .as_array_mut()
        .unwrap()
        .push(attempt(2, 2, "s2"));
    h["assessments"]
        .as_array_mut()
        .unwrap()
        .push(judgment(2, "incorrect"));
    let p = run("compute-progress", progress(h.clone()));
    assert_eq!(p["objectives"]["o"]["state"], "practicing");
    assert_eq!(p["objectives"]["o"]["historical_successes"], json!(["a1"]));
    h["assessments"][1]["assessment_status"] = json!("disputed");
    assert_eq!(
        run("compute-progress", progress(h))["objectives"]["o"]["state"],
        "demonstrated"
    );
}
#[test]
fn incomplete_history_fails_closed() {
    let mut h = history();
    h["complete"] = json!(false);
    assert_eq!(error("compute-progress", progress(h)), "incomplete_history");
}
#[test]
fn intervals_terminal_stage_and_same_session_limit() {
    let mut h = history();
    for (n, day, s) in [
        (2, 2, "s2"),
        (3, 2, "s2"),
        (4, 5, "s3"),
        (5, 12, "s4"),
        (6, 26, "s5"),
    ] {
        h["attempts"]
            .as_array_mut()
            .unwrap()
            .push(attempt(n, day, s));
        h["assessments"]
            .as_array_mut()
            .unwrap()
            .push(judgment(n, "correct"));
    }
    let result = run("compute-schedule", schedule(h));
    assert_eq!(result["reviews"]["o"]["stage"], 3);
    assert_eq!(result["reviews"]["o"]["due_date"], "2026-11-09");
}
#[test]
fn lapse_retains_stage_and_next_study_day_even_with_immediate_retry() {
    let mut h = history();
    for (n, day, s, outcome) in [
        (2, 2, "s2", "correct"),
        (3, 9, "s3", "incorrect"),
        (4, 9, "s3", "correct"),
    ] {
        h["attempts"]
            .as_array_mut()
            .unwrap()
            .push(attempt(n, day, s));
        h["assessments"]
            .as_array_mut()
            .unwrap()
            .push(judgment(n, outcome));
    }
    let mut input = schedule(h);
    input["calendars"][0]["study_days"] = json!([1, 2, 3, 4, 5]);
    let result = run("compute-schedule", input);
    assert_eq!(result["reviews"]["o"]["stage"], 1);
    assert_eq!(result["reviews"]["o"]["due_date"], "2026-10-12");
}
#[test]
fn unknown_disputed_and_unassessable_do_not_advance() {
    for mode in ["unknown", "disputed", "unassessable"] {
        let mut h = history();
        h["attempts"]
            .as_array_mut()
            .unwrap()
            .push(attempt(2, 2, "s2"));
        h["assessments"]
            .as_array_mut()
            .unwrap()
            .push(judgment(2, "correct"));
        match mode {
            "unknown" => h["attempts"][1]["assistance"] = json!(mode),
            "disputed" => h["assessments"][1]["assessment_status"] = json!(mode),
            _ => h["assessments"][1]["outcome"] = json!(mode),
        };
        let r = run("compute-schedule", schedule(h));
        assert_eq!(r["reviews"]["o"]["stage"], 0);
        assert_eq!(r["reviews"]["o"]["due_date"], "2026-10-02");
    }
}
#[test]
fn budget_overflow_fill_and_unused_time() {
    let mut input = schedule(history());
    input["review_estimates"]["o"] = json!(400);
    input["objectives"].as_array_mut().unwrap().push({
        let mut o = objective();
        o["id"] = json!("p");
        o
    });
    let mut a = attempt(2, 1, "s1");
    a["objective"] = json!("p");
    a["is_review"] = json!(false);
    input["history"]["attempts"].as_array_mut().unwrap().push(a);
    input["history"]["assessments"]
        .as_array_mut()
        .unwrap()
        .push(judgment(2, "correct"));
    input["review_estimates"]["p"] = json!(120);
    let r = run("compute-schedule", input);
    assert_eq!(r["review_allowance_seconds"], 300);
    assert_eq!(r["deferred_reviews"][0]["work"]["id"], "o");
    assert_eq!(r["selected_reviews"][0]["id"], "p");
    assert_eq!(r["unused_seconds"], 80);
}
#[test]
fn replay_normalizes_input_order_and_correction_uses_original_attempt_time() {
    let mut h = history();
    h["attempts"]
        .as_array_mut()
        .unwrap()
        .push(attempt(2, 2, "s2"));
    h["assessments"]
        .as_array_mut()
        .unwrap()
        .push(judgment(2, "incorrect"));
    h["assessments"].as_array_mut().unwrap().push(json!({"id":"fix","attempt":"a2","assessment_status":"resolved","outcome":"correct","criteria":{"explain":true},"supersedes":"j2"}));
    let r = run("compute-schedule", schedule(h.clone()));
    h["attempts"].as_array_mut().unwrap().reverse();
    h["assessments"].as_array_mut().unwrap().reverse();
    assert_eq!(r, run("compute-schedule", schedule(h)));
    assert_eq!(r["reviews"]["o"]["due_date"], "2026-10-05");
}
#[test]
fn dst_uses_local_days_and_calendar_changes_preserve_overdue_dates() {
    let mut h = history();
    h["attempts"][0]["timestamp"] = json!("2026-10-24T22:30:00Z");
    let mut input = schedule(h);
    let r = run("compute-schedule", input.clone());
    assert_eq!(r["reviews"]["o"]["due_date"], "2026-10-26");
    input["calendars"].as_array_mut().unwrap().push(json!({"version":"c2","effective_at":"2026-10-28T00:00:00Z","timezone":"America/New_York","study_days":[1]}));
    assert_eq!(
        run("compute-schedule", input)["reviews"]["o"]["due_date"],
        "2026-10-26"
    );
}

#[test]
fn support_trigger_counts_unique_saved_origins_and_early_obstruction() {
    let mut input = json!({"strategy":"small-step","applications":[{"id":"1","origin_id":"a","strategy":"small-step"},{"id":"2","origin_id":"b","strategy":"small-step"},{"id":"3","origin_id":"c","strategy":"small-step"}],"reviewed_origins":[],"threshold":3,"obstructive":false,"policy_version":"support-v1"});
    assert_eq!(run("review-support", input.clone())["review_due"], true);
    input["reviewed_origins"] = json!(["a"]);
    assert_eq!(run("review-support", input.clone())["review_due"], false);
    input["obstructive"] = json!(true);
    assert_eq!(run("review-support", input)["review_due"], true);
}
#[test]
fn initial_assisted_practice_has_follow_up_without_claiming_evidence() {
    let mut h = history();
    h["attempts"][0]["assistance"] = json!("hints");
    let r = run("compute-schedule", schedule(h));
    assert_eq!(r["reviews"]["o"]["evidence_established"], false);
    assert_eq!(r["reviews"]["o"]["due_date"], "2026-10-02");
}
#[test]
fn explicit_dispute_and_assistance_clarification_affect_replay() {
    let mut h = history();
    h["disputed_assessments"] = json!(["j1"]);
    assert_eq!(
        run("resolve-assessments", h.clone())["a1"]["state"],
        "disputed"
    );
    assert_eq!(run("compute-schedule", schedule(h))["reviews"], json!({}));
    let mut h = history();
    h["assistance_updates"] = json!({"a1":"cheat_sheet"});
    assert_eq!(
        run("compute-progress", progress(h))["objectives"]["o"]["state"],
        "practicing"
    );
}
#[test]
fn calendar_revision_recalculates_future_and_short_budget_floors() {
    let mut input = schedule(history());
    input["now"] = json!("2026-10-01T23:00:00Z");
    input["budget_seconds"] = json!(3);
    input["calendars"].as_array_mut().unwrap().push(json!({"version":"c2","effective_at":"2026-10-01T13:00:00Z","timezone":"America/New_York","study_days":[1]}));
    let r = run("compute-schedule", input);
    assert_eq!(r["reviews"]["o"]["original_due_date"], "2026-10-02");
    assert_eq!(r["reviews"]["o"]["due_date"], "2026-10-05");
    assert_eq!(r["review_allowance_seconds"], 0);
}
