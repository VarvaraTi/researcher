//! Narrow publication adapter. All active writes go through OKF's guarded changeset API.
use crate::{
    rules::{check_state, Record, State},
    Error, Request, Result, VERSION,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    okf: PathBuf,
    bundle: PathBuf,
    #[serde(default)]
    scope_bundles: Vec<String>,
    #[serde(default)]
    study: Option<String>,
    #[serde(skip)]
    catalog_overrides: Option<PathBuf>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Publication {
    context: Context,
    changeset: Value,
    /// Digest obtained from OKF changeset plan for this exact changeset.
    expected_digest: String,
    #[serde(default)]
    mode: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveAttempt {
    context: Context,
    record: Record,
}

fn command(context: &Context, args: &[String]) -> Result<String> {
    let mut process = Command::new(fs::canonicalize(&context.okf)?);
    process
        .current_dir(fs::canonicalize(&context.bundle)?)
        .args(args);
    if let Some(overrides) = &context.catalog_overrides {
        process.env("OKF_CATALOG_OVERRIDES", overrides);
    }
    let output = process.output()?;
    if !output.status.success() {
        return Err(Error::new(
            "okf",
            format!(
                "{}: {}{}",
                args.first().map(String::as_str).unwrap_or("command"),
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            ),
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| Error::new("encoding", e.to_string()))
}
fn ndjson(output: &str) -> Result<Vec<Value>> {
    let records: Vec<Value> = output
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| serde_json::from_str(s).map_err(Error::from))
        .collect::<Result<_>>()?;
    let start = records.first();
    let end = records.last();
    if start.is_none_or(|v| {
        v["kind"] != "operation-start" || v["schema_version"] != 1 || v["stream_version"] != 1
    }) || end.is_none_or(|v| {
        v["kind"] != "operation-complete"
            || v["schema_version"] != 1
            || v["outcome"] != "ok"
            || v["exit_code"] != 0
            || v["completeness"]["delivery"] != "complete"
            || !matches!(
                v["completeness"]["examination"].as_str(),
                Some("complete" | "not-applicable")
            )
    }) {
        return Err(Error::new(
            "okf_contract",
            "Missing, unsuccessful or incomplete OKF operation envelope",
        ));
    }
    Ok(records[1..records.len() - 1].to_vec())
}
fn check_cli(context: &Context) -> Result<()> {
    let version = command(context, &["version".into()])?;
    if !version.split_whitespace().any(|s| s == "0.6.0") {
        return Err(Error::new(
            "okf_version",
            format!("Expected pinned OKF 0.6.0, got {}", version.trim()),
        ));
    }
    if context.bundle.join(".okf-transaction").exists() {
        return Err(Error::new(
            "recovery_required",
            "Run OKF changeset recover, inspect its result, then reconcile",
        ));
    }
    Ok(())
}
fn bundle_arg(context: &Context) -> String {
    fs::canonicalize(&context.bundle)
        .unwrap_or_else(|_| context.bundle.clone())
        .to_string_lossy()
        .into_owned()
}
fn inventory(context: &Context) -> Result<State> {
    let output = ndjson(&command(
        context,
        &[
            "search".into(),
            bundle_arg(context),
            "--full-scan".into(),
            "--json".into(),
        ],
    )?)?;
    let summary = output
        .iter()
        .find(|v| v["kind"] == "query-summary")
        .ok_or_else(|| Error::new("incomplete_history", "Missing OKF query-summary"))?;
    if summary["scan_complete"] != true
        || summary["partial"] != false
        || summary["has_more"] != false
        || output.iter().any(|v| v["kind"] == "warning")
    {
        return Err(Error::new(
            "incomplete_history",
            "OKF query is incomplete or has warnings",
        ));
    }
    let mut records = Vec::new();
    for entry in output
        .iter()
        .filter(|v| v.get("type").is_some() && v.get("id").is_some())
    {
        let id = entry["id"]
            .as_str()
            .ok_or_else(|| Error::new("okf_contract", "Expected concept ID"))?;
        safe_id(id)?;
        let path = context
            .bundle
            .join(format!("{}.md", id.trim_start_matches('/')));
        let raw = fs::read_to_string(path)?;
        records.push(parse_record(id, &raw)?);
    }
    if summary["returned"].as_u64() != Some(records.len() as u64) {
        return Err(Error::new("okf_contract", "Inventory count mismatch"));
    }
    records.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(State {
        complete: true,
        records,
    })
}
pub fn parse_record(id: &str, text: &str) -> Result<Record> {
    let remaining = text
        .strip_prefix("---\n")
        .ok_or_else(|| Error::new("frontmatter", id))?;
    let (yaml, body) = remaining
        .split_once("\n---\n")
        .ok_or_else(|| Error::new("frontmatter", id))?;
    let metadata: BTreeMap<String, Value> =
        serde_yaml::from_str(yaml).map_err(|e| Error::new("frontmatter", e.to_string()))?;
    Ok(Record {
        id: id.into(),
        metadata,
        body: body.into(),
    })
}
pub fn render(record: &Record) -> Result<String> {
    // JSON is valid YAML; keeping type first also respects OKF authoring order.
    let mut fields = Vec::new();
    for key in std::iter::once("type").chain(
        record
            .metadata
            .keys()
            .map(String::as_str)
            .filter(|k| *k != "type"),
    ) {
        if let Some(value) = record.metadata.get(key) {
            fields.push(format!(
                "{}: {}",
                serde_json::to_string(key)?,
                serde_json::to_string(value)?
            ));
        }
    }
    Ok(format!("---\n{}\n---\n{}", fields.join("\n"), record.body))
}
fn safe_id(id: &str) -> Result<()> {
    if !id.starts_with('/')
        || id.ends_with(".md")
        || id[1..].split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || !part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
    {
        return Err(Error::new(
            "record_id",
            format!("Expected /directory/stable-id, without extension: {id}"),
        ));
    }
    Ok(())
}
fn digest_plan(context: &Context, changeset: &Value) -> Result<String> {
    let output = ndjson(&command(
        context,
        &[
            "changeset".into(),
            "plan".into(),
            bundle_arg(context),
            "--from".into(),
            changeset.to_string(),
            "--json".into(),
        ],
    )?)?;
    output
        .iter()
        .find(|v| v["kind"] == "changeset-summary")
        .and_then(|v| v["base_digest"].as_str())
        .map(String::from)
        .ok_or_else(|| Error::new("okf_contract", "Missing changeset digest"))
}
fn apply(context: &Context, changeset: &Value, digest: &str) -> Result<()> {
    command(
        context,
        &[
            "changeset".into(),
            "apply".into(),
            bundle_arg(context),
            "--from".into(),
            changeset.to_string(),
            "--expect".into(),
            digest.into(),
            "--json".into(),
        ],
    )?;
    Ok(())
}
fn copy_bundle(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let typ = entry.file_type()?;
        if typ.is_symlink() {
            return Err(Error::new(
                "staging",
                "Symlinked bundle content requires explicit relocation before publication",
            ));
        }
        if entry.file_name() == ".okf-transaction.lock" {
            continue;
        }
        if typ.is_dir() {
            copy_bundle(&entry.path(), &target.join(entry.file_name()))?;
        } else if typ.is_file() {
            fs::copy(entry.path(), target.join(entry.file_name()))?;
        }
    }
    Ok(())
}
fn structural(context: &Context) -> Result<()> {
    command(
        context,
        &["validate".into(), bundle_arg(context), "--json".into()],
    )?;
    let mut args = vec![
        "lint".into(),
        bundle_arg(context),
        "--fail-on".into(),
        "never".into(),
        "--json".into(),
    ];
    for scope in &context.scope_bundles {
        args.extend(["--scope-bundle".into(), scope.clone()]);
    }
    for finding in ndjson(&command(context, &args)?)? {
        if finding["kind"] == "scope" {
            if finding["unavailable"]
                .as_array()
                .is_none_or(|v| !v.is_empty())
            {
                return Err(Error::new(
                    "structural_gate",
                    "Unavailable bundle in validation scope",
                ));
            }
            continue;
        }
        let rule = finding["rule"].as_str().unwrap_or("");
        // Index coverage, formatting and descriptive recommendations do not block learning.
        if finding["kind"] != "finding"
            || !matches!(
                rule,
                "frontmatter-order"
                    | "index-coverage"
                    | "missing-description"
                    | "orphan"
                    | "missing-title"
            )
        {
            return Err(Error::new("structural_gate", finding.to_string()));
        }
    }
    Ok(())
}
fn immutable(before: &State, after: &State) -> Result<()> {
    for old in &before.records {
        let typ = old
            .metadata
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("");
        if matches!(
            typ,
            "Attempt"
                | "Assessment"
                | "Exercise"
                | "Objective"
                | "LearningPlan"
                | "Source"
                | "AdaptationPolicy"
                | "Note"
                | "ReadinessChoice"
                | "SkipEvent"
                | "AssistanceEvent"
                | "StrategyApplication"
                | "LearnerObservation"
                | "LearnerModelReview"
        ) || (typ == "Lesson" && old.metadata.get("readiness") == Some(&json!("ready")))
        {
            let new = after.records.iter().find(|r| r.id == old.id);
            if new.is_none_or(|new| new.metadata != old.metadata || new.body != old.body) {
                return Err(Error::new(
                    "immutable_record",
                    format!("{}: create a new version/correction instead", old.id),
                ));
            }
        }
    }
    Ok(())
}
fn retained_artifacts(original: &Path, staged: &Path) -> Result<()> {
    if !original.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(original)? {
        let entry = entry?;
        let target = staged.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            retained_artifacts(&entry.path(), &target)?;
        } else if fs::read(entry.path())?
            != fs::read(&target)
                .map_err(|_| Error::new("historical_artifact", target.display().to_string()))?
        {
            return Err(Error::new(
                "historical_artifact",
                "Retain snapshots under new paths instead of replacing them",
            ));
        }
    }
    Ok(())
}
fn outside_study(root: &Path, selected: &str) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    fn walk(
        root: &Path,
        dir: &Path,
        selected: &Path,
        files: &mut BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|e| Error::new("study_boundary", e.to_string()))?;
            if relative.starts_with(selected) || relative == Path::new(".okf-transaction.lock") {
                continue;
            }
            if entry.file_type()?.is_dir() {
                walk(root, &path, selected, files)?;
            } else {
                files.insert(relative.to_path_buf(), fs::read(&path)?);
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(
        root,
        root,
        Path::new(crate::studies::root(selected)?.trim_start_matches('/')),
        &mut files,
    )?;
    Ok(files)
}
fn library_immutable(before: &State, after: &State) -> Result<()> {
    for old in &before.records {
        let typ = old
            .metadata
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("");
        let new = after.records.iter().find(|r| r.id == old.id);
        if new.is_none()
            || (!matches!(typ, "Concept" | "SourceSelection")
                && new.is_some_and(|r| r.metadata != old.metadata || r.body != old.body))
        {
            return Err(Error::new(
                "immutable_record",
                format!("{}: create a new record", old.id),
            ));
        }
    }
    Ok(())
}

fn library_target<'a>(state: &'a State, owner: &Record, field: &str) -> Result<&'a Record> {
    let reference = owner.metadata[field].as_str().ok_or_else(|| {
        Error::new(
            "library_reference",
            format!("{}: missing {field}", owner.id),
        )
    })?;
    library_resolve(state, owner, reference)
}

fn library_resolve<'a>(state: &'a State, owner: &Record, reference: &str) -> Result<&'a Record> {
    let mut parts: Vec<&str> = if reference.starts_with('/') {
        Vec::new()
    } else {
        owner.id.trim_start_matches('/').split('/').collect()
    };
    if !reference.starts_with('/') {
        parts.pop();
    }
    for part in reference.trim_end_matches(".md").split('/') {
        match part {
            "" | "." => (),
            ".." => {
                if parts.pop().is_none() {
                    return Err(Error::new("library_reference", reference));
                }
            }
            _ => parts.push(part),
        }
    }
    let id = format!("/{}", parts.join("/"));
    state
        .records
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| Error::new("library_reference", id))
}

fn library_coverage(record: &Record) -> Result<()> {
    let count = record.metadata["page_count"].as_u64().unwrap_or(0);
    for field in [
        "indexed_ranges",
        "no_text_ranges",
        "failed_ranges",
        "passage_refs",
    ] {
        if !record.metadata[field].is_array() {
            return Err(Error::new(
                "library_coverage",
                format!("Missing list: {field}"),
            ));
        }
    }
    if count == 0 {
        return Ok(());
    }
    let mut pages = BTreeSet::new();
    let mut indexed = 0usize;
    let mut missing = 0usize;
    for field in ["indexed_ranges", "no_text_ranges", "failed_ranges"] {
        let ranges = record.metadata[field]
            .as_array()
            .ok_or_else(|| Error::new("library_coverage", field))?;
        for item in ranges {
            let text = item
                .as_str()
                .ok_or_else(|| Error::new("library_coverage", field))?;
            let (first, last) = text.split_once('-').unwrap_or((text, text));
            let first: u64 = first
                .parse()
                .map_err(|_| Error::new("library_coverage", text))?;
            let last: u64 = last
                .parse()
                .map_err(|_| Error::new("library_coverage", text))?;
            if first == 0 || last < first || last > count {
                return Err(Error::new("library_coverage", text));
            }
            for page in first..=last {
                if !pages.insert(page) {
                    return Err(Error::new("library_coverage", "Overlapping ranges"));
                }
                if field == "indexed_ranges" {
                    indexed += 1;
                } else {
                    missing += 1;
                }
            }
        }
    }
    if pages.len() as u64 != count {
        return Err(Error::new("library_coverage", "Incomplete coverage"));
    }
    let passages = record.metadata["passage_refs"]
        .as_array()
        .ok_or_else(|| Error::new("library_coverage", "Missing passages"))?;
    let expected = if indexed == 0 {
        "failed"
    } else if missing > 0 {
        "partial"
    } else {
        "complete"
    };
    if record.metadata["manifest_status"] != json!(expected)
        || (indexed == 0) != passages.is_empty()
    {
        return Err(Error::new(
            "library_coverage",
            "Status or passages disagree with coverage",
        ));
    }
    Ok(())
}

fn library_state(state: &State, bundle: &Path) -> Result<()> {
    for record in &state.records {
        let typ = record
            .metadata
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !matches!(
            typ,
            "Source"
                | "SourcePassage"
                | "IndexManifest"
                | "Concept"
                | "SourceSelection"
                | "PassageReview"
        ) {
            return Err(Error::new(
                "library_type",
                format!("Unexpected library type: {typ}"),
            ));
        }
        if typ == "Source" {
            let location = record.metadata["location"]
                .as_str()
                .ok_or_else(|| Error::new("library_artifact", "Missing source artifact"))?;
            let path = Path::new(location);
            if !path.starts_with("references")
                || path
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(_)))
            {
                return Err(Error::new(
                    "library_artifact",
                    "Artifact must be inside references",
                ));
            }
            let full = fs::canonicalize(bundle.join(path))?;
            let references = fs::canonicalize(bundle.join("references"))?;
            if !full.starts_with(references) {
                return Err(Error::new(
                    "library_artifact",
                    "Artifact escapes references",
                ));
            }
            let digest = format!("sha256:{:x}", Sha256::digest(fs::read(full)?));
            if record.metadata["content_digest"] != json!(digest) {
                return Err(Error::new("library_digest", &record.id));
            }
        }
        if typ == "SourcePassage" && record.metadata["review_status"] != json!("unreviewed") {
            return Err(Error::new(
                "library_review",
                "Extraction cannot assert passage review",
            ));
        }
        if typ == "SourcePassage"
            && (record.metadata["pdf_page_start"].as_u64().unwrap_or(0) == 0
                || record.metadata["pdf_page_end"].as_u64().unwrap_or(0)
                    < record.metadata["pdf_page_start"].as_u64().unwrap_or(0))
        {
            return Err(Error::new("library_locator", &record.id));
        }
        if matches!(typ, "IndexManifest" | "SourcePassage" | "SourceSelection") {
            let source = library_target(state, record, "source_ref")?;
            if source.metadata["type"] != json!("Source") {
                return Err(Error::new("library_reference", "Expected Source"));
            }
            if typ != "SourceSelection"
                && source.metadata["content_digest"] != record.metadata["source_digest"]
            {
                return Err(Error::new("library_digest", &record.id));
            }
        }
        if matches!(typ, "SourcePassage" | "SourceSelection") {
            let manifest = library_target(state, record, "manifest_ref")?;
            if manifest.metadata["type"] != json!("IndexManifest")
                || library_target(state, manifest, "source_ref")?.id
                    != library_target(state, record, "source_ref")?.id
            {
                return Err(Error::new("library_reference", "Manifest source mismatch"));
            }
            if typ == "SourcePassage"
                && (manifest.metadata["source_digest"] != record.metadata["source_digest"]
                    || record.metadata["pdf_page_end"].as_u64().unwrap_or(0)
                        > manifest.metadata["page_count"].as_u64().unwrap_or(0))
            {
                return Err(Error::new("library_locator", &record.id));
            }
            if typ == "SourceSelection"
                && (manifest.metadata["manifest_status"] == json!("failed")
                    || manifest.metadata["passage_refs"]
                        .as_array()
                        .is_none_or(|v| v.is_empty()))
            {
                return Err(Error::new(
                    "library_selection",
                    "Selected run must contain usable passages",
                ));
            }
        }
        if typ == "IndexManifest" {
            library_coverage(record)?;
            let refs = record.metadata["passage_refs"]
                .as_array()
                .ok_or_else(|| Error::new("library_coverage", "Missing passage references"))?;
            let mut listed = BTreeSet::new();
            let mut text_pages = BTreeSet::new();
            for reference in refs {
                let text = reference
                    .as_str()
                    .ok_or_else(|| Error::new("library_reference", "Invalid passage reference"))?;
                let passage = library_resolve(state, record, text)?;
                let mut current = None;
                let mut content = false;
                for line in passage.body.lines() {
                    if let Some(page) = line
                        .trim()
                        .strip_prefix("<!-- PDF page ")
                        .and_then(|s| s.strip_suffix(" -->"))
                    {
                        if current.is_some() && !content {
                            return Err(Error::new("library_coverage", "Empty passage page"));
                        }
                        let page: u64 = page
                            .parse()
                            .map_err(|_| Error::new("library_locator", "Invalid page marker"))?;
                        let start = passage.metadata["pdf_page_start"].as_u64().unwrap_or(0);
                        let end = passage.metadata["pdf_page_end"].as_u64().unwrap_or(0);
                        if page < start || page > end || current.is_some_and(|v| page <= v) {
                            return Err(Error::new("library_locator", "Out-of-order page marker"));
                        }
                        current = Some(page);
                        text_pages.insert(page);
                        content = false;
                    } else if !line.trim().is_empty() {
                        content = true;
                    }
                }
                if current.is_none() || !content {
                    return Err(Error::new(
                        "library_coverage",
                        "Passage must retain page-marked text",
                    ));
                }
                if passage.metadata["type"] != json!("SourcePassage")
                    || library_target(state, passage, "manifest_ref")?.id != record.id
                    || !listed.insert(passage.id.clone())
                {
                    return Err(Error::new(
                        "library_coverage",
                        "Invalid or duplicated manifest passage",
                    ));
                }
            }
            let mut indexed_pages = BTreeSet::new();
            for range in record.metadata["indexed_ranges"].as_array().unwrap() {
                let text = range.as_str().unwrap();
                let (start, end) = text.split_once('-').unwrap_or((text, text));
                let start: u64 = start.parse().unwrap();
                let end: u64 = end.parse().unwrap();
                indexed_pages.extend(start..=end);
            }
            if text_pages != indexed_pages {
                return Err(Error::new(
                    "library_coverage",
                    "Indexed pages do not match retained text",
                ));
            }
            for passage in state
                .records
                .iter()
                .filter(|r| r.metadata["type"] == json!("SourcePassage"))
            {
                if library_target(state, passage, "manifest_ref")?.id == record.id
                    && !listed.contains(&passage.id)
                {
                    return Err(Error::new(
                        "library_coverage",
                        "Passage missing from its manifest",
                    ));
                }
            }
        }
        if typ == "IndexManifest"
            && record.metadata["page_count"] == json!(0)
            && (record.metadata["manifest_status"] != json!("failed")
                || record.metadata["document_failure"]
                    .as_str()
                    .is_none_or(str::is_empty)
                || [
                    "indexed_ranges",
                    "no_text_ranges",
                    "failed_ranges",
                    "passage_refs",
                ]
                .iter()
                .any(|key| {
                    record.metadata[*key]
                        .as_array()
                        .is_none_or(|v| !v.is_empty())
                }))
        {
            return Err(Error::new(
                "library_coverage",
                "Unknown page count requires a document failure and empty coverage",
            ));
        }
    }
    let mut selections = BTreeSet::new();
    for selection in state
        .records
        .iter()
        .filter(|r| r.metadata["type"] == json!("SourceSelection"))
    {
        let source = library_target(state, selection, "source_ref")?;
        if !selections.insert(source.id.clone()) {
            return Err(Error::new(
                "library_selection",
                "Multiple selections for one source",
            ));
        }
    }
    library_reviews(state)?;
    Ok(())
}

fn library_reviews(state: &State) -> Result<()> {
    let mut groups: BTreeMap<String, Vec<&Record>> = BTreeMap::new();
    let mut replaced = BTreeSet::new();
    for review in state
        .records
        .iter()
        .filter(|r| r.metadata["type"] == json!("PassageReview"))
    {
        let passage = library_target(state, review, "passage_ref")?;
        if passage.metadata["type"] != json!("SourcePassage") {
            return Err(Error::new("library_review", "Expected passage"));
        }
        groups.entry(passage.id.clone()).or_default().push(review);
        if review.metadata.contains_key("supersedes") {
            let prior = library_target(state, review, "supersedes")?;
            if prior.metadata["type"] != json!("PassageReview")
                || library_target(state, prior, "passage_ref")?.id != passage.id
                || !replaced.insert(prior.id.clone())
            {
                return Err(Error::new(
                    "library_review",
                    "Invalid or branched review history",
                ));
            }
        }
    }
    for reviews in groups.values() {
        if reviews.iter().filter(|r| !replaced.contains(&r.id)).count() != 1 {
            return Err(Error::new(
                "library_review",
                "Review history requires one terminal record",
            ));
        }
        // A disconnected cycle may coexist with one terminal; traverse from terminal.
        let mut current = *reviews.iter().find(|r| !replaced.contains(&r.id)).unwrap();
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(current.id.clone()) {
                return Err(Error::new("library_review", "Review cycle"));
            }
            if !current.metadata.contains_key("supersedes") {
                break;
            }
            current = library_target(state, current, "supersedes")?;
        }
        if visited.len() != reviews.len() {
            return Err(Error::new("library_review", "Disconnected review history"));
        }
    }
    Ok(())
}

fn publish(publication: Publication) -> Result<Value> {
    let Publication {
        context,
        changeset,
        expected_digest,
        mode,
    } = publication;
    let library = match mode.as_deref() {
        None => false,
        Some("library") => true,
        _ => return Err(Error::new("publication_mode", "Unknown publication mode")),
    };
    let identity = bundle_identity(&context)?;
    if library != (identity == "learning.library") {
        return Err(Error::new(
            "publication_mode",
            "Library mode requires the registered learning.library bundle",
        ));
    }
    check_cli(&context)?;
    if digest_plan(&context, &changeset)? != expected_digest {
        return Err(Error::new(
            "stale_guard",
            "Bundle changed since the reviewed plan",
        ));
    }
    let before = inventory(&context)?;
    if context.study.is_none()
        && before
            .records
            .iter()
            .any(|r| r.metadata.get("type") == Some(&json!("Study")))
    {
        return Err(Error::new(
            "study_required",
            "Select a study before publication",
        ));
    }
    // A sibling staging root preserves document-relative links to other bundles.
    let parent = context
        .bundle
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let stage = tempfile::Builder::new()
        .prefix(".learning-stage-")
        .tempdir_in(parent)?;
    copy_bundle(&context.bundle, stage.path())?;
    // Register the staged primary under its original catalog identity. Standard
    // cross-bundle sources with bundle_ref need catalog context during planning.
    let primary = bundle_identity(&context)?;
    let overrides_file = tempfile::Builder::new()
        .prefix(".learning-catalog-")
        .tempfile_in(parent)?;
    let mut overrides = serde_json::Map::new();
    for entry in ndjson(&command(&context, &["catalog".into(), "--json".into()])?)? {
        if entry["kind"] == "bundle-registration" {
            if let (Some(id), Some(root)) = (entry["id"].as_str(), entry["root"].as_str()) {
                overrides.insert(
                    id.to_owned(),
                    json!(if id == primary {
                        fs::canonicalize(stage.path())?
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        root.to_owned()
                    }),
                );
            }
        }
    }
    fs::write(
        overrides_file.path(),
        serde_json::to_vec(&json!({"bundles":overrides}))?,
    )?;
    let staged = Context {
        okf: context.okf.clone(),
        bundle: stage.path().to_path_buf(),
        scope_bundles: context.scope_bundles.clone(),
        study: context.study.clone(),
        catalog_overrides: Some(fs::canonicalize(overrides_file.path())?),
    };
    let stage_digest = digest_plan(&staged, &changeset)?;
    apply(&staged, &changeset, &stage_digest)?;
    // Validate external consumers against their real registered roots: their
    // ordinary paths still refer to the live primary, not the temporary copy.
    let staged = Context {
        catalog_overrides: None,
        ..staged
    };
    structural(&staged)?;
    let proposed = inventory(&staged)?;
    if library {
        library_state(&proposed, &staged.bundle)?;
        library_immutable(&before, &proposed)?;
    } else {
        let shared_links = proposed.records.iter().any(|record| {
            [
                "shared_source_refs",
                "shared_passage_refs",
                "shared_concept_refs",
            ]
            .iter()
            .any(|field| {
                record
                    .metadata
                    .get(*field)
                    .is_some_and(|value| value.as_array().is_some_and(|refs| !refs.is_empty()))
            })
        });
        if shared_links
            && !context
                .scope_bundles
                .iter()
                .any(|id| id == "learning.library")
        {
            return Err(Error::new(
                "library_scope_required",
                "Shared library links require explicit learning.library validation scope",
            ));
        }
        check_state(&proposed)?;
        crate::studies::selection(&proposed, context.study.as_deref())?;
    }
    if let Some(study) = context.study.as_deref() {
        if outside_study(&context.bundle, study)? != outside_study(&staged.bundle, study)? {
            return Err(Error::new(
                "study_boundary",
                "Publication changed files outside the selected study",
            ));
        }
        let prefix = format!("{}/", crate::studies::root(study)?);
        for record in before.records.iter().chain(proposed.records.iter()) {
            if !record.id.starts_with(&prefix) {
                let old = before.records.iter().find(|r| r.id == record.id);
                let new = proposed.records.iter().find(|r| r.id == record.id);
                if serde_json::to_value(old)? != serde_json::to_value(new)? {
                    return Err(Error::new("study_boundary", &record.id));
                }
            }
        }
    }
    if !library {
        immutable(&before, &proposed)?;
    }
    retained_artifacts(
        &context.bundle.join("references"),
        &staged.bundle.join("references"),
    )?;
    for record in before
        .records
        .iter()
        .filter(|r| r.metadata.get("type") == Some(&json!("Study")))
    {
        let dir = crate::studies::root(&record.id)?.trim_start_matches('/');
        retained_artifacts(
            &context.bundle.join(dir).join("references"),
            &staged.bundle.join(dir).join("references"),
        )?;
    }
    // Recheck the original reviewed baseline after staging. OKF also checks during apply.
    if digest_plan(&context, &changeset)? != expected_digest {
        return Err(Error::new(
            "stale_guard",
            "Intervening edit during validation",
        ));
    }
    apply(&context, &changeset, &expected_digest)?;
    Ok(
        json!({"publication": "applied", "base_digest": expected_digest, "records": proposed.records.len()}),
    )
}
fn save_attempt(input: SaveAttempt) -> Result<Value> {
    save_record(input, "Attempt", None)
}
fn save_record(input: SaveAttempt, typ: &str, directory: Option<&str>) -> Result<Value> {
    check_cli(&input.context)?;
    safe_id(&input.record.id)?;
    if input.record.metadata.get("type") != Some(&json!(typ)) {
        return Err(Error::new("record_type", format!("Expected {typ}")));
    }
    if let Some(directory) = directory {
        let origin = input
            .record
            .metadata
            .get("origin_id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::new("origin_id", "Stable origin is required"))?;
        let prefix = input
            .context
            .study
            .as_deref()
            .map(crate::studies::root)
            .transpose()?
            .unwrap_or("");
        let expected = format!(
            "{prefix}/{directory}/{:x}",
            Sha256::digest(origin.as_bytes())
        );
        if input.record.id != expected {
            return Err(Error::new("origin_path", format!("Use {expected}")));
        }
    }
    let state = inventory(&input.context)?;
    crate::studies::selection(&state, input.context.study.as_deref())?;
    if input
        .context
        .study
        .as_deref()
        .is_some_and(|id| crate::studies::owner(&input.record) != Some(id))
    {
        return Err(Error::new("study_boundary", &input.record.id));
    }
    if let Some(saved) = state.records.iter().find(|r| r.id == input.record.id) {
        if saved.metadata == input.record.metadata && saved.body == input.record.body {
            return Ok(json!({"save": "identical_retry", "id": saved.id}));
        }
        return Err(Error::new(
            "submission_conflict",
            "Stable submission path already contains a different answer or context",
        ));
    }
    let changeset = json!({"version":1,"operations":[{"op":"create","path":format!("{}.md", input.record.id.trim_start_matches('/')), "content":render(&input.record)?}]});
    let expected_digest = digest_plan(&input.context, &changeset)?;
    publish(Publication {
        context: input.context,
        changeset,
        expected_digest,
        mode: None,
    })?;
    Ok(json!({"save":"created", "id": input.record.id}))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelAudit {
    context: Context,
    learner_context: Context,
}
fn bundle_identity(context: &Context) -> Result<String> {
    let records = ndjson(&command(
        context,
        &[
            "bundle".into(),
            "context".into(),
            bundle_arg(context),
            "--json".into(),
        ],
    )?)?;
    records
        .iter()
        .find(|r| r["kind"] == "bundle-context" && r["primary"] == true)
        .and_then(|r| r["bundle"].as_str())
        .map(String::from)
        .ok_or_else(|| Error::new("bundle_identity", "Missing primary bundle identity"))
}
fn audit_model(input: ModelAudit) -> Result<Value> {
    for context in [&input.context, &input.learner_context] {
        check_cli(context)?;
        structural(context)?;
    }
    let source = inventory(&input.context)?;
    let learner = inventory(&input.learner_context)?;
    check_state(&source)?;
    check_state(&learner)?;
    crate::studies::selection(&source, input.context.study.as_deref())?;
    if !learner
        .records
        .iter()
        .any(|r| r.metadata.get("type") == Some(&json!("LearnerProfile")))
    {
        return Err(Error::new(
            "learner_context",
            "Expected a shared learner bundle",
        ));
    }
    crate::learner_model::audit(
        &source,
        &learner,
        &bundle_identity(&input.context)?,
        &bundle_identity(&input.learner_context)?,
        input.context.study.as_deref(),
    )
}
pub fn dispatch(operation: &str, request: Request) -> Result<Value> {
    if request.schema_version != VERSION {
        return Err(Error::new(
            "schema_version",
            "Only schema_version 1 is supported",
        ));
    }
    let result = match operation {
        "audit-learner-model" => audit_model(serde_json::from_value(request.data)?)?,
        "publish" => publish(serde_json::from_value(request.data)?)?,
        "save-attempt" => save_attempt(serde_json::from_value(request.data)?)?,
        "save-observation" => save_record(
            serde_json::from_value(request.data)?,
            "LearnerObservation",
            Some("observations"),
        )?,
        "save-strategy-application" => save_record(
            serde_json::from_value(request.data)?,
            "StrategyApplication",
            Some("strategy-applications"),
        )?,
        "inspect-state" => {
            let context: Context = serde_json::from_value(request.data)?;
            check_cli(&context)?;
            structural(&context)?;
            let state = inventory(&context)?;
            check_state(&state)?;
            crate::studies::selection(&state, context.study.as_deref())?;
            let next_sequence = state
                .records
                .iter()
                .filter_map(|r| r.metadata.get("event").and_then(|e| e["sequence"].as_u64()))
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| Error::new("sequence", "Sequence exhausted"))?;
            let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&state.records)?));
            let selected_records: Vec<_> = state
                .records
                .iter()
                .filter(|r| {
                    context
                        .study
                        .as_deref()
                        .is_none_or(|id| crate::studies::owner(r) == Some(id))
                })
                .cloned()
                .collect();
            let coverage: Vec<_> = selected_records.iter().filter(|r| r.metadata.get("type") == Some(&json!("Session"))).map(|r| json!({"session":r.id,"status":r.metadata.get("session_status"),"covered_objectives":r.metadata.get("covered_objectives"),"understanding_check":r.metadata.get("understanding_check")})).collect();
            let mut history = json!({"complete":true,"attempts":[],"assessments":[],"disputed_assessments":[],"assistance_updates":{}});
            let mut objectives = Vec::new();
            for record in &selected_records {
                let m = &record.metadata;
                match m.get("type").and_then(Value::as_str).unwrap_or("") {
                    "Attempt" => history["attempts"].as_array_mut().unwrap().push(m["event"].clone()),
                    "Assessment" => history["assessments"].as_array_mut().unwrap().push(m["judgment"].clone()),
                    "AssessmentDispute" if m.get("dispute_status") == Some(&json!("open")) => history["disputed_assessments"].as_array_mut().unwrap().push(m["assessment"].clone()),
                    "AssistanceEvent" => { history["assistance_updates"][m["attempt"].as_str().unwrap()] = m["assistance"].clone(); },
                    "Objective" => objectives.push(json!({"id":record.id,"version":m["contract_version"],"required_criteria":m["required_criteria"],"allowed_kinds":m["allowed_kinds"]})),
                    _ => (),
                }
            }
            let resolved =
                crate::rules::resolve_assessments(&serde_json::from_value(history.clone())?)?;
            let pending: Vec<_> = resolved
                .as_object()
                .unwrap()
                .iter()
                .filter(|(_, v)| v["state"] != "resolved")
                .map(|(id, _)| id)
                .collect();
            json!({"records":selected_records,"study":context.study,"coverage":coverage,"next_sequence":next_sequence,"history_digest":digest,"history":history,"objectives":objectives,"pending_assessments":pending,"resolved_assessments":resolved})
        }
        _ => return Err(Error::new("command", operation)),
    };
    Ok(json!({"schema_version":VERSION,"rule_version":crate::RULE_VERSION,"result":result}))
}

#[cfg(test)]
mod stream_tests {
    use super::*;

    #[test]
    fn review_history_requires_one_linear_chain_for_each_passage() {
        fn record(id: &str, meta: Value) -> Record {
            Record {
                id: id.into(),
                metadata: serde_json::from_value(meta).unwrap(),
                body: String::new(),
            }
        }
        let passage = record("/passages/a", json!({"type":"SourcePassage"}));
        let first = record(
            "/reviews/a",
            json!({"type":"PassageReview", "passage_ref":"/passages/a", "review_status":"blocked"}),
        );
        let second = record(
            "/reviews/b",
            json!({"type":"PassageReview", "passage_ref":"/passages/a", "supersedes":"/reviews/a", "review_status":"passage_checked"}),
        );
        let state = |records| State {
            complete: true,
            records,
        };
        assert!(
            library_reviews(&state(vec![passage.clone(), first.clone(), second.clone()])).is_ok()
        );
        let branch = record(
            "/reviews/c",
            json!({"type":"PassageReview", "passage_ref":"/passages/a", "supersedes":"/reviews/a"}),
        );
        assert!(library_reviews(&state(vec![
            passage.clone(),
            first.clone(),
            second.clone(),
            branch
        ]))
        .is_err());
        let cycle = record(
            "/reviews/a",
            json!({"type":"PassageReview", "passage_ref":"/passages/a", "supersedes":"/reviews/b"}),
        );
        assert!(library_reviews(&state(vec![passage.clone(), cycle, second])).is_err());
        let other = record("/passages/b", json!({"type":"SourcePassage"}));
        let mismatch = record(
            "/reviews/b",
            json!({"type":"PassageReview", "passage_ref":"/passages/b", "supersedes":"/reviews/a"}),
        );
        assert!(library_reviews(&state(vec![passage, other, first, mismatch])).is_err());
    }

    #[test]
    fn reject_truncated_failed_and_partial_operations() {
        let start = json!({"kind":"operation-start","schema_version":1,"stream_version":1});
        let payload = json!({"kind":"query-summary","scan_complete":true});
        let end = json!({"kind":"operation-complete","schema_version":1,"outcome":"ok","exit_code":0,
            "completeness":{"delivery":"complete","examination":"complete"}});
        let stream = |end: &Value| format!("{start}\n{payload}\n{end}\n");
        assert_eq!(ndjson(&stream(&end)).unwrap(), vec![payload.clone()]);
        assert!(ndjson(&format!("{start}\n{payload}\n")).is_err());
        assert!(ndjson("").is_err());
        for (field, value) in [("outcome", json!("error")), ("exit_code", json!(1))] {
            let mut broken = end.clone();
            broken[field] = value;
            assert!(ndjson(&stream(&broken)).is_err());
        }
        for dimension in ["delivery", "examination"] {
            let mut partial = end.clone();
            partial["completeness"][dimension] = json!("partial");
            assert!(ndjson(&stream(&partial)).is_err());
        }
    }
}
