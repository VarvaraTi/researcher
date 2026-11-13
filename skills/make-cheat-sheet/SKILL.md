---
name: make-cheat-sheet
description: Prepare lesson reference sheets, reveal covered sections and assemble linked topic summaries while preserving learner additions.
---

# Make Cheat Sheet

Resolve plugin resources and workspace paths using [runtime locations](../../docs/plugin.md#runtime-locations) before running commands. Keep personal data in the working workspace, never in the installed plugin.

For HTML or other visual sheets, follow [visual learning assets](../../docs/learning-operations.md#visual-learning-assets): retain the versioned asset and dependencies inside its course/study, link the sheet to its topics, lessons, sources and artifact, and return that artifact link. Do not maintain a duplicate in project docs or rely on a temporary preview.

When preparing a lesson, save a CheatSheet alongside it with a content version, section IDs, topics, source/lesson links, main rules, a small example and relevant pitfalls. Support every factual section with selected-source locators. Keep unstudied sections unrevealed. A prepared sheet is not evidence the learner has covered it.

At lesson end, intersect covered items with sheet section IDs and publish revealed_sections with test guards. Present only those sections. Partial lesson coverage reveals only its subset. Record uncertainty as a labeled reminder, not an accepted fact. Once revealed, sections remain available outside practice; consulting them during recall records assistance.

After all required topic activities are completed or explicitly skipped, provide a final topic sheet as linked contents: main idea plus link to each detailed sheet, clearly marking skipped/uncovered material. Completion is distinct from demonstrated evidence. Do not expose unstudied sections as if covered.

Keep personal additions as separately authored Note references and retain them across content versions. Use textual lists/tables or a small grounded Mermaid diagram with a text fallback. A new content version never silently overwrites personal text or historical exercise meaning.

For an opted-in full course (`Course.course_map.enabled`), refresh its [course map](../../docs/learning-operations.md#course-maps) after the saved plan, lesson, session, assessment or revealed-reference changes made in this workflow. Use the stable course-local entry link; keep coverage separate from evidence and planned stages separate from prepared lessons. Do not enable maps for unrelated courses or studies.

Ground book-derived sections through [shared-library retrieval](../../docs/learning-operations.md#retrieve-book-evidence): inspect selected-run passages, cite precise page markers, and retain optional shared source/passage/concept links alongside existing local sources. Include explicit library scope for publication. A Concept is navigation; it does not replace source evidence or a course Topic. Preserve historical sheet versions and course/study ownership of visual artifacts.
