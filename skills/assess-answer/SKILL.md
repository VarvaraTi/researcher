---
name: assess-answer
description: Assess a saved learning submission or resolve a challenge while preserving original answers and judgment history.
---

# Assess Answer

Resolve plugin resources and workspace paths using [runtime locations](../../docs/plugin.md#runtime-locations) before running commands. Keep personal data in the working workspace, never in the installed plugin.

Use inspect-state and load the retained exercise/rubric referenced by the saved Attempt. If the answer has not saved successfully, save it before assessment. If assessment fails, the answer remains authoritative pending work; never create another submission for the retry.

Use grade-answer only for saved option IDs or an explicit deterministic comparison rule. Judge open answers against named criteria, accepting equivalent wording; distinguish self-reported practical results from checked results. Save assessor, criterion results, correct/partial/incorrect/unassessable outcome and explanatory feedback with source locators. Uncertainty must not become a confident failure. Record available model identifier and learning-skills-v1 when known, without guessing an unavailable model ID.

Create a separate Assessment. A challenge creates an open AssessmentDispute; do not advance or worsen evidence from an unresolved judgment. Reassess against the original retained rubric. A correction creates a new Assessment whose judgment.supersedes points to the old assessment for the same attempt, and resolves the dispute with a guarded patch. Keep all judgments. Run resolve-assessments/check-learning-state before activation; cycles, cross-attempt replacement and active branches block affected derived work.

Recompute compute-progress and compute-schedule from complete saved history after publication. Corrections take effect at the original attempt’s recorded position. Do not derive authoritative status from feedback prose or a readiness choice. Save pending derived work if publication fails. Shared-profile observations use a stable original event link and are independently retryable.

After a correct answer with uncertainty, return to run-session’s readiness conversation. Give brief feedback tailored to the learner’s preferences; do not require another check merely to remove the agent’s uncertainty.

After saving feedback/assessment or session changes, run audit-learner-model and
route pending sources to update-learner-model. Complete a receipt only after its
assertions exist, or record why no new learner assertion is justified. Report
pending updates on failure; never re-request or resubmit an already saved answer.

For an opted-in full course (`Course.course_map.enabled`), refresh its [course map](../../docs/learning-operations.md#course-maps) after the saved plan, lesson, session, assessment or revealed-reference changes made in this workflow. Use the stable course-local entry link; keep coverage separate from evidence and planned stages separate from prepared lessons. Do not enable maps for unrelated courses or studies.
