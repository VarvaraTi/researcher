---
name: update-learner-model
description: Inspect, correct or extend shared learning preferences, goals, optional onboarding and support trials.
---

# Update Learner Model

Resolve plugin resources and workspace paths using [runtime locations](../../docs/plugin.md#runtime-locations) before running commands. Keep personal data in the working workspace, never in the installed plugin.

Use coordinate-learning’s contracts and publication protocol. Retrieve the selected shared profile and explicit course scope via okf:retrieval. Show inspectable current assertions; derive the working summary from them.

Offer short optional questions about goals, feedback, habits or starting difficulties, reusing known answers. Each can be skipped; save skips in QuestionnaireResponse.original_answers. Do not attach scores to this custom preference questionnaire. Subject diagnostics are separate. An additional instrument requires a concrete need, research of its source/version and permitted use, intended population, limits and permitted decisions before adoption; it can be declined.

A direct preference request authorizes that scope. Save a new preference with source, timestamps and optional supersedes; retire the superseded assertion with a test guard. A session request stays in Session. Confirm inferred lasting preferences and goal/general-plan changes before activation. Keep self-report, observations and unconfirmed hypotheses distinct. Correcting or retiring a claim changes subsequent settings resolution; never transfer subject knowledge between courses without objective correspondence.

Read the [default support policy](../../policies/defaults/support-v1.json) and the active bundle copy. Apply a smaller first step, alternative explanation or shorter session when supported by the request. Save each StrategyApplication with stable origin_id and policy version via save-strategy-application. Call review-support with saved applications, reviewed origin IDs and the active policy threshold; at the policy threshold offer a brief usefulness review, earlier if obstructive. Keep comfort/usefulness separate from performance; one success does not show causation. Offer alternatives without imposing a questionnaire after every task.

Course-derived LearnerObservation uses origin_id = course catalog ID + ':' + original event ID + ':' + observation purpose. Add standard sources with the document-relative course event resource and matching bundle_ref, then save via save-observation; retry it separately if the shared bundle fails. Do not lose or resubmit the course answer. Export, deletion and migration follow operations; deletion requires the learner’s explicit request.

## Reconcile saved evidence

Run audit-learner-model before and after reconciliation. Process its source/digest review items, never
invent observations to fill the profile. Publish justified assertions with an
immutable LearnerModelReview receipt (or not_needed with a reason). Use the returned
review_id, verify referenced assertions exist, and re-audit after publication.
A changed source needs re-review; append corrections rather than rewriting history.
Preserve event time and record the actual backfill time separately. Confirmed
course/study preferences remain scoped; do not promote them to global settings.

Capture meaningful unsaved chat evidence before auditing: the command sees saved
records only. Use the selected course/study context and a separate learner_context;
never pass the article Study selection as learner_context.study. Review each
pending source before saving a receipt. Reuse an existing justified assertion
rather than creating one just to clear the queue; retain standard source links to
the original event and bundle. A not_needed receipt requires an explicit reason.
If publication fails or review remains incomplete, report pending items and retry
later from saved evidence. Do not claim the profile is current while
review_complete is false, or ask the learner to resubmit a saved answer.
