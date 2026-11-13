---
name: run-session
description: Teach a prepared lesson in small blocks, save original answers, handle readiness and persist pause/resume.
---

# Run Session

Resolve plugin resources and workspace paths using [runtime locations](../../docs/plugin.md#runtime-locations) before running commands. Keep personal data in the working workspace, never in the installed plugin.

Show only retained course/study visual assets linked to the current topic, lesson/sheet and sources. Follow [visual learning assets](../../docs/learning-operations.md#visual-learning-assets); do not hand over project-doc or temporary copies as learning references.

Start through coordinate-learning to retrieve the saved cursor and effective preferences. Load only the ready lesson and supporting sources needed now; do not re-ask a saved answer. Explain the session scope and approximate budget. Allow shortening, pause, discussion and explicit skips.

Present one small block/check at a time. Practice must suit the objective; explanation-only material need not force a practical task. Record covered items and SkipEvent separately. Reuse adequate prerequisite evidence; an explicit skip does not mark learned.

Assign and retain a submission ID before attempting save. Save the exact original answer and immutable exercise version via save-attempt, then route to assess-answer. Record observed/self-reported assistance; external assistance may be unknown. Opening a cheat sheet during recall counts as assistance. An immediate correction after feedback uses independent_eligible=false and is practice, not delayed retention.

For a correct answer with residual uncertainty, ask whether the learner is ready to continue. Offer clarification or an optional check without imposing it. Persist ReadinessChoice and uncertainty separately. Continuing does not fabricate demonstrated knowledge.

Apply support policies when appropriate, saving reason/version and stable StrategyApplication. Ask before generalizing temporary context. End with covered sections only via make-cheat-sheet, concrete factual progress, unresolved difficulties and optional next-step agreement. Save session cursor, completed/skipped items and effective settings/origins before yielding.

For an article study, pass context.study for all course persistence. On completion,
save a substantive session summary, covered_objectives and understanding_check.
A discussion without retained attempts is not_performed; show its coverage without
claiming demonstrated understanding. Use the article-studies contract in docs.

After saving feedback/assessment or session changes, run audit-learner-model and
route pending sources to update-learner-model. Complete a receipt only after its
assertions exist, or record why no new learner assertion is justified. Report
pending updates on failure; never re-request or resubmit an already saved answer.

For an opted-in full course (`Course.course_map.enabled`), refresh its [course map](../../docs/learning-operations.md#course-maps) after the saved plan, lesson, session, assessment or revealed-reference changes made in this workflow. Use the stable course-local entry link; keep coverage separate from evidence and planned stages separate from prepared lessons. Do not enable maps for unrelated courses or studies.

Route ordinary book questions through [shared-library retrieval](../../docs/learning-operations.md#retrieve-book-evidence), within the selected book unless comparison is requested. Open/check cited passages and disclose coverage gaps; text extraction does not verify diagrams. Discussion alone creates no Attempt or progress evidence. Save a generated paraphrase as a course Note only when requested. Include `learning.library` in context.scope_bundles when validating shared references.
