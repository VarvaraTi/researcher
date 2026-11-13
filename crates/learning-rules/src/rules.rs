use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub nodes: Vec<String>,
    pub prerequisites: Vec<[String; 2]>,
}
pub fn check_prerequisites(graph: Graph) -> Result<Value> {
    let nodes: BTreeSet<_> = graph.nodes.iter().cloned().collect();
    if nodes.len() != graph.nodes.len() || nodes.iter().any(|n| n.is_empty()) {
        return Err(Error::new(
            "identity",
            "Node IDs must be unique and nonempty",
        ));
    }
    let mut edges: BTreeMap<String, BTreeSet<String>> =
        nodes.iter().map(|n| (n.clone(), BTreeSet::new())).collect();
    for [from, to] in graph.prerequisites {
        if !nodes.contains(&from) || !nodes.contains(&to) {
            return Err(Error::new("missing_reference", format!("{from} -> {to}")));
        }
        edges.get_mut(&from).unwrap().insert(to);
    }
    fn visit(
        n: &str,
        edges: &BTreeMap<String, BTreeSet<String>>,
        stack: &mut Vec<String>,
        done: &mut BTreeSet<String>,
    ) -> Result<()> {
        if let Some(start) = stack.iter().position(|x| x == n) {
            let mut path = stack[start..].to_vec();
            path.push(n.into());
            return Err(Error::new("prerequisite_cycle", path.join(" -> ")));
        }
        if done.contains(n) {
            return Ok(());
        }
        stack.push(n.into());
        for next in &edges[n] {
            visit(next, edges, stack, done)?;
        }
        stack.pop();
        done.insert(n.into());
        Ok(())
    }
    let mut done = BTreeSet::new();
    for n in nodes {
        visit(&n, &edges, &mut Vec::new(), &mut done)?;
    }
    Ok(json!({"valid": true}))
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Assistance {
    None,
    Hints,
    Solution,
    CheatSheet,
    Unknown,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub id: String,
    pub submission_id: String,
    pub exercise: String,
    pub session: String,
    pub sequence: u64,
    pub timestamp: String,
    pub assistance: Assistance,
    pub objective: String,
    pub evidence_kind: String,
    pub is_review: bool,
    /// False after feedback/rehearsal: immediate correction is practice, not retention.
    pub independent_eligible: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    Resolved,
    Disputed,
    Pending,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Correct,
    Partial,
    Incorrect,
    Unassessable,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assessment {
    pub id: String,
    pub attempt: String,
    pub assessment_status: AssessmentStatus,
    pub outcome: Option<Outcome>,
    pub criteria: BTreeMap<String, bool>,
    pub supersedes: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct History {
    pub complete: bool,
    pub attempts: Vec<Attempt>,
    pub assessments: Vec<Assessment>,
    #[serde(default)]
    pub disputed_assessments: BTreeSet<String>,
    #[serde(default)]
    pub assistance_updates: BTreeMap<String, Assistance>,
}
pub(crate) fn active(history: &History) -> Result<BTreeMap<String, Option<&Assessment>>> {
    if !history.complete {
        return Err(Error::new(
            "incomplete_history",
            "Complete scoped history is required",
        ));
    }
    let mut attempts = BTreeSet::new();
    let mut submissions = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    for a in &history.attempts {
        crate::schedule::utc(&a.timestamp)?;
        if a.sequence == 0
            || a.id.is_empty()
            || a.submission_id.is_empty()
            || !attempts.insert(a.id.clone())
            || !submissions.insert(&a.submission_id)
            || !sequences.insert(a.sequence)
        {
            return Err(Error::new(
                "identity",
                "Duplicate or empty attempt, submission, or sequence",
            ));
        }
    }
    let mut assessments = BTreeMap::new();
    for a in &history.assessments {
        if a.id.is_empty() || assessments.insert(a.id.clone(), a).is_some() {
            return Err(Error::new("identity", "Duplicate or empty assessment ID"));
        }
        if !attempts.contains(&a.attempt) {
            return Err(Error::new("missing_attempt", a.attempt.clone()));
        }
        if a.assessment_status == AssessmentStatus::Resolved && a.outcome.is_none() {
            return Err(Error::new("missing_outcome", a.id.clone()));
        }
    }
    let mut replaced = BTreeSet::new();
    for id in &history.disputed_assessments {
        if !assessments.contains_key(id) {
            return Err(Error::new("missing_assessment", id));
        }
    }
    for id in history.assistance_updates.keys() {
        if !attempts.contains(id) {
            return Err(Error::new("missing_attempt", id));
        }
    }
    for a in assessments.values() {
        let mut seen = BTreeSet::new();
        let mut cursor = *a;
        while let Some(parent) = &cursor.supersedes {
            if !seen.insert(cursor.id.clone()) {
                return Err(Error::new("supersession_cycle", a.id.clone()));
            }
            let previous = assessments
                .get(parent)
                .ok_or_else(|| Error::new("missing_assessment", parent.clone()))?;
            if previous.attempt != a.attempt {
                return Err(Error::new("cross_attempt_supersession", a.id.clone()));
            }
            replaced.insert(parent.clone());
            cursor = previous;
        }
    }
    let mut result: BTreeMap<_, _> = attempts.into_iter().map(|a| (a, None)).collect();
    for a in assessments.values().filter(|a| !replaced.contains(&a.id)) {
        let slot = result.get_mut(&a.attempt).unwrap();
        if slot.is_some() {
            return Err(Error::new("assessment_branch", a.attempt.clone()));
        }
        *slot = Some(*a);
    }
    Ok(result)
}
pub fn resolve_assessments(history: &History) -> Result<Value> {
    let result: BTreeMap<_, _> = active(history)?
        .into_iter()
        .map(|(id, a)| {
            let state = match a {
                None => json!({"state": "pending", "assessment": null}),
                Some(a) if history.disputed_assessments.contains(&a.id) => {
                    json!({"state":"disputed", "assessment":a.id,"outcome":a.outcome})
                }
                Some(a) => {
                    json!({"state": a.assessment_status, "assessment": a.id, "outcome": a.outcome})
                }
            };
            (id, state)
        })
        .collect();
    Ok(json!(result))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    pub value: Value,
    pub source: String,
    pub version: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub defaults: BTreeMap<String, Setting>,
    pub global: BTreeMap<String, Setting>,
    pub course: BTreeMap<String, Setting>,
    pub session: BTreeMap<String, Setting>,
}
pub fn resolve_settings(input: Settings) -> Result<Value> {
    let mut result = BTreeMap::new();
    for (scope, settings) in [
        ("default", input.defaults),
        ("global", input.global),
        ("course", input.course),
        ("session", input.session),
    ] {
        for (key, setting) in settings {
            if setting.source.is_empty() || setting.version.is_empty() || setting.value.is_null() {
                return Err(Error::new("setting_provenance", key));
            }
            result.insert(key, json!({"value": setting.value, "scope": scope, "source": setting.source, "version": setting.version}));
        }
    }
    Ok(json!(result))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Comparison {
    Exact {
        expected: String,
    },
    Trimmed {
        expected: String,
        ascii_case_insensitive: bool,
    },
    OptionSet {
        expected: BTreeSet<String>,
        allowed: BTreeSet<String>,
    },
    Numeric {
        expected: f64,
        absolute_tolerance: f64,
    },
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grade {
    pub rule_version: String,
    pub comparison: Comparison,
    pub answer: Value,
}
pub fn grade_answer(input: Grade) -> Result<Value> {
    if input.rule_version.is_empty() {
        return Err(Error::new(
            "rule_version",
            "A retained comparison version is required",
        ));
    }
    let correct = match input.comparison {
        Comparison::Exact { expected } => {
            input
                .answer
                .as_str()
                .ok_or_else(|| Error::new("answer_type", "Expected a string"))?
                == expected
        }
        Comparison::Trimmed {
            expected,
            ascii_case_insensitive,
        } => {
            let answer = input
                .answer
                .as_str()
                .ok_or_else(|| Error::new("answer_type", "Expected a string"))?
                .trim();
            if ascii_case_insensitive {
                answer.eq_ignore_ascii_case(expected.trim())
            } else {
                answer == expected.trim()
            }
        }
        Comparison::OptionSet { expected, allowed } => {
            if !expected.is_subset(&allowed) || expected.is_empty() {
                return Err(Error::new(
                    "answer_key",
                    "Expected options must be nonempty and allowed",
                ));
            }
            let answer: Vec<String> = serde_json::from_value(input.answer)?;
            let chosen: BTreeSet<_> = answer.iter().cloned().collect();
            if chosen.len() != answer.len() || !chosen.is_subset(&allowed) {
                return Err(Error::new("option_id", "Unknown or repeated option ID"));
            }
            chosen == expected
        }
        Comparison::Numeric {
            expected,
            absolute_tolerance,
        } => {
            let answer = input
                .answer
                .as_f64()
                .ok_or_else(|| Error::new("answer_type", "Expected a JSON number"))?;
            if !expected.is_finite() || !absolute_tolerance.is_finite() || absolute_tolerance < 0.0
            {
                return Err(Error::new(
                    "tolerance",
                    "Expected finite nonnegative tolerance and finite key",
                ));
            }
            (answer - expected).abs() <= absolute_tolerance
        }
    };
    Ok(
        json!({"outcome": if correct { "correct" } else { "incorrect" }, "comparison_version": input.rule_version}),
    )
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceContract {
    pub id: String,
    pub version: String,
    pub required_criteria: BTreeSet<String>,
    pub allowed_kinds: BTreeSet<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub history: History,
    pub objectives: Vec<EvidenceContract>,
    pub required_items: BTreeSet<String>,
    pub completed_items: BTreeSet<String>,
    pub skipped_items: BTreeSet<String>,
}
pub fn compute_progress(input: Progress) -> Result<Value> {
    let active = active(&input.history)?;
    if !input.completed_items.is_disjoint(&input.skipped_items) {
        return Err(Error::new(
            "item_state",
            "An item cannot be both completed and skipped",
        ));
    }
    let mut states = BTreeMap::new();
    for objective in input.objectives {
        if objective.required_criteria.is_empty()
            || objective.allowed_kinds.is_empty()
            || objective.version.is_empty()
            || states.contains_key(&objective.id)
        {
            return Err(Error::new("evidence_contract", objective.id));
        }
        let mut history: Vec<_> = input
            .history
            .attempts
            .iter()
            .filter(|a| a.objective == objective.id)
            .collect();
        history.sort_by_key(|a| (a.sequence, &a.id));
        let mut state = "not_started";
        let mut successes = Vec::new();
        let mut unresolved = Vec::new();
        for attempt in history {
            if state == "not_started" {
                state = "practicing";
            }
            match active[&attempt.id] {
                Some(a)
                    if a.assessment_status == AssessmentStatus::Resolved
                        && !input.history.disputed_assessments.contains(&a.id) =>
                {
                    let assistance = input
                        .history
                        .assistance_updates
                        .get(&attempt.id)
                        .unwrap_or(&attempt.assistance);
                    if a.outcome == Some(Outcome::Correct)
                        && *assistance == Assistance::None
                        && attempt.independent_eligible
                        && objective.allowed_kinds.contains(&attempt.evidence_kind)
                        && objective
                            .required_criteria
                            .iter()
                            .all(|c| a.criteria.get(c) == Some(&true))
                    {
                        state = "demonstrated";
                        successes.push(attempt.id.clone());
                    } else if attempt.is_review
                        && matches!(a.outcome, Some(Outcome::Incorrect | Outcome::Partial))
                    {
                        state = "practicing";
                    } else if a.outcome == Some(Outcome::Unassessable) {
                        unresolved.push(attempt.id.clone());
                    }
                }
                _ => unresolved.push(attempt.id.clone()),
            }
        }
        states.insert(objective.id, json!({"state": state, "historical_successes": successes, "unresolved": unresolved, "contract_version": objective.version}));
    }
    for a in &input.history.attempts {
        if !states.contains_key(&a.objective) {
            return Err(Error::new("missing_objective", a.objective.clone()));
        }
    }
    let covered: BTreeSet<_> = input
        .completed_items
        .union(&input.skipped_items)
        .cloned()
        .collect();
    Ok(
        json!({"activity": {"attempts": input.history.attempts.len(), "completed_items": input.completed_items, "skipped_items": input.skipped_items}, "lesson_complete": input.required_items.is_subset(&covered), "objectives": states}),
    )
}

/// Normalized records retain frontmatter and exact body; all active publications use this gate.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: String,
    pub metadata: BTreeMap<String, Value>,
    pub body: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub complete: bool,
    pub records: Vec<Record>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyUse {
    pub id: String,
    pub origin_id: String,
    pub strategy: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportReview {
    pub strategy: String,
    pub applications: Vec<StrategyUse>,
    pub reviewed_origins: BTreeSet<String>,
    pub threshold: u64,
    pub obstructive: bool,
    pub policy_version: String,
}
pub fn review_support(input: SupportReview) -> Result<Value> {
    if input.threshold == 0 || input.strategy.is_empty() || input.policy_version.is_empty() {
        return Err(Error::new(
            "support_policy",
            "Require a positive threshold, strategy and policy version",
        ));
    }
    let mut origins = BTreeMap::new();
    for application in input
        .applications
        .iter()
        .filter(|a| a.strategy == input.strategy)
    {
        if application.id.is_empty() || application.origin_id.is_empty() {
            return Err(Error::new(
                "identity",
                "Missing strategy application identity",
            ));
        }
        if let Some(previous) = origins.insert(&application.origin_id, &application.id) {
            if previous != &application.id {
                return Err(Error::new("duplicate_origin", &application.origin_id));
            }
        }
    }
    let relevant: Vec<_> = origins
        .keys()
        .filter(|origin| !input.reviewed_origins.contains(origin.as_str()))
        .collect();
    Ok(
        json!({"strategy":input.strategy,"applications_since_review":relevant.len(),"review_due":input.obstructive || relevant.len() as u64 >= input.threshold,"policy_version":input.policy_version,"reason":if input.obstructive {"obstructive"} else {"application_count"}}),
    )
}

pub fn check_state(state: &State) -> Result<Value> {
    if !state.complete {
        return Err(Error::new(
            "incomplete_history",
            "Publication requires full selected-bundle inventory",
        ));
    }
    let records: BTreeMap<_, _> = state.records.iter().map(|r| (r.id.as_str(), r)).collect();
    if records.len() != state.records.len() {
        return Err(Error::new("identity", "Duplicate record IDs"));
    }
    crate::studies::check(state)?;
    crate::learner_model::check(state)?;
    let mut graph = Graph {
        nodes: Vec::new(),
        prerequisites: Vec::new(),
    };
    let mut history = History {
        complete: true,
        attempts: Vec::new(),
        assessments: Vec::new(),
        disputed_assessments: BTreeSet::new(),
        assistance_updates: BTreeMap::new(),
    };
    let mut origins = BTreeSet::new();
    for record in &state.records {
        let m = &record.metadata;
        let string = |k: &str| -> Result<String> {
            m.get(k)
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .ok_or_else(|| Error::new("required_field", format!("{}: {k}", record.id)))
        };
        let reference = |k: &str, typ: &str| -> Result<&Record> {
            let id = string(k)?;
            let target = records.get(id.as_str()).ok_or_else(|| {
                Error::new("missing_reference", format!("{}: {k} -> {id}", record.id))
            })?;
            if target.metadata.get("type").and_then(Value::as_str) != Some(typ) {
                return Err(Error::new(
                    "reference_type",
                    format!("{}: {k} requires {typ}", record.id),
                ));
            }
            Ok(target)
        };
        let typ = string("type")?;
        if m.get("schema_version") != Some(&json!(1)) {
            return Err(Error::new("schema_version", record.id.clone()));
        }
        if ["Topic", "Objective"].contains(&typ.as_str()) {
            graph.nodes.push(record.id.clone());
            if let Some(edges) = m.get("prerequisites") {
                for to in serde_json::from_value::<Vec<String>>(edges.clone())? {
                    graph.prerequisites.push([record.id.clone(), to]);
                }
            }
        }
        if typ == "Attempt" {
            let a: Attempt = serde_json::from_value(
                m.get("event")
                    .cloned()
                    .ok_or_else(|| Error::new("attempt_event", &record.id))?,
            )?;
            let expected_path = if let Some(study) = m.get("study").and_then(Value::as_str) {
                format!(
                    "{}/attempts/{}",
                    crate::studies::root(study)?,
                    a.submission_id
                )
            } else {
                format!("/attempts/{}", a.submission_id)
            };
            if a.id != record.id || record.id != expected_path {
                return Err(Error::new("submission_path", record.id.clone()));
            }
            for (field, expected_type) in [
                ("exercise", "Exercise"),
                ("session", "Session"),
                ("objective", "Objective"),
            ] {
                let target = reference(field, expected_type)?;
                let event_ref = match field {
                    "exercise" => &a.exercise,
                    "session" => &a.session,
                    _ => &a.objective,
                };
                if event_ref != &target.id {
                    return Err(Error::new("event_reference", record.id.clone()));
                }
            }
            let exercise = reference("exercise", "Exercise")?;
            if exercise.metadata.get("version") != m.get("exercise_version") {
                return Err(Error::new("historical_version", record.id.clone()));
            }
            history.attempts.push(a);
        }
        if typ == "Assessment" {
            let a: Assessment = serde_json::from_value(
                m.get("judgment")
                    .cloned()
                    .ok_or_else(|| Error::new("assessment_judgment", &record.id))?,
            )?;
            if a.id != record.id || a.attempt != string("attempt")? {
                return Err(Error::new("assessment_identity", record.id.clone()));
            }
            reference("attempt", "Attempt")?;
            history.assessments.push(a);
        }
        if typ == "AssessmentDispute" && m.get("dispute_status") == Some(&json!("open")) {
            history.disputed_assessments.insert(string("assessment")?);
        }
        if typ == "AssistanceEvent" {
            let id = string("attempt")?;
            let assistance: Assistance =
                serde_json::from_value(m.get("assistance").cloned().unwrap_or(Value::Null))?;
            if history
                .assistance_updates
                .insert(id.clone(), assistance)
                .is_some()
            {
                return Err(Error::new(
                    "assistance_conflict",
                    format!("Multiple clarifications for {id}; reconcile explicitly"),
                ));
            }
        }
        if typ == "LearningPlan" && m.get("plan_status") == Some(&json!("accepted")) {
            string("acceptance_evidence")?;
        }
        if (typ == "Course" || typ == "Study") && m.contains_key("accepted_plan") {
            let plan = reference("accepted_plan", "LearningPlan")?;
            if plan.metadata.get("plan_status") != Some(&json!("accepted")) {
                return Err(Error::new("plan_acceptance", record.id.clone()));
            }
        }
        if typ == "LearnerPreference"
            && m.get("preference_status") == Some(&json!("confirmed"))
            && m.get("origin") == Some(&json!("inferred"))
        {
            string("confirmation_evidence")?;
        }
        if typ == "Lesson" && m.get("readiness") == Some(&json!("ready")) {
            string("source_review")?;
            reference("cheat_sheet", "CheatSheet")?;
            if m.get("source_supported") != Some(&json!(true)) {
                return Err(Error::new("unsupported_content", record.id.clone()));
            }
            let sources: Vec<String> =
                serde_json::from_value(m.get("theory_refs").cloned().unwrap_or(Value::Null))?;
            if sources.is_empty() {
                return Err(Error::new("unsupported_content", record.id.clone()));
            }
            for id in sources {
                let source = records
                    .get(id.as_str())
                    .ok_or_else(|| Error::new("missing_reference", &id))?;
                if source.metadata.get("type") != Some(&json!("Source"))
                    || source.metadata.get("access_status") != Some(&json!("available"))
                {
                    return Err(Error::new("unsupported_content", id));
                }
            }
        }
        if typ == "CheatSheet" {
            let sections: BTreeSet<String> =
                serde_json::from_value(m.get("sections").cloned().unwrap_or(Value::Null))?;
            let revealed: BTreeSet<String> =
                serde_json::from_value(m.get("revealed_sections").cloned().unwrap_or(json!([])))?;
            if !revealed.is_subset(&sections) {
                return Err(Error::new("sheet_sections", record.id.clone()));
            }
        }
        if typ == "Exercise" {
            string("rubric")?;
            string("version")?;
        }
        if typ == "LearnerObservation" || typ == "StrategyApplication" {
            let origin = string("origin_id")?;
            if !origins.insert((typ, origin)) {
                return Err(Error::new("duplicate_origin", record.id.clone()));
            }
        }
    }
    check_prerequisites(graph)?;
    resolve_assessments(&history)?;
    Ok(json!({"valid": true, "records": state.records.len()}))
}
