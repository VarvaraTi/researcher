# Personal Learning Assistant — Specification

Status: Current product requirements. Deterministic runtime and learning workflows are implemented; remaining behavioral evaluation is described in section 14.
Product intent: [learning-intent.md](learning-intent.md).
Historical compatibility research: `learning-okf-capability-audit.md` in the source checkout. Current interfaces are defined by the pinned runtime, schemas and contracts.

Requirements below implement the intent. Accepted defaults and current requirements are distinguished from deferred extensions. Product defaults are not claims of optimal educational effectiveness. Requirements alone do not establish that a behavioral scenario has been verified. Architecture rationale is recorded in [ADRs](docs/adr/README.md); exact runtime rules are in [contracts](docs/learning-contracts.md).

## 1. Product and First Release

The assistant turns selected source material into a manageable learning plan, daily lessons, practice, feedback, and later review. Personal notes and concise cheat sheets remain connected to the material after the course ends.

Adaptation to the learner is a core product requirement. The first release serves one learner through a new Codex chat, stores a shared learner model in one local OKF bundle, stores standalone articles together in one shared articles bundle, and stores each full course in its own bundle. A dedicated website is optional future work. The learner's language is a course preference; this specification being in English does not require lessons to be in English.

The first complete experience must support a shared learner profile, one topic, one or more selected sources, several progressively prepared lessons adapted to that learner, saved answers, feedback, personal notes, a cheat sheet for each lesson, a final linked topic sheet, and a subsequent review session.

Included in v1:

- Source selection and inspection, course planning, and a daily schedule.
- Short theory sections with precise source references.
- Multiple-choice questions, free-text answers, and practical tasks with written results or attached artifacts.
- Hints, answer assessment, assessment challenges, and saved attempts.
- Review scheduling, resumable sessions, and progress views.
- Personal notes, questions, textual cheat sheets, and simple visual explanations.
- A visible, editable learner model, progressive profiling, optional questionnaires, and configurable teaching and support strategies.

Deferred: multiple users, competitive rankings, automatic execution of submitted code, a dedicated graph editor, an automatically optimized learning algorithm, general personality inventories, and external reminders. No external note-taking application integration is required.

## 2. Responsibilities and Architecture

| Component | Responsibility |
|---|---|
| Current chat agent and learning skills | Inspect material, propose a plan, explain, generate exercises, assess open answers, and guide sessions |
| Existing OKF skills | Guide the agent through established bundle creation, modeling, ingestion, retrieval, and maintenance workflows |
| Focused learning code | Enforce learning-specific invariants and state transitions and compute schedules and progress where existing OKF capabilities are insufficient |
| OKF CLI | Read, search, create, edit, and check bundle records through supported commands |
| OKF bundles | Persist the shared learner model, policies, sources, course content, learner notes, attempts, assessments, and schedule records |
| Interface | Present lessons and collect learner actions; initially conversational |

Learning skills reuse existing OKF skill workflows rather than replacing them with ad hoc command sequences. A skill is an instruction set followed by the current chat agent, not an executable service or a separate agent. Learning skills direct the agent to the appropriate OKF skill; the agent follows that workflow and executes the supported OKF CLI operations.

There is no mandatory new middleware layer between the agent and OKF. Ordinary knowledge-base operations use existing OKF workflows. Add focused code only for learning-specific requirements that need deterministic enforcement, such as schedule calculation, progress calculation, retry deduplication, and assessment-transition checks not already provided by OKF. Instruction text alone is not a substitute for these guarantees. Reuse supported OKF persistence and recovery mechanisms instead of building another storage engine; verify capabilities against the installed version before relying on them.

Expose any required learning code through small local commands callable by the agent. Such commands may use OKF to persist validated changes, but reads and ordinary writes do not all need to pass through them. A separate server, model connection, or comprehensive learning CLI is not required. The implementation uses an independent Rust library with thin local commands, as recorded in [ADR 0001](docs/adr/0001-okf-and-local-rust-commands.md). Python is not a product dependency.

The first release uses the model supplied by the Codex chat; no separate model API connection or independently deployed model is required. An implementation should record the model identifier when available and the instruction version used for generated content. Evaluate explanation grounding and assessment quality using the first real course.

Given identical saved inputs, settings, and an explicit current time, scheduling and progress calculations must produce the same result. AI-generated wording, planning suggestions, and open-answer judgments are not guaranteed to repeat exactly. Persist accepted outputs rather than regenerating them on every read.

### 2.1 Verified implementation boundary

The installed OKF 0.6.0 provides catalogs, structured queries, typed ontology checks, bounded graph inspection, source drift/impact queries, guarded patches, and recoverable single-bundle changesets. Reuse these through existing OKF skills. Do not create replacement storage, search, catalog, ontology, or transaction systems.

Focused deterministic functions cover the missing learning semantics:

| Function | Inputs and required result |
|---|---|
| Review scheduler | Active assessments, assistance, objective history, rule version, study days, timezone, time budget and explicit time → review stages, due dates and an ordered, bounded queue |
| Progress calculation | Attempts, active assessments, required items and explicit skips → activity, completion and learning evidence as separate outputs |
| Prerequisite validation | The prerequisite relation only → reject cycles with the cycle path; ordinary related-topic cycles remain allowed |
| Assessment-history validation | Attempts, assessments and supersession links → one unambiguous active assessment per assessed attempt; reject cross-attempt replacement, cycles and conflicting active branches |
| Preference resolution | Current-session, course, global and default settings → effective settings with their origins and policy versions |
| Deterministic assessment | Saved option IDs or predefined comparison rules → reproducible result; instrument-specific scoring is added only for selected questionnaires |

Use stable record paths and OKF patch/changeset guards for retry and transition protocols. Add narrow orchestration/checks where these primitives do not establish the complete learning requirement. Code does not need to intercept ordinary OKF reads and writes. Supported workflow guarantees do not prevent manual edits to local files.

### 2.2 Plugin distribution and runtime

Distribute reusable skills, schemas, helpers and supporting resources through an
explicit package allowlist, excluding personal data, caches and upstream checkouts.
Build writable commands outside the installed plugin cache. Setup initializes only
missing storage and preserves existing registrations, records and selections;
book-library setup is on demand. Installed documentation links must resolve without
the source checkout. Architecture rationale:
[ADR 0006](docs/adr/0006-plugin-packaging-and-runtime.md).

## 3. Course Setup and Planning

Start with a few short questions and progressively clarify the profile during learning. Collect the learning goal, preferred language, prior knowledge, selected sources, timezone, available study days, and daily time budget as needed; do not require a long psychological questionnaire to begin. Reuse confirmed preferences from the shared learner model. Offer a short optional subject diagnostic. Skipping it is allowed and leaves the initial level provisional.

Proposed defaults, used only when the learner accepts them: 20 minutes per session, difficulty starting at introductory level, and no deadline. Study days and timezone must be explicit before assigning calendar dates.

The plan contains:

- A concrete learning goal and observable objectives, such as explaining a concept or solving a class of problems.
- Topics with source references, difficulty labels, and explicit prerequisites.
- Lessons with objectives, estimated minutes, and scheduled local dates.
- Practice and review time within the daily budget.
- Visible gaps, assumptions, and source limitations.

Prerequisite links must form a directed acyclic graph. Reject cycles with a readable explanation. Ordinary related-topic links may form cycles. The learner can revise the plan, skip known material, or return to prerequisites; skipped material is not automatically counted as learned.

Show the proposed plan for acceptance before starting scheduled lessons. Subsequent changes preserve completed work and show which future sessions changed. A time estimate is approximate and is updated using learner feedback, not treated as a deadline.

Keep three planning levels: an accepted course map of goals/topics/dependencies, a revisable near-term lesson plan, and the current session's activities. Prepare detailed upcoming lessons progressively after course-plan acceptance, using previous results and current preferences. Prepare each lesson's cheat sheet with that lesson. Changes to goals or the general plan require learner acceptance; local presentation and support adjustments do not require repeated approval.

## 4. Source Grounding

Every instructional block, exercise rubric, and cheat sheet must identify supporting sources with useful locators: a heading, page, paragraph range, or timestamp where available. A broad homepage link alone is insufficient evidence for a specific explanation.

Source records store title, location, format, access status, inspected date, and an available content fingerprint or version. Store an inspectable local snapshot when permitted and practical, or retain the original location and version information.

Initial input support: Markdown, plain text, inspectable web text, and text-layer PDFs on macOS via `scripts/read-pdf.sh` (Swift/PDFKit). PDF extraction preserves PDF page locators but does not inspect diagrams or layout. OCR for scans and video extraction remain outside v1; offer an accessible text version or transcript instead. Use diagrams within supported sources only when an appropriate viewing capability can inspect them. If a source cannot be inspected, mark it unavailable; do not generate purportedly grounded lessons from its title alone.

The assistant must distinguish source-derived explanations, assistant-created examples, and learner-authored notes. Newly proposed prerequisite material needs sources and learner acceptance before becoming part of the course scope.

Search for additional course sources only at the learner's request, and always obtain agreement before adding them to the selected course material. Inspection of an already selected source does not require a new source-selection approval. If coverage is insufficient, explain the gap and offer to search; do not silently expand the source set.

A structural check can verify that references exist; it cannot prove that a source supports a claim. Content preparation therefore includes a semantic check of explanations and rubrics against the referenced passages. Unresolved conflicts or insufficient evidence block the affected content from being marked ready.

When a source or lesson changes, flag dependent content for review. Preserve old attempts and their content versions. Updated content must not silently alter what an earlier attempt meant.

### 4.1 Book library and grounded questions

Keep reusable book sources in the private shared library (`learning.library`),
separate from course learning history. Retain one Source per edition, its canonical
original PDF and all extractable original text in immutable, section-based
SourcePassage records. Split long sections at paragraph boundaries while preserving
page text and exact 1-based PDF page markers. Printed page labels and headings are
used only when inspected. Original text must not be replaced by generated summaries.

Each extraction run has an immutable IndexManifest accounting for indexed,
no-text and failed pages, source digest and extractor version. Partial extraction
permits work supported by available passages; it does not make unsupported content
ready. Retaining a PDF or inspecting its contents page does not establish searchable
chapter coverage. Extraction, semantic passage review, diagram review and learning
progress are distinct. Do not claim OCR or diagram understanding from text extraction.

For ordinary book questions, resolve the selected edition and usable extraction
run, search its passages and Concept aliases through OKF, open candidate text and
check source digest, run membership, review blocks and exact locators. Do not use
search snippets, generated lessons, plans or learner answers as sole evidence for
claims attributed to the book. Cite book/edition, precise page or inspected heading,
and the retained passage. Distinguish source claims, assistant explanations/examples
and learner history; learner-history answers cite their owning learning records.

If evidence is insufficient, report the selected scope, coverage gaps and search
limits. Try related terms or sections as needed; no lexical match does not prove
absence. Incomplete query delivery, stale source bytes, blocked evidence and source/
run mismatches cannot establish support. External source search and inclusion
follow the source-authorization rule in section 4; separately label external material.

Concepts navigate equivalent or related passages without merging source-specific
claims. Link clear meaning/scope matches; ask about ambiguous matches. Compare
books only on request, citing each edition separately and preserving disagreement.
Generate explanations when useful, but save a paraphrase as a course Note only on
request. Do not maintain a canonical generated cross-book synthesis by default.

A new edition or changed source requires an explicit update choice. Preserve old
runs, original course citations, historical inspection statements and learning
records. Run activation and review evidence use separate records. New live
migrations require approval, a recoverable private backup and guarded publication;
per-book coverage and rollout receipts belong in the owning runtime. Exports must
include dependencies or an explicit omission manifest, without publishing substitute
book copies. Indexing never creates learning evidence or enables a map by itself.

Use the existing lexical retriever initially. Any future semantic/hybrid search
requires measured retrieval failures and a separately approved change, preserving
source identity, locators and original text separately from generated search context.
Architecture rationale: [ADR 0004](docs/adr/0004-shared-book-library-and-retrieval.md).
Exact fields: [library schema](schemas/library-ontology.yaml). Runtime invariants:
[shared-library contract](docs/learning-contracts.md#shared-library-contract).
Commands and recovery: [library operations](docs/learning-operations.md#shared-book-library).

## 5. Lesson Experience

Each lesson has a title, objectives, prerequisites, estimated duration, one or more short theory blocks, appropriate checks or optional practice, and a closing summary. Not every kind of material requires a practical task; select checks according to the learning goal rather than requiring exercises for every block.

Use small learning activities with an explicit objective, prerequisites, explanation/example, success criteria and estimated duration; include practice where it serves that objective. Microlearning is a flexible unit of organization, not a universal minute limit. Combine short activities with larger tasks that connect ideas where appropriate. Use retrieval practice, spaced practice, worked examples with gradually reduced assistance, and self-explanation according to the material and current knowledge, not as mandatory steps for every topic.

Session flow:

1. Load effective learner/course settings and the saved session. Use known current constraints; ask a brief context question only when useful. Show today's focus, estimated time, and due reviews within the available budget.
2. Present a manageable theory block and its references.
3. Where a check is appropriate, offer a question or task before revealing its solution. Allow explanatory or discussion-based material to proceed without imposing a practical task.
4. Save the learner's submission before assessing it.
5. Give feedback, optional hints, and an opportunity to try again.
6. Save a short reflection or personal note if the learner wants.
7. Reveal the covered lesson cheat sheet at the end of the lesson; summarize completed work, unresolved difficulties, and the next step.

The learner can pause, shorten, or resume the session. Persist the current lesson and item so resuming does not depend on chat history. Completion requires the required items to be attempted or explicitly skipped; a skipped item remains visible and does not count as successful practice.

When the learning workflow is opened, the coordinator may suggest a suitable next step; the learner can disable this initiative. Provide project/skill entry instructions so a new chat can discover the selected bundles and resume. OKF does not independently launch chats or send notifications.

## 6. Exercises and Assessments

Each exercise defines its target objectives, prompt, difficulty, source references, expected answer or rubric, hints, estimated time, and content version before it is presented. Solutions are separate from the initial prompt. In a local file-based prototype they are not secret from someone browsing the underlying files.

| Format | Assessment |
|---|---|
| Multiple choice | Match saved option identifiers against the answer key |
| Constrained answer | Use a predefined comparison rule only where appropriate; show tolerances or accepted forms |
| Free text | Assess meaning against explicit rubric criteria; allow equivalent wording |
| Practical task | Inspect submitted reasoning, results, or artifacts against a rubric; distinguish reported results from independently checked results |

Assessment outcomes: `correct`, `partial`, `incorrect`, or `unassessable`. Feedback identifies satisfied and missing criteria, explains the issue, and points to relevant material. An uncertain assessment must not become a confident failure. Record assistance separately: hints used, solution revealed, cheat sheet consulted, or assistance unknown.

For a chat interface, assistance tracking includes observable actions and learner self-report; the system cannot reliably detect reference use outside the interface.

Every submission creates an attempt record that is immutable within the supported learning workflow, including the original answer, exercise version, timestamp, and observed assistance. OKF itself permits editing an Attempt document; its type name does not enforce immutability. Assessments are separate records. A new learner answer creates a new attempt; retrying transmission of the same submission does not. Challenging an assessment triggers review; a correction supersedes the previous assessment without deleting its judgment. Disputed assessments do not advance review stages until resolved.

If assessment fails after submission, retain the answer and persist pending assessment work separately from the immutable answer. Retry assessment without duplicating the attempt. Use assessment_status for assessment workflow states and validate supersession links before deriving progress. If history is ambiguous, block the affected derived update and request reconciliation. Code and uploaded artifacts are not executed automatically in v1.

### 6.1 Evidence of understanding and readiness to continue

Define observable objectives and assessment criteria before presenting a check. Suitable evidence may include an explanation in the learner's own words, independent application, interpretation of a new example, finding an error, or delayed recall. Select only the checks appropriate to the material; neither all these forms nor a universal number of correct answers is required for every objective.

The agent judges criterion-level evidence, while deterministic progress rules use the saved assessment, assistance and history. A self-report of understanding is valuable readiness information but does not by itself establish independent performance. An immediate corrected answer after feedback records practice progress, not delayed retention. A single error may warrant clarification rather than a broad conclusion of non-understanding. Uncertain or disputed evidence must not automatically worsen progress.

When the answer is correct but the agent remains unsure whether the learner has understood sufficiently, ask whether the learner feels ready to move to the next topic. Offer a brief clarification or optional check if wanted; do not automatically impose more tasks. If the learner chooses to continue, record that choice and the limits of the available evidence separately. Readiness to continue and demonstrated knowledge are distinct; the choice to continue does not fabricate mastery or erase existing evidence. For material that does not call for a task, a readiness conversation may be the appropriate way to proceed.

Report evidence concretely, such as "explained the idea independently; transfer and delayed retention not yet checked," rather than inventing a precise percentage of understanding. Define the minimum evidence for demonstrated status by objective/type of goal during course preparation; each objective declares its required criteria and allowed evidence kinds, without a fixed threshold for all subjects.

## 7. Review and Daily Scheduling

Use a transparent, configurable rule per learning objective. The accepted initial review intervals are 1, 3, 7, and 14 calendar days, with approximately 25% of the session time budget allocated to reviews. The learner can change this allocation. These are trial product defaults, not a validated optimal schedule. Adjustments follow the implemented review-v1 policy. Exact stage, event-ordering, calendar and budget rules are defined in the [review contract](docs/learning-contracts.md#review-policy-and-event-ordering).

An objective enters review after a correct, unassisted attempt meeting its declared evidence criteria, with the first interval of one calendar day subject to study-day adjustment. A correct unassisted qualifying review advances one stage and schedules the next interval from the review date. At the last stage, future reviews remain 14 days apart as the initial rule.

After an error or assisted response, prefer a brief review tailored to the learner and the specific difficulty. Do not mandate resetting the whole interval sequence for every such response. Store the selected support and scheduling policy explicitly. Under review-v1, a resolved failure or assisted response schedules the next study day and retains the stage. A same-session lapse takes precedence over subsequent success; unknown assistance and disputed or unassessable outcomes do not advance the stage.

Do not advance an objective through several review stages in one session. A corrected answer immediately after explanation is successful practice, with independent retention checked later. If assistance is unknown, clarify only when it affects progress; without clarification, save the response but do not advance the review stage. An unassessable or disputed response leaves the stage unchanged pending clarification or reassessment.

If a computed date is not a selected study day, move it to the next selected study day. Store timestamps in UTC and scheduled dates in the learner's timezone. A timezone change recalculates future schedule dates explicitly without changing historical timestamps.

Build the review portion of the daily queue from due reviews, sorted by oldest due date and then stable objective ID, within the review allocation. Use unused review time for new material and add eligible lesson activities within the remaining total budget. Use positive integer-second estimates and floor the review allowance to seconds. Smaller due items may fill the allowance when an older item does not fit. Show overflow as deferred work; do not erase overdue dates or automatically lengthen the session. If an item does not fit, offer to split it where meaningful or explicitly agree a different allocation/session duration.

For prerequisites requiring a foundational skill, use existing adequate evidence or offer a short appropriate check; do not re-test a skill already demonstrated without a reason. The learner may explicitly choose to skip a prerequisite, which remains visible and is not marked learned. This does not impose practical tasks on all material or override the readiness choice in Section 6.1.

Missing a day does not erase progress. On return, offer a manageable next step without requiring an explanation for the absence. Discuss obstacles only if the learner wants to. Replan unfinished work while preserving the accepted goals and requesting agreement for changes to the general plan. Review activities should test the objective using an appropriate task; repeatedly selecting the same memorized answer is insufficient evidence of broader application.

## 8. Progress and Support

Keep activity and evidence of learning separate:

- Activity: lessons attempted or completed, practice attempts, and notes written.
- Learning evidence: objectives demonstrated independently, assistance needed, recent review results, and unresolved difficulties.

Use descriptive states such as `not_started`, `practicing`, and `demonstrated`, with review dates shown separately. Demonstrated means the objective's criteria were met under recorded conditions, not that the whole topic is permanently mastered. A resolved failed review returns an objective to `practicing`; historical successful attempts remain visible. Merely choosing to move on does not require demonstrated status or change the evidence on record.

Offer individually configurable support: calm progress information, praise for specific actions, small milestones, and agreements about the next step. Praise, achievements and next-step agreements can be disabled; factual progress remains available. Do not use competitive rankings, punitive streaks, or lost progress for absences. Offer an easier explanation, smaller task, or break when requested. Do not infer psychological diagnoses or stable personality traits from performance, silence or missed sessions.

### 8.1 Open learner model

Keep the following concerns separate so each can be updated without rewriting the others:

| Concern | Scope and contents |
|---|---|
| Shared learner profile | Confirmed language/interaction preferences, interests, goals, accessibility needs, study habits and requested psychological support across courses |
| Course settings | Explicit overrides and goals for one course |
| Knowledge state | Evidence per objective, misconceptions, assistance and delayed review results; stored with the course |
| Session context | Current time availability, self-reported energy and immediate intentions; temporary unless the learner explicitly generalizes them |
| Adaptation policies | Versioned rules for selecting teaching and support actions, with their rationale and limits |
| Plans | Accepted course map, near-term plan and current activities derived from the above |

Effective setting priority is current explicit request/session setting, then course setting, then shared profile, then default. Record where each effective value came from. A temporary request must not silently overwrite a permanent preference. Transfer confirmed interaction preferences between courses; transfer knowledge claims only when objective correspondence is justified.

Store individual assertions rather than repeatedly rewriting an unconstrained psychological portrait. Distinguish user self-report, observed events/results, and agent hypotheses. Each assertion has its source, date, scope, status and relevant evidence links. The learner can inspect, correct, retire or explicitly request deletion of records. A compact working summary is derived from current records and does not replace their evidence.

Direct requests can update the stated scope without repeated confirmation. Persistent preferences inferred from observations require confirmation before promotion. One successful task does not establish a lasting learning trait. Do not classify the learner into a fixed learning style or treat a questionnaire label as a prescription.

### 8.2 Adaptation and support strategies

Adaptation follows an observable loop: retrieve learner/course context, choose an action, record its reason and policy version, conduct the activity, save performance and optional feedback, then reconsider the next action. The first release uses explicit rules and hypotheses, not an automatically learned optimal policy.

Store changeable adaptation rules as versioned policy records separate from skill instructions. Skills define the workflow and retrieve applicable policies; revising a policy must not require rewriting every skill or the learner profile. Base decisions on explicit learner requests, objective-specific performance, the material's prerequisites and task demands, and the policy's pedagogical evidence. Psychological information informs communication and support; it does not replace evidence of knowledge.

The agent may adjust examples, explanation length, block size and amount of assistance without repeated approval, briefly explaining meaningful changes. Changes to goals or the general course plan require acceptance. If a preferred format appears to conflict with learning results, explain the observation and offer a small experiment; the learner may decline.

A SupportStrategy records the need, evidence, applicable context, proposed action, learner constraints, evaluation method, policy version and trial/retained/retired state. Keep observed performance, comfort, confidence and enjoyment separate. Initially review a strategy after approximately three relevant applications, combining a brief usefulness question with available learning evidence. Review or change it earlier if it is clearly uncomfortable or obstructive; avoid a questionnaire after every action. Three applications are a configurable review trigger, not proof of efficacy. If a strategy is unhelpful, offer another; confirm any inferred lasting preference change. Treat improvement as an observation, not proof of causation.

| Reported need or observed difficulty | Candidate response to try | Evidence to inspect |
|---|---|---|
| Understands now but forgets later | Short retrieval activities distributed across sessions | Delayed independent recall |
| Recalls a rule but cannot apply it | Varied examples and application tasks | Performance on a new task |
| Does not understand the idea yet | Different explanation, prerequisite check, worked example | Explanation/application after support |
| Performs correctly but feels unsure | Show concrete evidence of progress and offer a delayed check | Confidence self-report and later performance separately |
| A large session feels hard to start | Offer one small first action, then a choice to continue or stop | Whether the learner starts and reports the format helpful |
| Fear of mistakes interferes with responding | Offer an unscored draft attempt and focused feedback | Willingness to attempt and independent performance |
| Task feels irrelevant or already known | Connect it to the learner's goal or offer a brief diagnostic | Reported relevance or demonstrated knowledge |
| Low energy/time today | Offer a smaller activity, short review, or rescheduling | Current self-report and agreed scope |
| Concrete agreements help | Agree a starting situation and first action | Learner-reported usefulness and follow-through |

These are candidate product strategies, not guaranteed psychological interventions. When the learner reports procrastination, clarify the nearest obstacle if it is not already known. Do not infer its cause from chat latency or absence. Combine support needs where useful, such as starting with one brief review question when the learner wants more repetition but finds it hard to begin.

### 8.3 Optional questionnaires

Offer questionnaires voluntarily during onboarding or when a specific difficulty makes one useful. They are not repeated for every new source or lesson. Initially focus on learning habits, motivation, difficulty starting, and attitude to mistakes. General personality inventories are deferred. Subject diagnostics are separate from these questionnaires.

The initial preference questionnaire is brief and covers goals, habits, feedback preferences and difficulties; any question may be skipped. For additional questionnaires, research and compare existing instruments for the concrete need before selection. Explain why an instrument is being offered and which decisions its result may inform. Specific instruments have not yet been selected.

Before selecting an instrument, document its source/version, intended purpose and population, permission to use it, questions, scoring where applicable, interpretation limits, and the concrete teaching/support decisions it may inform. A custom preference questionnaire must not be represented as a validated psychological instrument. Store responses and interpretations separately, with authorship and instrument version. Results suggest discussion or a support trial; they do not override the learner's correction or automatically assign a teaching method. Specific instruments remain an implementation choice.

## 9. Notes, Questions, and Relationships

The learner can create a note or question at any time and optionally link it to topics, sources, lessons, or exercises. Preserve the learner's exact text. Assistant comments and suggested corrections have separate authorship and require an explicit action to replace learner text.

Questions use question_status values `open`, `exploring`, or `resolved`; resolution retains the answer and supporting evidence. Closing a question does not imply that every linked learning objective has been demonstrated.

Relationship kinds include `based_on`, `covers`, `prerequisite`, `related_to`, `practices`, and `summarizes`. Store explicit relationships rather than treating shared keywords as proof of a connection. Any traversal must track visited IDs and impose a depth or result limit.

## 10. Visuals and Cheat Sheets

Ownership and derived-view rationale:
[ADR 0005](docs/adr/0005-derived-course-visualizations.md).

Use source diagrams and illustrations with attribution and useful locators. Supplementary visuals must be grounded in the selected material and labeled as assistant-created. Use tables, Mermaid diagrams, annotated examples or animated HTML as appropriate to the material and the learner’s explicit preferences; always provide a readable text explanation if rendering is unavailable.

All learner-specific visual assets (including HTML animations, images and their dependencies) must be retained inside the owning course or article study, in versioned `references/visualizations/` paths. Link them to the corresponding topic, lessons/cheat sheet and source passages; include context in the visual itself. Share the retained course/study path, not a temporary preview or project-level documentation copy. Follow [visual learning assets](docs/learning-operations.md#visual-learning-assets) for publication, navigation, versioning and export checks.

Prepare a lesson cheat sheet together with each lesson and reveal it at the end of that lesson. Its content follows the lesson's scope; if the session ends before all material is covered, keep unstudied sections unrevealed. Include essential rules or formulas, a small example where useful, and relevant pitfalls. Link each section to its supporting lessons or sources. Unresolved misunderstandings may appear as clearly labeled reminders, not accepted facts.

After the topic is completed, provide a final topic cheat sheet as a table of contents: each lesson or subtopic has its main idea and a clickable link to the corresponding detailed cheat sheet. Offer a textual or visual format based on the material. The learner can edit personal additions; preserve their authorship. Updating a sheet must not overwrite those additions silently.

Once revealed, cheat sheets are available outside practice at any time. During a recall task, opening a sheet marks the current attempt as assisted without penalizing the learner.

An optional course map is enabled only for a requested full course. It derives
planned/prepared/covered states and assessed evidence from saved records; it is
not another progress database. Show only revealed reference material, refresh
within the chat workflow after relevant saves, and retain the previous map as
stale if refresh fails. Article-study maps are not implemented. See
[map operation](docs/learning-operations.md#course-maps).

## 11. Persistence and OKF Model

Ownership rationale: [ADR 0002](docs/adr/0002-learning-data-ownership.md).

Use one shared learner bundle, one shared articles bundle (`learning.articles`, `courses/articles/`) for standalone article studies, and one local OKF bundle per full course in v1. An article used as a source within a full course stays in that course. Topics and chapters do not create bundles. Each standalone article has a Study record at `studies/<slug>/study.md`, its own accepted plan, settings and sessions in that folder, and explicit study ownership. Domains are classification labels, not separate records. Full courses keep their existing Course records. Ownership and completion are enforced by the article schema and studies.rs; [prepare-course](skills/prepare-course/SKILL.md) handles selection and creation. Store learning content, learner-model assertions and history as Markdown records with frontmatter; no additional database engine is required. Register nonoverlapping bundle roots in an OKF catalog and explicitly scope reads to the learner and selected course. Runtime caches and compact profile summaries are optional and must be reconstructible from saved records.

Proposed layout:

```text
okf-catalog.yaml
learner/
  index.md
  ontology.yaml
  profile.md
  preferences/
  goals/
  observations/
  hypotheses/
  questionnaires/
  policies/
  support-strategies/
  references/
courses/<course-id>/
  index.md
  ontology.yaml
  course.md
  learner-settings.md
  plans/
  sources/
  topics/
  objectives/
  lessons/
  exercises/
  notes/
  questions/
  cheat-sheets/
  attempts/
  assessments/
  sessions/
  schedules/
  references/
```

Directory names are application conventions, not required OKF semantics. `references/` holds source snapshots and other artifacts where appropriate. Each root index provides navigation. Define custom record types in each bundle ontology; verify exact declarations against the installed version. Use portable relative paths for authored cross-bundle links and catalog IDs for CLI selection; catalog registration does not make authored paths independent of checkout layout.

The following are application field requirements, not additional OKF conformance rules. The built-in status field retains OKF lifecycle values `draft`, `stable`, and `deprecated`; use distinct application fields for learning states.

| Record type | Required application fields in addition to `type` and `title` |
|---|---|
| Course | goal, language, timezone, study days, time budget, review allocation; accepted plan reference once approved |
| CourseSettings | learner reference, course reference, scoped preference overrides and their sources |
| LearningPlan | course, plan level, objectives/activities, plan version, plan_status; acceptance evidence when accepted; optional superseded plan |
| Source | location, format, access status, inspected date when available |
| Topic | source references, related topics, prerequisites |
| Objective | topic, observable success criteria |
| Lesson | objectives, theory references, exercise references, estimated minutes, readiness, version |
| Exercise | objectives, kind, rubric or answer key, source references, version |
| Note | author, created timestamp, related records; body contains original text |
| Question | author, question_status, related records; body contains question and attributed discussion |
| CheatSheet | covered topics, source and lesson references, content version |
| Attempt | submission ID, exercise and version, session, timestamp, assistance; body contains answer |
| Assessment | attempt, assessment_status; outcome, criterion results and assessor when a judgment is available; optional superseded assessment |
| Session | session_status, current lesson and item, start time, completed/skipped item references, temporary context, effective settings and their origins, applied strategy/policy references and reasons |
| Schedule | objective or lesson, due date, review stage where applicable, rule version |
| LearnerProfile | learner identity, references to current goals and confirmed preferences; summary is derived |
| LearnerGoal | goal, scope, goal_status, source, created/updated timestamps |
| LearnerPreference | setting/value, scope, origin, preference_status, source, created/updated timestamps, confirmation evidence when inferred, optional superseded preference |
| LearnerObservation | observation, evidence references, scope, observed timestamp, author |
| LearnerHypothesis | claim, evidence references, scope, hypothesis_status, proposed check, author, timestamp |
| QuestionnaireDefinition | instrument source/version, purpose, questions, optional scoring rule, use/interpretation limits and permitted decision uses |
| QuestionnaireResponse | learner, instrument/version, original answers, timestamp |
| QuestionnaireInterpretation | response reference, author, scoring version if applicable, result, limits and proposed support |
| AdaptationPolicy | policy version, conditions, actions, constraints, evidence/rationale, evaluation method and policy_status |
| SupportStrategy | need, evidence, context, action, constraints, evaluation method, policy reference/version, strategy_status and outcome references |

Record identity comes from its bundle-relative path, qualified by bundle identity when needed. Keep IDs stable; title edits must not rename records automatically. Retain an inspectable exercise and rubric version/snapshot used by each attempt. A version number alone, or an uncommitted Git working tree, does not preserve historical content. Keep historical policy references resolvable as policies change. Avoid renaming history records in v1.

Declare reference fields for OKF relationship inspection. Separate four checks: portable conformance (`validate`), advisory structural/ontology checks (`lint`), semantic review against sources, and learning-specific invariants. Existing ontology checks cover required values, types, allowed values, bounds and typed references; reuse them. The workflow must explicitly gate acceptance/publication on relevant findings and inspect incomplete-check diagnostics. If invalid records must never become active, validate proposed state before activating it; ordinary successful `add`, `edit` or `changeset` calls are insufficient. Learning checks cover prerequisite cycles, transition rules, readiness evidence, and historical-version integrity.

Use the applicable existing OKF skills to guide supported CLI operations such as `init`, `add`, `edit`, `search`, `show`, `links`, `validate`, and `lint`. These are existing OKF operations; course workflows introduced here are not new built-in OKF commands. Follow the skills' compatibility checks and consult the installed CLI schema and help for exact arguments. Custom learning record types and learning-specific checks are implemented by the project schemas and focused runtime; they are not built-in OKF behavior.

### 11.1 Persistence protocol and recovery

Rationale for retained history and guarded publication:
[ADR 0003](docs/adr/0003-immutable-history-and-guarded-publication.md).

Use a single coordinator for course writes and shared-profile changes. Assign and retain a stable submission ID before saving the answer, and derive one stable Attempt path from it. On a repeated save, inspect and compare the existing record before treating it as success; a conflicting payload must not overwrite the original. OKF rejects duplicate paths, but does not enforce uniqueness of a custom submission_id across different paths.

Persist attempts before assessment and derived schedule updates. Use separate records for revised judgments, challenges and supplemental observations. Preserve old judgments while recording resolution/supersession; validate that a replacement refers to the same attempt and that active history is unambiguous. Enforce write-once attempt behavior in the supported workflow, without claiming filesystem-level immutability.

Reuse OKF JSON Patch test guards and changeset plan/apply with expected base digests for coordinated changes. Changesets reject new dangling references and portable nonconformance; they do not run all ontology or learning checks. Publication is journaled per file within one bundle, not an all-files-at-once or cross-bundle transaction. Do not implement a duplicate transaction engine.

If publication is interrupted, inspect and use OKF recovery, which can roll back a recoverable changeset without overwriting intervening edits. Then reconcile the learning operation from saved attempts and resolved assessments and recompute derived state. Never assume recovery itself recalculates progress or resumes teaching. Keep course events authoritative; shared-profile observations derived from them can be retried separately with stable origin links/IDs, so failure of one bundle update does not lose an answer in another.

Never report an answer as saved before persistence succeeds. Local history is retained by default; deletion or export must be an explicit learner action. Export should include the requested records and their relevant provenance, with cross-bundle dependencies made explicit.

## 12. Learning Skills

These are new skills to implement, not existing capabilities of the OKF plugin.

| Skill | Purpose |
|---|---|
| `coordinate-learning` | Inspect saved state, route the learner's request, and resume the appropriate workflow |
| `update-learner-model` | Capture explicit preferences, questionnaire responses, observations and hypotheses; maintain confirmed profile changes and review support strategies |
| `prepare-course` | Inspect sources, identify gaps, and propose objectives and a plan |
| `prepare-lesson` | Produce grounded theory, exercises, and rubrics; prepare the lesson cheat sheet using the make-cheat-sheet workflow; check readiness |
| `run-session` | Present material and tasks, collect answers, guide the session, and reveal the lesson cheat sheet at its end |
| `assess-answer` | Evaluate an attempt and handle assessment challenges |
| `review-learning` | Present scheduled review activities and feedback |
| `capture-note` | Preserve notes and questions and propose relevant links |
| `make-cheat-sheet` | Summarize studied content into concise textual or visual references |

The coordinator selects workflows for the current chat agent; separate subagents are not required for v1. Parallel source research may be considered later, but is not a baseline dependency. Keep one writer per course even if research is later delegated.

The agent following coordinate-learning orchestrates the workflow; there is no separate orchestration model. It reads saved context and versioned policies, selects the next skill workflow, invokes deterministic calculations when needed, and persists results through OKF. After assessment, it can route to a short explanation or readiness conversation and request schedule recalculation using the applicable rule. Record each support-strategy application with a stable event identity; structured queries or deterministic counting supply the review trigger, while the agent discusses usefulness with the learner. The three-application trigger lives in a policy, not duplicated across skill instructions. No orchestration runs while the chat is closed; the next invocation resumes from saved state.

Learning skills specify the educational workflow and direct the agent to existing OKF skills for knowledge-base work:

| Existing skill | Use in a course workflow |
|---|---|
| `okf:init` | Create the shared articles bundle once, or a separate full-course bundle |
| `okf:ontology` | Define local record types, fields, and relationships for the course |
| `okf:migrate` | Transform existing authored documents into grounded bundle records |
| `okf:ingest` | Research repository or mixed-artifact sources and author grounded records |
| `okf:retrieval` | Find and inspect saved course material and its source lineage |
| `okf:update` | Reconcile affected records when sources or concepts change |

Choose the source workflow according to the material; not every course needs both migrate and ingest. These skills are instructions used by the same agent, not functions that automatically invoke one another.

For a new course, prepare-course collects the learning goal and selected materials, uses the appropriate OKF workflows to establish the bundle and grounded source records, and proposes objectives, topics, dependencies, and lessons. The learner reviews the plan before it becomes the accepted course plan. Detailed lessons are prepared progressively after acceptance.

The coordinator retrieves the shared learner model, course overrides, relevant knowledge state and session context through explicitly scoped OKF retrieval. All teaching skills use effective settings and applicable policy versions. prepare-lesson selects appropriate activities and support; run-session records meaningful adaptation reasons; assess-answer contributes objective-specific evidence without personality inferences; review-learning checks retention and strategy outcomes. update-learner-model keeps self-report, observations and hypotheses distinct and confirms inferred permanent changes.

Skills invoke focused learning code where deterministic calculations or enforced state transitions are required. They must not calculate authoritative progress independently or bypass learning-specific record validation. Decisions needed to resume a workflow are persisted, not left only in conversation history.

## 13. Acceptance Scenarios

1. Given an inspectable source and a learning goal, the assistant proposes a sourced plan with objectives, estimated workload, prerequisites, and dates fitting the learner's settings.
2. An inaccessible or conflicting source produces a visible limitation and blocks unsupported content from becoming ready.
3. The learner completes a lesson, submits an answer, receives rubric-based feedback, and can inspect the saved original submission.
4. After restarting the interface, the assistant resumes the saved session without asking the learner to repeat a submitted answer.
5. Retrying a failed save or assessment does not duplicate an attempt; interrupted derived updates can be reconstructed.
6. An assessment challenge preserves the original judgment and records a superseding correction. Scheduling uses only the active resolved assessment.
7. Identical attempt history, settings, and current time produce identical review dates and queue ordering.
8. Missing a study day retains progress and produces a manageable rescheduled queue with deferred work visible.
9. A learner note remains unchanged when the assistant adds feedback or updates linked content.
10. A lesson cheat sheet can be prepared before teaching; the learner-facing sheet reveals only covered content at the end of the lesson, links to sources, and marks reference-assisted practice appropriately.
11. A changed exercise or source does not rewrite the historical meaning of previous attempts.
12. A prerequisite cycle is rejected, while an ordinary relationship cycle can be inspected safely with bounded traversal.
13. A confirmed global preference is available in a new course; a course override changes that course only, and a temporary session request does not rewrite either persistent setting.
14. A new chat retrieves the relevant learner/course context, policy versions and saved position without relying on previous chat history.
15. A user correction retires or supersedes a mistaken profile claim and changes subsequent behavior; an unconfirmed hypothesis is never treated as a confirmed trait.
16. Optional onboarding or difficulty-specific questionnaires can be declined without blocking learning. Instrument responses, interpretations and subject diagnostics remain distinct.
17. The agent adjusts an example or block size within its agreed discretion, but an unaccepted goal/general-plan change does not become active.
18. Additional source discovery occurs only on request; a discovered source does not enter the selected course material before agreement.
19. A learner reporting difficulty starting is offered an appropriate small next step; missed sessions alone do not create a psychological label. Disabled praise or achievements stay disabled.
20. A support trial records its rationale and outcome evidence. Comfort feedback and independent/delayed performance remain distinct; one success does not become a causal claim.
21. A 20-minute session initially allocates approximately 5 minutes to reviews. Overflow stays visible and due dates remain intact without automatic session extension; changed allocations follow learner settings.
22. An invalid application enum or wrong reference type is caught by the workflow's structural gate even though OKF may permit the write. Learning states use custom fields rather than repurposing OKF status.
23. Conflicting active assessments, cross-attempt supersession or a supersession cycle block affected progress updates. A failed guarded write does not silently replace newer state.
24. Interrupted cross-bundle observation updates can be retried without losing the course attempt or duplicating its learner-model observation.
25. The accepted course map remains stable while later lessons adapt to earlier results. Each lesson sheet is prepared alongside its lesson and revealed at the end; the completed topic has a linked contents sheet with main ideas.
26. Multiple immediate retries do not advance an objective through multiple review stages in one session. Unknown assistance is clarified only when needed and cannot silently count as unassisted success.
27. An error or assisted answer leads to brief learner-appropriate review without a mandatory full interval reset; the selected policy and evidence are retained.
28. Unused review allocation can be used for new material. An explicitly skipped prerequisite stays visible without being counted as learned.
29. After approximately three relevant strategy applications, the coordinator offers a short review of usefulness; it may change an obstructive strategy earlier. Any initial questionnaire question can be skipped.
30. Material without a practical-task requirement can proceed through explanation/discussion. After a correct answer with residual uncertainty about understanding, the agent asks whether the learner is ready to continue, honors that choice, and preserves uncertainty separately instead of requiring additional tasks or claiming mastery.

## 14. Implementation Status and Remaining Evaluation

The Rust runtime, learning skills, article isolation, learner-model reconciliation,
optional course maps and shared-book retrieval have implementations and synthetic
checks. Exact command contracts are in [learning contracts](docs/learning-contracts.md);
architectural choices and consequences are in [ADRs](docs/adr/README.md). Accepted
requirements and passing deterministic tests do not prove teaching quality or
observed retention.

Remaining acceptance work is behavioral: review source grounding and rubric quality
against inspected passages; exercise setup and fresh-chat teaching/resume in the
actual host; evaluate learner comfort and later independent recall/application
when those sessions occur. A selected real source, map or successful indexing run
does not by itself complete this evaluation. The walkthrough is maintained in
[learning-flow scenarios](tests/scenarios/learning-flow.md). Test results are reported
for actual runs; they are not maintained as a separate repository validation log.

Before claiming release readiness, account for all 30 acceptance scenarios with
relevant automated evidence or an observed behavioral walkthrough. Record the
instruction version and actual host model identifier when exposed, without guessing.
Simulated time verifies calendar mechanics, not delayed retention. Approximate
estimates, correctable open-answer judgments and unobserved outside assistance
remain limits. Publication is journaled per file with one writer and no cross-bundle
atomicity or filesystem-level immutability. Export/deletion/migration remain
explicit supported procedures, not a background service.

No scored questionnaire instrument has been selected. Research purpose, permitted
use, population, scoring and limits only when a concrete need arises; refusal
must not block learning. New policy/schema versions require explicit migration
and replay checks. Do not reopen implemented review-v1 rules as unresolved choices
or infer new personal preferences from engineering documentation.

## 15. Research Basis and Limits

These sources inform design choices; they do not validate this assistant or establish an optimal personalized method:

- [Dunlosky et al., effective learning techniques](https://pubmed.ncbi.nlm.nih.gov/26173288/): basis for retrieval/distributed practice and context-sensitive use of other techniques.
- [IES, organizing instruction and study](https://ies.ed.gov/ncee/wwc/PracticeGuide/1): worked examples, practice, explanatory questions and learning over time.
- [Microlearning systematic review](https://www.sciencedirect.com/science/article/pii/S2405844024174440): focused short units; heterogeneous settings and no universal optimal duration.
- [Pashler et al., learning styles](https://brucehayes.org/Teaching/papers/PashlerEtAl2008OnLearningStyles.pdf): do not equate a preferred format with a proven fixed learning-style prescription.
- [Open learner models review](https://doi.org/10.1016/j.compedu.2020.103878): context for inspectable learner models and learner participation.
- [Deslauriers et al., actual versus perceived learning](https://doi.org/10.1073/pnas.1821936116): reason to distinguish perceived ease from measured performance, with limits to generalization from the studied context.
- [Cepeda et al., spacing and retention](https://laplab.ucsd.edu/articles/Cepeda%20et%20al%202008_psychsci.pdf): interval usefulness depends on the retention goal; no universal 1–3–7–14 prescription.
- [Academic procrastination and goal accomplishment](https://www.sciencedirect.com/science/article/pii/S1041608017300109): planning techniques are not guaranteed to reduce procrastination; support strategies remain trials to evaluate.

Evaluate the product using independent attempts, delayed recall and application to new tasks, alongside separate reports of usefulness and comfort. Keep evidence provenance and uncertainty visible. Update policies as evidence and learner feedback warrant without rewriting past attempts or silently changing the meaning of historical decisions.

## Learner-model reconciliation implementation

Before teaching or file authoring, load confirmed applicable preferences through
audit-learner-model. After saving learning evidence, reconcile the source-digest
review queue against immutable LearnerModelReview receipts. Each source either
links justified saved assertions or has an explicit no-new-information reason.
Changed sources become pending again; unsaved chat content must first be captured.
Follow [saved-evidence reconciliation](skills/update-learner-model/SKILL.md#reconcile-saved-evidence) for review and incomplete-audit handling; evidence/scope requirements are in section 8.1.
