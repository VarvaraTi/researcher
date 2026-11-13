use crate::{
    rules::{active, AssessmentStatus, Assistance, EvidenceContract, History, Outcome},
    Error, Result,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub fn utc(value: &str) -> Result<DateTime<Utc>> {
    let time =
        DateTime::parse_from_rfc3339(value).map_err(|e| Error::new("timestamp", e.to_string()))?;
    if time.offset().local_minus_utc() != 0 {
        return Err(Error::new(
            "timestamp",
            "Authoritative timestamps must be UTC",
        ));
    }
    Ok(time.with_timezone(&Utc))
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Calendar {
    pub version: String,
    pub effective_at: String,
    pub timezone: String,
    /// ISO weekdays: Monday=1, Sunday=7.
    pub study_days: BTreeSet<u32>,
}
impl Calendar {
    fn zone(&self) -> Result<Tz> {
        self.timezone
            .parse()
            .map_err(|_| Error::new("timezone", &self.timezone))
    }
    fn next_study(&self, date: NaiveDate) -> Result<NaiveDate> {
        for offset in 0..7 {
            let candidate = date
                .checked_add_signed(Duration::days(offset))
                .ok_or_else(|| Error::new("date_range", date.to_string()))?;
            if self
                .study_days
                .contains(&candidate.weekday().number_from_monday())
            {
                return Ok(candidate);
            }
        }
        Err(Error::new(
            "study_days",
            "Select at least one ISO weekday (1..7)",
        ))
    }
    fn due(&self, at: DateTime<Utc>, days: i64) -> Result<NaiveDate> {
        let date = at
            .with_timezone(&self.zone()?)
            .date_naive()
            .checked_add_signed(Duration::days(days))
            .ok_or_else(|| Error::new("date_range", at.to_string()))?;
        self.next_study(date)
    }
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub id: String,
    pub seconds: u64,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub history: History,
    pub objectives: Vec<EvidenceContract>,
    pub calendars: Vec<Calendar>,
    pub now: String,
    pub policy_version: String,
    pub budget_seconds: u64,
    pub review_percent: u64,
    pub review_estimates: BTreeMap<String, u64>,
    pub new_work: Vec<Work>,
}
#[derive(Debug, Serialize)]
struct Review {
    stage: usize,
    evidence_established: bool,
    original_due_date: NaiveDate,
    due_date: NaiveDate,
    last_event: String,
    reason: &'static str,
    calendar_version: String,
    #[serde(skip)]
    anchor: DateTime<Utc>,
    #[serde(skip)]
    interval: i64,
}
pub fn compute(mut input: Input) -> Result<Value> {
    if input.policy_version != "review-v1" {
        return Err(Error::new(
            "policy_version",
            "Only agreed review-v1 is implemented",
        ));
    }
    if input.review_percent > 100 || input.budget_seconds == 0 || input.budget_seconds > 86400 {
        return Err(Error::new(
            "budget",
            "Use 1..86400 seconds and 0..100 review percent",
        ));
    }
    let now = utc(&input.now)?;
    input
        .calendars
        .sort_by(|a, b| a.effective_at.cmp(&b.effective_at));
    let mut effective = BTreeSet::new();
    let mut versions = BTreeSet::new();
    for calendar in &input.calendars {
        calendar.zone()?;
        if calendar.version.is_empty()
            || !versions.insert(&calendar.version)
            || !effective.insert(utc(&calendar.effective_at)?)
            || calendar.study_days.is_empty()
            || calendar.study_days.iter().any(|d| !(1..=7).contains(d))
        {
            return Err(Error::new(
                "calendar",
                "Require unique versions/effective instants and nonempty ISO weekdays",
            ));
        }
    }
    input
        .calendars
        .sort_by_key(|c| utc(&c.effective_at).unwrap());
    let calendar_at = |at| -> Result<&Calendar> {
        input
            .calendars
            .iter()
            .rev()
            .find(|c| utc(&c.effective_at).unwrap() <= at)
            .ok_or_else(|| Error::new("calendar", "No effective calendar for event/current time"))
    };
    let current = calendar_at(now)?;
    let today = now.with_timezone(&current.zone()?).date_naive();
    let judgments = active(&input.history)?;
    let mut contracts = BTreeMap::new();
    for c in &input.objectives {
        if c.version.is_empty()
            || c.required_criteria.is_empty()
            || c.allowed_kinds.is_empty()
            || contracts.insert(&c.id, c).is_some()
        {
            return Err(Error::new("evidence_contract", &c.id));
        }
    }
    let mut ordered: Vec<_> = input.history.attempts.iter().collect();
    ordered.sort_by_key(|a| (a.sequence, &a.id));
    let mut reviews: BTreeMap<String, Review> = BTreeMap::new();
    let mut advanced = BTreeSet::new();
    let mut lapsed = BTreeSet::new();
    let intervals = [1, 3, 7, 14];
    let mut last_time = None;
    for attempt in &ordered {
        let at = utc(&attempt.timestamp)?;
        if at > now || last_time.is_some_and(|last| at < last) {
            return Err(Error::new(
                "event_order",
                "Attempt times must be nondecreasing in recorded sequence and no later than now",
            ));
        }
        last_time = Some(at);
        let contract = contracts
            .get(&attempt.objective)
            .ok_or_else(|| Error::new("missing_objective", &attempt.objective))?;
        let calendar = calendar_at(at)?;
        let Some(assessment) = judgments[&attempt.id] else {
            continue;
        };
        if assessment.assessment_status != AssessmentStatus::Resolved
            || input.history.disputed_assessments.contains(&assessment.id)
            || assessment.outcome == Some(Outcome::Unassessable)
        {
            continue;
        }
        let session_key = (attempt.objective.clone(), attempt.session.clone());
        let assistance = input
            .history
            .assistance_updates
            .get(&attempt.id)
            .unwrap_or(&attempt.assistance);
        let lapse = matches!(
            assessment.outcome,
            Some(Outcome::Incorrect | Outcome::Partial)
        ) || matches!(
            assistance,
            Assistance::Hints | Assistance::Solution | Assistance::CheatSheet
        );
        let qualifies = assessment.outcome == Some(Outcome::Correct)
            && *assistance == Assistance::None
            && attempt.independent_eligible
            && contract.allowed_kinds.contains(&attempt.evidence_kind)
            && contract
                .required_criteria
                .iter()
                .all(|c| assessment.criteria.get(c) == Some(&true));
        if lapse {
            lapsed.insert(session_key.clone());
            if let Some(review) = reviews.get_mut(&attempt.objective) {
                let due = calendar.due(at, 1)?;
                review.original_due_date = due;
                review.due_date = due;
                review.last_event = attempt.id.clone();
                review.reason = "lapse_next_study_day_retain_stage";
                review.anchor = at;
                review.interval = 1;
                review.calendar_version = calendar.version.clone();
            } else {
                let due = calendar.due(at, 1)?;
                reviews.insert(
                    attempt.objective.clone(),
                    Review {
                        stage: 0,
                        evidence_established: false,
                        original_due_date: due,
                        due_date: due,
                        last_event: attempt.id.clone(),
                        reason: "practice_follow_up",
                        calendar_version: calendar.version.clone(),
                        anchor: at,
                        interval: 1,
                    },
                );
            }
        } else if qualifies && !lapsed.contains(&session_key) && !advanced.contains(&session_key) {
            if let Some(review) = reviews.get_mut(&attempt.objective) {
                if review.evidence_established && !attempt.is_review {
                    continue;
                }
                if review.evidence_established {
                    review.stage = (review.stage + 1).min(3);
                }
                review.evidence_established = true;
                let interval = intervals[review.stage];
                let due = calendar.due(at, interval)?;
                review.original_due_date = due;
                review.due_date = due;
                review.last_event = attempt.id.clone();
                review.reason = "qualifying_review";
                review.anchor = at;
                review.interval = interval;
                review.calendar_version = calendar.version.clone();
            } else {
                let due = calendar.due(at, 1)?;
                reviews.insert(
                    attempt.objective.clone(),
                    Review {
                        stage: 0,
                        evidence_established: true,
                        original_due_date: due,
                        due_date: due,
                        last_event: attempt.id.clone(),
                        reason: "entered_review",
                        calendar_version: calendar.version.clone(),
                        anchor: at,
                        interval: 1,
                    },
                );
            }
            advanced.insert(session_key);
        }
    }
    // Apply calendar revisions in order: only dates still in the future at the change
    // are recalculated. The originally assigned due date remains available for audit.
    for review in reviews.values_mut() {
        for calendar in &input.calendars {
            let effective = utc(&calendar.effective_at)?;
            if effective <= review.anchor || effective > now {
                continue;
            }
            let change_day = effective.with_timezone(&calendar.zone()?).date_naive();
            if review.due_date > change_day {
                review.due_date = calendar.next_study(
                    calendar
                        .due(review.anchor, review.interval)?
                        .max(change_day),
                )?;
                review.calendar_version = calendar.version.clone();
            }
        }
    }
    let allowance = input.budget_seconds * input.review_percent / 100;
    let mut remaining = allowance;
    let mut due: Vec<_> = reviews
        .iter()
        .filter(|(_, r)| r.due_date <= today)
        .collect();
    due.sort_by_key(|(id, r)| (r.original_due_date, *id));
    let mut selected = Vec::new();
    let mut deferred = Vec::new();
    for (id, review) in due {
        let seconds = *input
            .review_estimates
            .get(id)
            .filter(|v| **v > 0)
            .ok_or_else(|| {
                Error::new(
                    "estimate",
                    format!("Missing positive review estimate: {id}"),
                )
            })?;
        let work = json!({"id":id,"seconds":seconds,"due_date":review.due_date,"original_due_date":review.original_due_date});
        if seconds <= remaining {
            selected.push(work);
            remaining -= seconds;
        } else {
            deferred.push(json!({"work":work,"reason":"does_not_fit_review_allowance","action":"offer_split_or_budget_change"}));
        }
    }
    let used_review = allowance - remaining;
    let mut total_remaining = input.budget_seconds - used_review;
    let mut selected_new = Vec::new();
    let mut deferred_new = Vec::new();
    let mut ids = BTreeSet::new();
    for work in &input.new_work {
        if work.seconds == 0 || work.id.is_empty() || !ids.insert(&work.id) {
            return Err(Error::new(
                "work",
                "Work IDs must be unique and estimates positive",
            ));
        }
        if work.seconds <= total_remaining {
            selected_new.push(work);
            total_remaining -= work.seconds;
        } else {
            deferred_new.push(work);
        }
    }
    // Serialize sorted history to keep provenance stable when caller input order changes.
    let mut assessments = input.history.assessments.clone();
    assessments.sort_by(|a, b| a.id.cmp(&b.id));
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(
            &json!({"attempts":ordered,"assessments":assessments,"disputes":input.history.disputed_assessments,"assistance_updates":input.history.assistance_updates})
        )?)
    );
    Ok(
        json!({"reviews":reviews,"selected_reviews":selected,"deferred_reviews":deferred,"selected_new":selected_new,"deferred_new":deferred_new,"review_allowance_seconds":allowance,"used_review_seconds":used_review,"unused_seconds":total_remaining,"provenance":{"history_digest":digest,"policy_version":input.policy_version,"calendars":input.calendars,"objective_contracts":contracts,"calculated_at":input.now,"budget_seconds":input.budget_seconds,"review_percent":input.review_percent}}),
    )
}
