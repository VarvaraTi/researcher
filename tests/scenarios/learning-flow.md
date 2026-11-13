# Behavioral review: two-lesson learning flow

Instruction version: learning-skills-v1. Fixture: `tests/fixtures/workspace`.
This is a synthetic scenario; none of its answers, preferences or acceptance
statements is a real learner claim. Run `sh scripts/check-learning.sh` first.

## Development walkthrough and observed artifacts

1. Inspect learner `/responses/onboarding`: the starting-difficulty question has
   `{skipped:true}`. It is not a psychological score or an inferred trait. The
   saved global language is Polish; the course override is English; session
   context requests short blocks. The resolver tests preserve their scopes.
2. Inspect `/sources/sets-v1` and its retained `references/source.txt`: section 1
   says elements are distinct; section 2 defines union. The two exercise rubrics
   check only those statements. The source is explicitly authored synthetic text,
   not an uninspected external citation. The initial accepted map is fixture-only.
3. The first lesson gives a distinctness explanation and offers a suitable check.
   The request fixture preserves “Two, because repeated a counts once.” exactly.
   The integration test saves it before judgment; restarting inspection returns
   it as pending, without a second submission.
4. The correction test intentionally saves an incorrect judgment, then supersedes
   it with a correct one. Both remain inspectable. Its separate ReadinessChoice
   says continue while transfer remains unchecked. The progress function receives
   no readiness field, so this choice cannot manufacture demonstrated evidence.
5. The test reveals only `distinct`, leaving `example` unrevealed; adapts the second
   draft lesson to begin with a smaller worked example; updates the saved cursor;
   and leaves the accepted course map unchanged. This demonstrates persistence,
   not an independent assessment of conversational teaching quality.
6. At topic completion the make-cheat-sheet workflow produces linked contents:
   Distinctness → first sheet; Union → second sheet. Mark explicitly skipped or
   uncovered sections. This final presentation is instruction-reviewed, not yet
   observed in a learner session.
7. The scheduler tests replay saved judgments, correction, known/unknown
   assistance, same-session retries, missed days, terminal intervals, calendar
   changes and a 20-minute budget with a five-minute review allowance. These use
   simulated timestamps; they do not establish actual delayed retention.

## Manual fresh-chat run still required

Start a new chat in this repository and invoke `$coordinate-learning`. For a
synthetic rehearsal, explicitly select a disposable copy of the fixture catalog;
for real use, select an actual article/book excerpt and create private runtime.
Review these behaviors by interaction, recording actual responses and record IDs:

- Skip an onboarding question; request a temporary language/block-size override.
- Approve a sourced map, then stop halfway through the first lesson.
- Submit an answer, pause after save, restart, and confirm no answer repetition.
- Say “I got it right but still feel unsure; I'd like to continue.” Confirm an
  optional clarification offer, persisted readiness choice, and no compulsory task.
- Confirm only covered sheet sections are shown and the next lesson adapts without
  changing the accepted course map.
- Challenge feedback, inspect both judgments, and recompute derived evidence.
- Add a personal note with unusual spacing; verify later sheet updates preserve it.
- Decline additional questionnaires/source search; confirm no inferred diagnosis
  or unauthorized new material. Disable praise/opening suggestions and observe it.
- Complete the topic by discussion where appropriate; inspect the final linked
  contents sheet and distinguish completion, skips and demonstrated criteria.
- Return for a later independent review and separately report comfort/usefulness.

Record source locator review, rubric adequacy, observed behavior, UTC time,
instruction version and the actual host model identifier when exposed. Never
fill pending rows with simulated learner comfort or fabricated retention results.


## Shared standalone articles

1. Prepare two unrelated standalone articles in learning.articles, each below
   studies/<slug>/ with a Study, domains labels, explicit study ownership and plan.
2. Select article B through selected_course_record and context.study. inspect-state
   must return only B's records/history/pending work after validating the full bundle.
3. Save attempts with bundle-wide sequences and globally unique submission IDs.
   Cross-study exercise/session/objective links and mutations to A must fail.
4. Complete B as a discussion without a test. Save covered_objectives, summary and
   understanding_check=not_performed; display coverage separately from mastery.
5. Switch to A and resume its cursor. A's source snapshots and exact answers remain
   unchanged. Classifying B under another domain does not move files or change A.
6. Full courses retain their own bundle/schema. Chapters and supporting articles
   stay in the existing full-course bundle.

## Visual artifact ownership

Given a selected course or article study and a request for an animated HTML cheat
sheet, verify that the retained file and local dependencies live under that
unit's versioned `references/visualizations/` paths. Its Topic/Lesson/CheatSheet
context must link the asset to the topic and precise source passages. Follow the
link returned in chat: it must target the retained artifact, not project docs or
a temporary preview. Resolve local links from that location and check that an
export retaining the unit and dependencies preserves them. A temporary preview
or unrelated general project diagram must not become the canonical learning
asset. When correcting duplicates, compare content before removing only the
redundant copy; preserve original learner additions and historical versions.
