---
name: coordinate-learning
description: Start or resume a personal learning session from this project’s saved learner and course bundles.
---

# Coordinate Learning

Resolve plugin resources and workspace paths using [runtime locations](../../docs/plugin.md#runtime-locations) before running commands. Keep personal data in the working workspace, never in the installed plugin.

For any visual learning output, enforce [course/study visual ownership](../../docs/learning-operations.md#visual-learning-assets), even when another tool creates it. Return the retained artifact link and verify its topic/lesson/source context before yielding.

Read [operations](../../docs/learning-operations.md) for publication and recovery. Read the [contracts](../../docs/learning-contracts.md) before writing records. Instruction version: learning-skills-v1.

For a new workspace or missing runtime tools, follow [first-run setup](../../docs/plugin.md#first-run-setup) before inspecting learning state. If no course is selected, route source selection to prepare-course; do not call course inspection or auditing without a valid course context.

Use one coordinating agent and one writer. Read learning.local.json for the selected bundle (`selected_course`) and learning-unit record (`selected_course_record`); for shared articles this is `/studies/<slug>/study`. Pass it as `context.study` to the persistence commands; legacy course and learner contexts omit study. Use the [command interface](../../docs/learning-contracts.md#history-and-retained-content) for selected-study calculations; inspect the catalog and shared profile through okf:retrieval. Ask for selection only if absent or ambiguous. Synthetic fixtures are development material, never the learner’s selected course.

Run inspect-state for the selected course/study; article reads return only that study after full-bundle validation. Use next_sequence for bundle-wide attempt sequencing. Present coverage separately from evidence: a completed discussion without attempts is covered, not independently assessed. Resolve any pending OKF journal before writing. Resume pending assessments from saved answers, then reconcile derived progress/schedule and separately retry learner observations. An incomplete or conflicting history blocks affected calculations; it need not block unrelated explanation. Never request an answer again merely because the chat changed.

Before teaching or authoring learner-facing files, run audit-learner-model with the selected context and learner_context. Use its confirmed global_settings/scoped_settings and source assertions; apply saved chat/file language before generating content. Follow [saved-evidence reconciliation](../update-learner-model/SKILL.md#reconcile-saved-evidence). Resolve session > course > confirmed global > accepted default settings with resolve-settings. Treat unconfirmed hypotheses separately. Retrieve applicable policy versions, not just a prose summary. Do not infer traits from absence or silence.

Route onboarding/profile work to update-learner-model, source/plan work to prepare-course, near-term detail to prepare-lesson, due work to review-learning, teaching to run-session, answers/challenges to assess-answer, notes to capture-note, and sheets to make-cheat-sheet. Read the corresponding sibling SKILL.md when routing.

Respect disabled opening suggestions, praise, milestones and next-step agreements; factual progress remains available. Ask timezone/study days only before calendar dates. Standalone articles share `learning.articles`; full courses have separate bundles. A changed topic/chapter stays in its course; a new standalone article selects its own learning-unit record in the shared bundle. A general plan change still needs acceptance.

Before yielding, run audit-learner-model again after saving new evidence. Route pending source reviews to update-learner-model; save linked assertions or an explicit not_needed receipt, then re-run the audit. Report any remaining pending reviews without losing course answers. Persist the cursor, effective settings/origins, pending work and meaningful readiness/support choices using guarded publication. Report what was actually saved and any pending operation. Nothing runs while the chat is closed.

For an opted-in full course (`Course.course_map.enabled`), refresh its [course map](../../docs/learning-operations.md#course-maps) after the saved plan, lesson, session, assessment or revealed-reference changes made in this workflow. Use the stable course-local entry link; keep coverage separate from evidence and planned stages separate from prepared lessons. Do not enable maps for unrelated courses or studies.

## Source-grounded questions

For book-content questions, including questions during a lesson, use `okf:retrieval` with [shared-library operations](../../docs/learning-operations.md#shared-book-library). Resolve the saved selected course's shared book links and usable run; clarify an ambiguous multi-book selection. Default to that edition only; comparisons require a request. Open/check retained passages and cite book/edition/page/passage with coverage gaps. Learner-history questions cite course-owned records. Ordinary explanation creates no Attempt or progress change, and a paraphrase becomes a Note only on request. Propagate `learning.library` in publication/inspection scope when shared links exist, retaining learner reconciliation and study ownership gates.
