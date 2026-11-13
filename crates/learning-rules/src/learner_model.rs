//! Reconcile saved teaching events with explicit learner-model review receipts.
//! Receipts track review, never infer preferences or traits from an event.
use crate::{
    rules::{Record, State},
    Error, Result,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn digest(record: &Record) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(record)?)))
}
pub fn receipt_id(source: &str, digest: &str) -> String {
    format!(
        "/model-reviews/{:x}",
        Sha256::digest(format!("{source}\n{digest}").as_bytes())
    )
}
pub fn check(state: &State) -> Result<()> {
    for r in &state.records {
        if r.metadata.get("type") != Some(&json!("LearnerModelReview")) {
            continue;
        }
        let required = |key: &str| {
            r.metadata
                .get(key)
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| Error::new("model_review", format!("{} needs {key}", r.id)))
        };
        let source = required("source_id")?;
        let digest = required("source_digest")?;
        if r.id != receipt_id(source, digest) {
            return Err(Error::new(
                "model_review",
                "Receipt identity must match source and digest",
            ));
        }
        required("reason")?;
        required("reviewed_at")?;
        let assertions: Vec<String> =
            serde_json::from_value(r.metadata.get("assertions").cloned().unwrap_or(json!([])))?;
        match required("decision")? {
            "recorded" if !assertions.is_empty() => (),
            "not_needed" if assertions.is_empty() => (),
            _ => {
                return Err(Error::new(
                    "model_review",
                    "Recorded requires assertions; not_needed requires no assertions",
                ))
            }
        }
        for id in assertions {
            let target = state
                .records
                .iter()
                .find(|x| x.id == id)
                .ok_or_else(|| Error::new("model_review", format!("Missing assertion {id}")))?;
            if !matches!(
                target.metadata.get("type").and_then(Value::as_str),
                Some(
                    "LearnerGoal"
                        | "LearnerPreference"
                        | "LearnerObservation"
                        | "LearnerHypothesis"
                )
            ) {
                return Err(Error::new(
                    "model_review",
                    "Receipt target must be a learner assertion",
                ));
            }
        }
    }
    Ok(())
}
pub fn audit(
    source: &State,
    learner: &State,
    bundle: &str,
    learner_bundle: &str,
    study: Option<&str>,
) -> Result<Value> {
    if !source.complete || !learner.complete {
        return Err(Error::new(
            "incomplete_history",
            "Model audit requires complete inventories",
        ));
    }
    check(learner)?;
    let mut pending = Vec::new();
    let mut reviewed = Vec::new();
    for event in &source.records {
        if study.is_some_and(|id| crate::studies::owner(event) != Some(id)) {
            continue;
        }
        if !matches!(
            event.metadata.get("type").and_then(Value::as_str),
            Some(
                "Course"
                    | "Study"
                    | "CourseSettings"
                    | "StudySettings"
                    | "Note"
                    | "Question"
                    | "Session"
                    | "Assessment"
                    | "AssessmentDispute"
                    | "ReadinessChoice"
                    | "SkipEvent"
                    | "AssistanceEvent"
                    | "StrategyApplication"
            )
        ) {
            continue;
        }
        let source_id = format!("{bundle}:{}", event.id);
        let source_digest = digest(event)?;
        let review_id = receipt_id(&source_id, &source_digest);
        if let Some(receipt) = learner.records.iter().find(|r| {
            r.id == review_id && r.metadata.get("type") == Some(&json!("LearnerModelReview"))
        }) {
            reviewed.push(json!({"source_id":source_id,"review":receipt.id,"decision":receipt.metadata.get("decision")}));
        } else {
            pending.push(json!({"source_id":source_id,"source_digest":source_digest,"review_id":review_id,"record":event}));
        }
    }
    let scope = study.map(|id| format!("{bundle}:{id}"));
    let mut global = BTreeMap::new();
    let mut scoped = BTreeMap::new();
    let mut assertions = Vec::new();
    for r in &learner.records {
        let m = &r.metadata;
        let s = m.get("scope").and_then(Value::as_str).unwrap_or("");
        if s != "global" && s != bundle && scope.as_deref() != Some(s) {
            continue;
        }
        if !matches!(
            m.get("type").and_then(Value::as_str),
            Some("LearnerGoal" | "LearnerPreference" | "LearnerObservation" | "LearnerHypothesis")
        ) {
            continue;
        }
        assertions.push(r);
        if m.get("type") == Some(&json!("LearnerPreference"))
            && m.get("preference_status") == Some(&json!("confirmed"))
        {
            let key = m
                .get("setting")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::new("preference", "Missing setting"))?;
            let target = if s == "global" {
                &mut global
            } else {
                &mut scoped
            };
            let entry = json!({"value":m["value"]["data"],"source":format!("{learner_bundle}:{}",r.id),"version":m.get("updated_at")});
            if target.insert(key.to_owned(), entry).is_some() {
                return Err(Error::new(
                    "preference_conflict",
                    format!("Multiple confirmed preferences for {key} in the selected scope"),
                ));
            }
        }
    }
    Ok(
        json!({"complete":true,"source_bundle":bundle,"study":study,"global_settings":global,"scoped_settings":scoped,"assertions":assertions,"pending":pending,"reviewed":reviewed,"review_complete":pending.is_empty()}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pref(id: &str, scope: &str, status: &str, value: &str) -> Record {
        serde_json::from_value(json!({"id":id,"metadata":{"type":"LearnerPreference","setting":"chat_language","scope":scope,"preference_status":status,"value":{"data":value},"updated_at":"2026-11-01T12:00:00Z"},"body":""})).unwrap()
    }
    #[test]
    fn only_confirmed_applicable_preferences_become_settings() {
        let source = State {
            complete: true,
            records: vec![],
        };
        let mut learner = State {
            complete: true,
            records: vec![
                pref("/p1", "global", "confirmed", "ru"),
                pref("/p2", "other", "confirmed", "de"),
                pref("/p3", "global", "proposed", "fr"),
                pref("/p4", "global", "retired", "es"),
                pref("/p5", "course", "confirmed", "en"),
            ],
        };
        let result = audit(&source, &learner, "course", "learner", None).unwrap();
        assert_eq!(result["global_settings"]["chat_language"]["value"], "ru");
        assert_eq!(result["scoped_settings"]["chat_language"]["value"], "en");
        learner
            .records
            .push(pref("/p6", "global", "confirmed", "uk"));
        assert_eq!(
            audit(&source, &learner, "course", "learner", None)
                .unwrap_err()
                .code,
            "preference_conflict"
        );
    }
}
