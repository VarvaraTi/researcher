---
name: prepare-lesson
description: Prepare the next grounded lesson, checks, retained rubrics and cheat sheet using saved learning feedback.
---

# Prepare Lesson

Resolve plugin resources and workspace paths using [runtime locations](../../docs/plugin.md#runtime-locations) before running commands. Keep personal data in the working workspace, never in the installed plugin.

Retain lesson images, diagrams, animations and HTML inside the owning course/study, with versioned artifact paths and lesson/sheet/source links, following [visual learning assets](../../docs/learning-operations.md#visual-learning-assets).

Retrieve the accepted course map, effective settings, objective contracts, prior resolved judgments, readiness choices and support applications. Keep the map stable while adapting near-term presentation. Read prepare-course for grounding boundaries and make-cheat-sheet for section coverage.

Prepare small theory blocks with exact selected-source locators. Include appropriate optional practice, or an explanation/discussion path. Before presenting a check, save its prompt, objective, kind, rubric/answer key, estimated seconds, hints and immutable version. Keep solutions separate from the initial learner-facing prompt. Do not execute learner code.

Prepare the lesson sheet in the same proposed changeset. Its sections identify the material they cover; revealed_sections starts empty. Keep personal additions as separately authored Note links. Save a semantic source-review rationale and source_supported only after inspecting supporting passages.

Adapt the next lesson from saved evidence: a smaller worked example after uncertainty, varied application after recall-only evidence, or less material after an explicit time request. Save rationale, origin evidence and applicable policy version. Changes to goals or the course map require acceptance; ordinary presentation changes do not.

Publish the lesson, exercise versions and sheet together through the structural and learning gates. A ready lesson or exercise is retained: create a new version for changed content. Mark stale eligibility through a separate pending/source-review record while keeping historical content inspectable.

For an opted-in full course (`Course.course_map.enabled`), refresh its [course map](../../docs/learning-operations.md#course-maps) after the saved plan, lesson, session, assessment or revealed-reference changes made in this workflow. Use the stable course-local entry link; keep coverage separate from evidence and planned stages separate from prepared lessons. Do not enable maps for unrelated courses or studies.

For shared-book evidence, follow [library retrieval](../../docs/learning-operations.md#retrieve-book-evidence). Open/check active-run passages and exact page markers before marking content supported. Retain optional shared source/passage/concept links with explicit `learning.library` scope. A partial book index blocks only unsupported affected content. Do not rewrite historic lessons to add links or treat extraction as diagram review.
