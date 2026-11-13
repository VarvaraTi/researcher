//! Article study boundaries. Legacy single-course bundles keep their existing contract.
use crate::{
    rules::{Record, State},
    Error, Result,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn root(study: &str) -> Result<&str> {
    let prefix = study
        .strip_suffix("/study")
        .ok_or_else(|| Error::new("study_id", study))?;
    let slug = prefix
        .strip_prefix("/studies/")
        .ok_or_else(|| Error::new("study_id", study))?;
    if slug.is_empty()
        || !slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(Error::new("study_id", study));
    }
    Ok(prefix)
}
pub fn owner(record: &Record) -> Option<&str> {
    if record.metadata.get("type").and_then(Value::as_str) == Some("Study") {
        Some(&record.id)
    } else {
        record.metadata.get("study").and_then(Value::as_str)
    }
}
pub fn selection(state: &State, selected: Option<&str>) -> Result<()> {
    let has_studies = state
        .records
        .iter()
        .any(|r| r.metadata.get("type") == Some(&json!("Study")));
    if has_studies && selected.is_none() {
        return Err(Error::new(
            "study_required",
            "Select one study before reading or publishing article learning state",
        ));
    }
    if let Some(id) = selected {
        root(id)?;
        if !state
            .records
            .iter()
            .any(|r| r.id == id && r.metadata.get("type") == Some(&json!("Study")))
        {
            return Err(Error::new("study_selection", id));
        }
    }
    Ok(())
}
fn strings(value: &Value) -> Vec<&str> {
    match value {
        Value::String(s) => vec![s],
        Value::Array(xs) => xs.iter().flat_map(strings).collect(),
        _ => vec![],
    }
}
pub fn check(state: &State) -> Result<()> {
    let scoped = state.records.iter().any(|r| {
        r.id.starts_with("/studies/")
            || r.metadata.contains_key("study")
            || r.metadata.get("type") == Some(&json!("Study"))
    });
    if !scoped {
        return Ok(());
    }
    let records: BTreeMap<_, _> = state.records.iter().map(|r| (r.id.as_str(), r)).collect();
    for r in &state.records {
        let study = owner(r).ok_or_else(|| Error::new("study_required", &r.id))?;
        let prefix = root(study)?;
        let target = records
            .get(study)
            .ok_or_else(|| Error::new("study_reference", study))?;
        if target.metadata.get("type") != Some(&json!("Study"))
            || !r.id.starts_with(&format!("{prefix}/"))
        {
            return Err(Error::new("study_boundary", &r.id));
        }
        for key in ["title", "description"] {
            if r.metadata
                .get(key)
                .and_then(Value::as_str)
                .is_none_or(|s| s.trim().is_empty())
            {
                return Err(Error::new("study_content", format!("{} needs {key}", r.id)));
            }
        }
        let typ = r.metadata.get("type").and_then(Value::as_str).unwrap_or("");
        if typ == "Study" {
            for field in ["goal", "language", "kind"] {
                if r.metadata
                    .get(field)
                    .and_then(Value::as_str)
                    .is_none_or(|s| s.trim().is_empty())
                {
                    return Err(Error::new("study_content", field));
                }
            }
            let domains = r
                .metadata
                .get("domains")
                .and_then(Value::as_array)
                .ok_or_else(|| Error::new("study_domains", &r.id))?;
            if domains.is_empty()
                || domains.iter().any(|v| {
                    v.as_str().is_none_or(|s| {
                        s.is_empty()
                            || !s
                                .chars()
                                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    })
                })
            {
                return Err(Error::new("study_domains", &r.id));
            }
        }
        if matches!(typ, "Topic" | "Source" | "Lesson" | "CheatSheet") && r.body.trim().is_empty() {
            return Err(Error::new(
                "study_content",
                format!("{} needs substantive body text", r.id),
            ));
        }
        // Learning ownership edges stay local. related_topics is the explicit cross-study knowledge edge.
        for key in [
            "accepted_plan",
            "topic",
            "prerequisites",
            "objectives",
            "source_refs",
            "theory_refs",
            "exercise_refs",
            "cheat_sheet",
            "topics",
            "lessons",
            "personal_additions",
            "current_lesson",
            "exercise",
            "session",
            "objective",
            "attempt",
            "assessment",
            "related_records",
            "policy",
            "strategy",
            "strategy_applications",
            "covered_objectives",
            "references",
        ] {
            if let Some(value) = r.metadata.get(key) {
                for id in strings(value) {
                    let linked = records.get(id).ok_or_else(|| {
                        Error::new("study_reference", format!("{}: {key} -> {id}", r.id))
                    })?;
                    if owner(linked) != Some(study) {
                        return Err(Error::new(
                            "study_boundary",
                            format!("{}: {key} -> {id}", r.id),
                        ));
                    }
                }
            }
        }
        if let Some(value) = r.metadata.get("related_topics") {
            for id in strings(value) {
                if records
                    .get(id)
                    .is_none_or(|r| r.metadata.get("type") != Some(&json!("Topic")))
                {
                    return Err(Error::new("study_reference", id));
                }
            }
        }
        if typ == "Session" && r.metadata.get("session_status") == Some(&json!("completed")) {
            if r.body.trim().is_empty()
                || r.metadata
                    .get("covered_objectives")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
            {
                return Err(Error::new("session_summary", &r.id));
            }
            if !matches!(
                r.metadata
                    .get("understanding_check")
                    .and_then(Value::as_str),
                Some("not_performed" | "performed")
            ) {
                return Err(Error::new("understanding_check", &r.id));
            }
            if r.metadata.get("understanding_check") == Some(&json!("performed"))
                && !state.records.iter().any(|a| {
                    a.metadata.get("type") == Some(&json!("Attempt"))
                        && a.metadata.get("session") == Some(&json!(r.id))
                })
            {
                return Err(Error::new(
                    "understanding_check",
                    "Performed check requires a retained answer",
                ));
            }
        }
    }
    Ok(())
}
