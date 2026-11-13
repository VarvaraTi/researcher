# Research skills specification

Status: Revised draft for review; incorporates the annotated review and iterative-refinement requirements. This document specifies behavior; it does not install skills or create research bundles.

## 1. Purpose and agreed requirements

Create a reusable family of research skills that works across topics and methods. The skills should help the researcher get oriented, understand problems, evaluate choices, investigate questions, conduct experiments, and develop capabilities. They should preserve useful results across sessions and make the evolution of research and decisions traceable.

Agreed requirements:

- One workspace per research theme, stored as one Git repository, with related projects as subfolders. Each project subfolder is an OKF bundle; projects do not have separate Git repositories.
- Each research project has its own hierarchical OKF bundle, containing knowledge and supporting artifacts.
- A workspace may also contain a shared OKF bundle used by its project bundles. Agents update relevant shared content during authorized research and create or promote shared concepts when reuse across projects is evident, preserving scope and provenance.
- Research concepts support branching, selective forking, cloning, and later reconciliation as ordinary OKF concepts within project bundles. These are research operations, not Git operations.
- Persist substantive research continuously: document ingestion, conversation, and refinement create or update the relevant concepts and related content when warranted, without repeated save prompts. Quick research saves a compact briefing and sources in the active project; separate concepts are created when useful. Git commits remain explicit actions.
- Independent streams can explore competing hypotheses, different objectives, or the same assignment using different models or methods; they remain isolated from sibling conclusions until synthesis. Complementary streams may exchange useful context when their assignments allow it, recording consequential exchanges in their stream concepts.
- Agent recommendations remain proposed until the researcher accepts them. Standing authorization may define what agents can decide within an explicit scope.
- Each project has one chronological journal. Its research overview summarizes current state, while topic and experiment concepts hold their own details. Decisions link to their historical rationale and asset changes.
- Experiments support planning and execution, including human-conducted and non-code activities, repeated sessions, status tracking, observations, and recommendations.

- Delegate bounded work to subagents when useful, matching model capability and cost to the task rather than assuming the starting model is appropriate. Use reasonable defaults within available runtime capabilities; ask only when an explicitly requested model is unavailable or a consequential tradeoff needs the researcher’s choice.
- Extract research-relevant content from text-like documents into the appropriate concepts, with source links and coverage limits. Link to external resources. For other artifacts, ask whether to save them unless the researcher has already specified a preference; reuse that preference without asking again.
- Experiment assessments can use multiple outcome dimensions rather than a single overall label.
- Support iterative refinement from a short initial request through successive questions, with recommendations for useful next directions and complementary parallel inquiries.

- Include concise guides for documentary/historical, interpretive, qualitative, and empirical/comparative research, with specialized guides added gradually as needs emerge.
- Use existing OKF and Git capabilities for persistence, links, copying, validation, and history. Custom helpers are outside the core package and may be developed later for specific projects.
- Start with compact templates and expand records as research develops, preserving minimum provenance and identity requirements.
- Workspace upgrades, format-version management, and migrations are handled by agents outside these research skills. Do not build a versioning or migration framework into the skills.

- Build the complete specified workflow, including research quality and retrieval. Skill improvement is ordinary agent work outside this family. Behavioral evaluation suites and harnesses are outside scope. Staging is an implementation sequence, not a reduction of scope.

### Required persistence architecture

Use OKF for research content and artifact organization, and Git for version history. The working OKF bundle is the current snapshot; historical snapshots are the bundle at identified Git revisions, not separately serialized copies. Store supporting artifacts using OKF artifact conventions, with Git LFS for large files. External artifacts retain explicit references and availability limits.

Record semantic history—observations, sessions, decisions, rationale, corrections, and significant changes—in relevant research records and journal/log files, themselves tracked by Git. An experiment record can contain its current plan, status, and assessment alongside accumulating session and correction entries. Update that record in place; Git preserves its earlier versions. Do not duplicate detailed entries in a central journal when a concise event and link suffice. Preserve OKF's reserved logs for their existing purpose.

Use Git revisions and workspace-relative paths to identify committed historical content. There are no automatic commits: a human or agent commits only as an explicit Git action, never as an implicit step of a research skill. Research may proceed on uncommitted files; identify such content as working state and do not claim that a prior commit includes it. Creating or copying a research concept does not require a commit. Git preserves exact committed file states, not every intermediate edit between commits. Record meaningful rationale and procedure changes in the relevant decision, experiment, or project journal; this semantic history does not promise exact reconstruction of uncommitted intermediate files. Do not add snapshots or force commits to fill that gap. Git LFS pointers alone do not establish artifact availability; required LFS content must also be retained and accessible.

Do not create a parallel object store, snapshot repository, transaction database, or general history layer, including a `.research` directory serving those purposes. Recovery uses Git, existing OKF operations, and proportionate operation-specific progress recorded in ordinary records or reports. The core skills use these existing mechanisms directly; any later project-specific helper must not reimplement version control. Temporary working copies for isolation or recovery are not an additional authoritative store.

The agreed requirements and record boundaries apply throughout this specification. Illustrative filenames, optional resources, and explicitly proposed choices do not override them.

## 2. Design principles

1. Select workflows by intended outcome, not subject matter. Adapt methods to the question rather than forcing every task into hypothesis testing.
2. Keep simple work simple. A quick briefing should not create a dozen empty records.
3. Distinguish observations, sourced claims, interpretations, hypotheses, recommendations, and accepted decisions.
4. Preserve provenance and historical context. A link to a mutable current document alone cannot establish what supported an earlier decision.
5. Maintain a readable current state alongside history. Users should not need to reconstruct the journal to resume work.
6. Treat disagreement and inconclusive results as useful outcomes. Do not manufacture consensus or certainty.
7. Separate execution from knowledge integration. Streams produce results; synthesis determines their implications for shared findings.
8. Apply evidence standards appropriate to the method and stakes. Source quality, uncertainty, and practical constraints should affect conclusions.
9. Match agent and model selection to task complexity, evidence demands, and budget. Use more capable models for complex interpretation or synthesis and less costly models for suitable bounded tasks; do not assume the starting model is the best choice.
10. Treat research direction as revisable. Help the researcher identify what to ask next while preserving the progression from the original overview to the current question.

## 3. Workspace and project organization

### Theme workspace

The theme workspace is one Git repository. Project subfolders each contain an OKF bundle; the optional shared bundle is in the same repository. A simple workspace overview describes the projects and links to their research overviews. Keep detailed research relationships in OKF links/backlinks. Do not create separate workspace, project, artifact, or stream manifests or catalog schemas.

Minimal illustrative layout, not a required directory schema (OKF-managed files are omitted):

```text
theme-workspace/
  README.md                    # workspace overview and links to research overviews
  shared/                      # optional shared OKF bundle, created only when needed
  projects/
    project-a/                 # OKF bundle
      research-overview.md
      journal.md
      topic-a.md
      topic-b.md
```

When starting a new theme workspace and intent and location are clear, initialize the chosen directory as a Git repository if it is not already one and create the simple workspace overview. Inspect the existing repository context first; do not create a nested repository or replace existing content. If the intended repository boundary or location is ambiguous, ask. Initialization does not create a commit.

When research intent and placement are clear and no suitable project exists, create a project subfolder as an OKF bundle using existing OKF capabilities, initialize a research overview and one project journal with the known purpose and starting context, and add the project to the workspace overview. Do not require a separate setup or approval step; ask only if project placement is ambiguous. Do not create a new project when the work fits an existing one, and do not automatically commit.

Create concepts as research needs them. Add subfolders only when useful for navigating accumulated content; do not pre-create type-specific folders or empty records. Store supporting artifacts using the existing OKF conventions. Hypotheses, experiments, decisions, and delegated/parallel streams are ordinary concepts and do not require dedicated directories.

### Shared workspace bundle

The optional shared bundle holds reusable topic concepts and artifacts, with source context recorded through OKF links in the relevant concepts that serve multiple projects. Project bundles can reference shared material or copy relevant concepts from the working bundle or, when requested, a recorded Git revision. Record which relationship applies: a reference to current shared knowledge is distinct from a historical baseline used for a decision or independent stream.

Within authorized workspace research, agents update relevant shared concepts without an additional promotion or update approval. When reuse across projects is evident, create or promote the reusable content into the shared bundle, creating that optional bundle through OKF if needed. Do not create it merely in anticipation of possible reuse. Honor access and copying restrictions and preserve project-specific distinctions.

Shared findings must retain their scope and provenance. A project-specific interpretation does not become universal merely because it is useful elsewhere. Promotion into the shared bundle includes checking scope, dependencies, and conflicts, preserving originating project links, and recording the change. Streams write to their own areas; shared-bundle changes are integrated deliberately rather than concurrently overwritten.

Skills maintain OKF links and backlinks when shared material changes, using the OKF plugin’s existing machine and human verification support. No additional explicit review step or separate review queue is required. Changes do not silently revise accepted decisions or copied research concepts. Copies preserve links to required shared concepts and artifacts; missing or unavailable dependencies remain explicit. The workspace overview remains a simple navigation document; the optional shared bundle holds reusable research content.

Shared disagreements and their rationale live in the shared concept they concern. Record significant shared changes in the initiating project’s journal and summarize their implications in affected project research overviews within the authorized scope, linking to the shared concept rather than duplicating its detailed history. The shared bundle has no separate research overview or journal. If an affected project cannot be updated, record that limitation in the initiating project’s journal.

### Project context

Every active project has a **Research overview**, its living entry point and main working document for project direction. It records purpose, questions, scope, current understanding, unresolved disagreements, active directions, and next steps, with links to topic concepts, experiments, and any delegated or parallel streams. Update it as research progresses and record significant changes in the journal. Detailed research develops directly in topic concepts; routine research does not require a stream or another main working document. Experiments and other research concepts maintain their own detailed current state. Do not create a separate current-state record or duplicate those details in the overview.

- **Research overview:** purpose, questions, intended use, scope, exclusions, audience, constraints, working preferences, research directions, and current disagreements with evidence links. Preserve successive meaningful refinements and their relationship to the original overview.
- **Current summary within the overview:** summarize current findings, accepted decisions, unresolved questions, and next useful actions; link to active hypotheses, streams, experiments, and pending work. Directions and disagreements remain sections of the overview. Detailed status stays in each owning concept.
- **Project journal:** one record per project containing significant research event entries, including changes to directions and disagreements, with links to the records and revisions involved.

### Cross-project relationships

Record explicit relationships such as depends on, extends, contradicts, derived from, and supersedes. These are proposed local research conventions, not assertions about standard OKF edge types.

Use OKF concept identities, links, and backlinks to connect related research, including copied and derived concepts. Skills maintain these relationships through the OKF plugin’s supported operations. Use its existing machine and human verification capabilities; do not build a separate dependency graph, backlink store, or explicit review workflow. Cross-project links follow supported OKF reference conventions; report capability gaps without inventing a parallel linking system.

## 4. Persistent records and artifacts

The following are research record types. Start compact and create records only when needed. Journal events are entries within the single project journal, not separate records. Hypotheses are separate OKF concepts that can be investigated through delegated/parallel streams or tested through experiments. Findings and comparisons are sections within topic concepts, not dedicated record types. Each experiment is one concept; further subdivision is a workspace- or project-specific extension, not part of these skills. Research directions belong in the research overview; project disagreements belong in the overview and their development is recorded in the journal. Shared-bundle disagreements remain in the relevant shared concept, with linked summaries in affected project overviews and significant changes in the initiating project’s journal. Source/evidence references and artifact descriptions use OKF links to concepts, artifacts, or external sources, with relevant context in the referring record; they are not separate note or description concepts. These boundaries also apply to templates and must not be overridden by the general advice to split independently useful research content.

| Record | Minimum useful content |
|---|---|
| Research overview | Question, scope, purpose, constraints, requested output, research directions and their status/rationale, disagreements and supporting links, current summary and links to active work, meaningful revisions and rationale |
| Topic concept | Evolving research on a topic or question: explanations, evidence links, findings, comparisons, and links to separate hypothesis concepts as useful; split only when the content benefits from independent linking or development |
| Hypothesis | Separate concept containing a proposed explanation, rationale, alternatives, distinguishing or weakening evidence, and links to any streams or experiments investigating it |
| Research stream | One concept per delegated or parallel assignment, containing its assignment, baseline, boundaries, method, execution details, progress/status, results, and links to resulting topic concepts |
| Experiment | One concept containing the plan, session/run entries, observations, current status, assessment, and links to hypotheses and supporting artifacts |
| Decision | Separate concept for a consequential proposed or accepted choice: alternatives, rationale, acceptance/rejection, actor, time, evidence links, consequences, and follow-up actions with their status |
| Project journal | One per project, containing accumulating event entries: what happened, when, who reported or performed it, why it matters, related OKF links; include the development of disagreements and direction changes |
| Learning plan | One concept containing target capabilities, starting point, learning sequence, activities, practice, progress, assessments, and adaptations |

### Artifact handling

Handle supplied, consulted, or generated material according to its content and the research task:

- **Text-like documents:** Extract research-relevant substance into existing or new topic concepts, regardless of whether the input is Markdown, a PDF, a word-processing document, or a transcript. Preserve source links, useful passage/page locators, and extraction or coverage limits. Do not recreate the document section by section or imply exhaustive conversion when only relevant content was extracted. Honor explicit requests for reference-only treatment or faithful full conversion.
- **External resources:** Add OKF links from relevant concepts to the external sources, with context supporting the claims made. Extract relevant substance when used in the research, but linking does not require downloading or archiving the resource.
- **Other artifacts:** For images, audio, video, datasets, executable files, instrument outputs, and other non-text-like material, ask whether to save a project copy unless an existing instruction or project preference already answers that question. Batch the choice when useful and do not ask again for an established preference. If saving is requested, use OKF artifact conventions and Git LFS for large files; otherwise retain an available external reference and identify availability limits.

For experiment-generated outputs, establish retention preferences during planning and apply them across runs without repeated prompts. Include expected data, recordings, analysis code, and instrument outputs where relevant. Distinguish temporary working output from artifacts retained in the project. If an unexpected output is not covered by the preference, clarify its retention only when needed; do not claim unsaved output is a retained artifact.

Content, not the filename extension alone, determines whether text extraction applies. A notebook may contain both extractable discussion and executable/data artifacts. If the relevant text cannot be extracted, report the gap rather than claim ingestion succeeded. An instruction to ingest documents authorizes extracting their research-relevant content without an additional ingestion/reference prompt. For documents supplied for ingestion, retain the original as the source artifact using supported OKF artifact conventions, reusing an existing copy where available and honoring explicit retention/access restrictions. Merely consulting an external source does not require downloading it.

Link to artifacts using OKF references from the relevant research record. Include necessary origin, purpose, version, limitations, and transformation context there or in supported link metadata; do not create separate artifact-description concepts. Preserve raw observations and original data separately from cleaned, transformed, or interpreted versions when those artifacts are saved. Document consequential transformations.

Ingest research-relevant document content into substantive OKF concepts and retain only the original document as its source artifact in the bundle. Do not retain duplicate extracted-text artifacts (`.txt`, `.md`, OCR sidecars, or full-text transcriptions), or disguise them as source-note concepts. Any needed intermediate extraction from PDFs or other materials must be temporary structured Markdown (`.md`) outside the bundle, not plain-text (`.txt`). If a tool emits plain text, structure it as Markdown before saving or handing it off; renaming the extension alone is insufficient. Preserve meaningful headings, paragraphs, lists, tables, quotations, and code blocks where supported by the source, alongside source identity, page/section locators, and extraction or OCR limits. Keep temporary extracts outside the bundle and integrate retained research text into the appropriate OKF concepts without creating separate source-note records. Cite the original artifact from concepts, not temporary extracts. Before reporting ingestion complete, check saved concept content and source links, original-artifact availability or recorded limits, and absence of retained extraction sidecars. Remove temporary files created for the ingestion when finished without deleting unrelated or pre-existing files. Carry these requirements into delegated ingestion assignments.

Markdown saved as concept content must conform to OKF. Use existing OKF ingestion/conversion capabilities to create or update concepts; do not place unconverted source Markdown into the bundle as if it were a valid concept. Original artifacts that are saved retain appropriate native formats.

Record large or externally held assets through OKF links and availability context in the relevant research records, without a separate manifest. Do not claim a clone is self-contained if required artifacts remain external or unavailable. Inspection of executable artifacts does not itself authorize execution.

### Evidence and citation contract

Represent sources and evidence through OKF links to other concepts, artifacts, or external sources, with supporting context in the referring research record rather than separate source/evidence-note records. Consequential claims must link to evidence that supports the actual claim, using stable locators where possible: page and edition, passage or section, interview/session entry, dataset version and rows, or experiment run. Distinguish original evidence, secondary interpretation, and agent inference. A citation is not evidence of verification unless the relevant source content was inspected; label unavailable, partially inspected, or secondhand material explicitly.

Record publication/collection date separately from retrieval date when relevant. Preserve source identity across mirrors, repeated citations, and derivative summaries so one underlying source is not counted as independent corroboration. When sources are corrected or withdrawn, use OKF links/backlinks and its verification support to identify and update affected claims within the active task; no separate explicit review workflow is required. Search snippets and generated summaries are discovery aids unless their limited evidential role is explicitly appropriate to the question.

For sustained inquiry, retain a proportionate search record: coverage, relevant queries or collections, inclusion/exclusion reasons, and material access gaps. Do not claim exhaustive coverage without a defined search scope and method. Quick briefings need only concise coverage limits. Confidence should have a reason tied to evidence quality and applicability; numerical confidence is optional and must not imply calibration that has not been established.

### Disagreement and uncertainty

Preserve material alternative explanations or readings through summarization, retrieval, promotion to shared knowledge, and ordinary research updates. Record current project disagreements in the research overview; for shared knowledge, keep the detailed disagreement in its shared concept and link summaries from affected project overviews. Capture: competing claims or readings, assumptions, evidence links, scope, whether the disagreement is factual, methodological, interpretive, or preference-based, and what could clarify it. Add their emergence, changes, and resolution to the journal as event entries. Do not create separate disagreement concepts. An unresolved dispute can be a durable research result rather than a failure to finish. A current summary must not make a contested finding look settled merely because it omits the disagreement recorded in the overview.

### Sensitive materials and permitted reuse

Honor project-specific access, retention, consent, redaction, and copying constraints when present. Record restrictions on relevant artifacts and carry them through sharing, cloning, delegation, and promotion to a shared bundle. Do not turn restricted participant material into reusable examples without appropriate authorization; use sanitized or synthetic examples instead.

Metadata labels do not themselves enforce access control. Execution adapters must respect actual storage and tool permissions and disclose enforcement limitations. Preserve the original separately from redacted derivatives when permitted. If an authorized deletion is required, distinguish it from ordinary append-oriented history and account for retained copies and backups; do not promise erasure that tools cannot verify. These are operational requirements, not assertions of compliance with a particular legal regime.

## 5. History, decisions, and asset changes

### Journal and historical evidence

Historical answers use available Git revisions and semantic entries in research records. Distinguish recorded rationale or procedure changes from an exact file state; if the requested intermediate state was never committed and cannot be reconstructed from the records, say so. No additional snapshot mechanism or automatic commit is required.

There is one journal concept per project, not one per topic or stream. Experiment observations and session entries stay inside the experiment concept; the project journal records only significant milestones and links to that concept, without duplicating the detailed entries. The project journal is a record containing event entries; events are not separate records or concepts. Use an append-oriented semantic journal: retain earlier events and correct them through explicit correction or supersession entries. Record meaningful progress rather than every tool call. The research journal is an application convention and must not overwrite or misuse OKF's reserved log structure.

A reference to committed history identifies the relevant record or artifact using its workspace-relative path and Git revision. Current research links may refer to working concepts without a commit; do not describe them as immutable historical evidence. Git tracks research records and OKF artifacts; Git LFS stores large artifact content. Preserve consequential external source content as an OKF artifact when appropriate and authorized, or record the external version identifier and preservation limitations. A fingerprint identifies content but does not preserve it. Do not maintain separate historical copies or snapshot objects alongside Git.

### Decision lifecycle

Consequential decisions are separate OKF concepts. Keep rationale, alternatives, acceptance, impacts, and follow-up actions within the decision concept; do not create a separate impact-list record. Ordinary research choices belong in the research overview and project journal.

1. Record a recommendation as proposed, with alternatives and evidence.
2. Record the researcher's acceptance, rejection, or request for revision. Identify the actual actor and any standing authority used.
3. On acceptance, record impacts and follow-up actions within the decision concept: assets to update, unresolved implications, and remaining work.
4. Apply authorized changes and link each material revision to the decision.
5. Track remaining changes as pending, blocked, or completed. Acceptance and implementation are distinct states.
6. For a substantive reversal of an accepted choice, create a new decision concept linked to the original as superseding it, retaining the original rationale, evidence, and acceptance history. If the reversal is still proposed, keep the original choice current until acceptance or applicable standing authority. Corrections, clarifications, and implementation-progress updates stay in the original decision concept; they do not create a new decision.

Routine note-taking and evidence-backed research updates do not require a new decision approval. When new evidence contradicts existing content, update affected concepts within the active research scope immediately, recording the evidence, conflict, rationale, and remaining uncertainty. Update the overview and project journal when significant; maintain OKF links/backlinks and verification status. Do not wait for a separate review request or invent certainty where the conflict is unresolved. Accepted decisions retain their accepted choice until the researcher reconsiders them or standing authority applies; record the new evidence and need for reconsideration within the decision without silently reversing it.

Journal entries, decisions, and revisions should allow navigation in both directions: from a decision to its evidence and changes, and from an asset revision to the event or decision that caused it.

## 6. Shared operating contract

Every skill should:

1. Resolve the active workspace, project, and question from context. Initialize a missing theme workspace and its Git repository/overview when intent and location are clear, without committing. If no suitable project exists and intent and placement are clear, initialize the project bundle, research overview, and project journal and link it from the workspace overview. Ask only if ambiguity could place or change material in the wrong project.
2. Read the relevant research overview, including its current summary, and retrieve supporting concepts progressively through the appropriate OKF workflow. Use existing OKF validation where needed. Do not perform package/template compatibility checks or maintain workspace, project, or record-format version markers; upgrades and migrations are outside these skills.
3. Establish only missing constraints that materially affect the result. Depth, interaction style, and execution authority are separate settings.
4. Select useful next questions and an appropriate execution arrangement when needed, then perform its own bounded workflow, preserving evidence and uncertainty. Use the refinement protocol when the purpose or direction is evolving.
5. Persist substantive results during the research conversation: create or update the appropriate concepts, reconcile related concepts when warranted, maintain OKF links/backlinks, and update the research overview and project journal for meaningful changes. Do not wait for a separate save request.
6. Report the outcome, material limitations, persisted results, and useful next action without requiring another workflow unnecessarily.

Budgets can include time, cost, breadth, iterations, or user attention. If a boundary is reached, preserve partial results and state what remains unresolved. Do not treat exhaustion of a budget as proof that a question is answered.

### Delegated execution boundary

The shared persistence contract is scoped to the assignment during delegated or parallel execution. A delegated agent writes only to its own stream concept and explicitly assigned concepts/artifacts. It does not independently update the project overview, project journal, related unassigned concepts, or shared-bundle content. Retain proposed changes for those records within the stream’s results. The coordinator integrates results, reconciles related research and shared content, and updates the overview and journal after integration. Independent results remain isolated until synthesis; permitted complementary exchanges do not grant broader write ownership.

### Skill selection and handoffs

Agents select and combine the relevant research skills within the same conversation, preserving the active project, research context, and authorization. A “handoff” in the skill descriptions means continuing the work through the appropriate workflow; it does not require the researcher to invoke another skill or repeat the request. Continue already authorized work without an additional confirmation step. Ask only when a material question about direction, scope, or authority needs researcher input. A handoff does not authorize optional follow-up research beyond the existing scope.

### Continuous persistence during research

Document ingestion and conversation form one ongoing research workflow. A researcher can supply documents, discuss them, ask follow-up questions, propose hypotheses, correct an interpretation, and refine the research direction. Agents must carry substantive results into the OKF bundle as the conversation progresses, rather than leaving them only in chat or requiring a separate update workflow.

Before creating a concept, retrieve the relevant existing content. Update topic concepts with new explanations, evidence, findings, and comparisons; create new topic or hypothesis concepts when warranted by the record model. Update experiment or decision concepts when the conversation concerns them. Follow relevant OKF links/backlinks to reconcile related content within the active research scope, preserving provenance, uncertainty, disagreements, and the distinction between proposals and accepted decisions. Maintain the overview’s summary and direction, and append significant developments to the project journal without duplicating detailed records.

Saving these changes is part of the authorized research task and does not require repeated confirmation. Honor explicit requests to keep discussion out of the records. Casual exchanges with no substantive research value need not be saved, and no verbatim conversation log is required. Apply the document-handling contract: extract relevant text, link external resources, and ask about saving other artifacts only when no existing preference applies. Ask about other material constraints only when unresolved. Do not infer acceptance of a recommendation from persistence, and do not automatically commit to Git.

Report material persisted changes concisely. If relevant content cannot be updated because it is inaccessible, outside the authorized scope, or unresolved, identify that limitation rather than implying the bundle is fully reconciled.

### Research sufficiency and stopping

Set a proportionate completion criterion based on purpose, alongside the execution budget. Orientation requires enough grounded context to address the initial question; understanding requires a usable explanation with unresolved gaps visible; interpretation requires a defensible reading with relevant alternatives; decision support requires explicit tradeoffs and sensitivity to remaining uncertainty; experiment planning requires an executable or human-actionable procedure.

At a natural checkpoint, explain whether further work is likely to change the conclusion, improve understanding materially, or merely add detail. Recommend continuing, narrowing, deferring, or stopping with reasons. Do not confuse a budget limit with evidential sufficiency, or persistent disagreement with a need to research indefinitely. Learning progress requires evidence appropriate to the target capability, not just completion of reading.

### Retrieval and reuse across accumulated research

Support explicit requests such as “Have I researched this before?”, “What currently applies?”, “What did I believe when I made that decision?”, and “Where do these projects disagree?” This is a project-management mode available to every research workflow, built on OKF retrieval rather than a separate competing storage system.

Resolve current versus historical intent; use the workspace overview to locate relevant research overviews and search their bundles and the optional shared bundle within authorized scope; retrieve progressively; follow source lineage and supersession; and return an answer with supporting record/version links, dispute status, and coverage limits. Do not silently answer a historical question with today's revised conclusions. Missing or inaccessible bundles must be reported as coverage gaps, not evidence that nothing exists.

Identify likely duplicates or overlapping findings without automatically merging them. Distinguish a reusable finding from a project-specific assumption. Generated indexes or search caches are derived and rebuildable; canonical concepts and artifacts remain authoritative. Start with existing OKF search and bounded cross-bundle traversal; optional semantic search is a later implementation choice, not a prerequisite for this workflow.

### Iterative refinement and direction selection

Refinement is available within every research workflow and through a dedicated skill. It develops the question, scope, and understanding over successive exchanges; updating research reconciles findings with changed evidence, while challenging research tests their strength. A single user request may require more than one of these operations without restarting the project.

A refinement cycle should:

1. Retrieve the current overview and relevant findings; retain the original intent and unresolved questions.
2. Interpret the new question or observation: does it deepen the same inquiry, introduce a new lens, narrow scope, reveal a prerequisite, or open a distinct project?
3. Answer what can be answered within the requested scope. Ask a few focused questions only when they materially determine the next direction; do not turn every follow-up into an intake interview.
4. When direction is uncertain, propose a small, ranked set of follow-up paths. Explain the question each addresses, why it matters now, expected contribution, needed sources or prerequisites, approximate effort, and whether it can proceed independently.
5. Recommend a path or combination based on the researcher's purpose, unresolved gaps, and dependencies. Distinguish necessary background from optional enrichment. Research value can mean interpretive depth or better understanding, not only uncertainty reduction or a decision.
6. Follow the user's selected direction, or proceed within previously delegated scope. A broad overview alone does not authorize unbounded expansion into every possible path. Record deferred or rejected directions without treating them as disproven.
7. Save the refined overview, substantive answers, and reason for the direction change; update current state and reconnect results to the original question. Keep minor conversational clarifications lightweight.

Selecting a follow-up question need not create a formal decision record. Use the project journal for ordinary direction choices, and a decision record when the choice materially commits resources, changes objectives, or affects established assets. Completion of a refinement cycle means a useful next step or a sufficiently developed answer, not permanent closure of the topic.

### Illustrative refinement scenario: interpreting an elegy

This is a hypothetical workflow example, not a researched interpretation of a particular poem.

- Begin with a short initial request: understand how a specified Victorian elegy presents grief. Establish the text or passage when needed and the reader's starting knowledge.
- Produce an initial reading grounded in the text and identify uncertainties that affect comprehension.
- Suggest complementary directions: historical word usage and syntax; form, imagery, and genre; the author's documented life and composition context; mourning practices or religious context; relevant critical interpretations.
- Explain priorities. Unclear language may need attention before broader interpretation; biographical context and formal analysis may then proceed independently from the same textual baseline.
- Define bounded assignments. A language stream explains difficult passages; a biography stream gathers documented context; a close-reading stream examines how form and imagery shape possible meanings. Do not assume that biography determines meaning or that the poem's speaker is identical to the author.
- Synthesize what each lens contributes, distinguish documented context from interpretation, and preserve plausible alternative readings. The goal can be a richer interpretation rather than choosing a winning stream.
- Refine the overview from the researcher's next question, such as focusing on tension between private grief and public conventions, retaining links to the passages and contextual evidence that prompted it.

## 7. Skill specifications

Names below are proposed invocation names. Agents select and combine these skills according to the shared operating contract; handoffs do not require separate user invocations. Shared conventions should have a single maintained source in the skill package, loaded when needed.

### Supporting-file conventions and assumptions

The following are proposed package resources, not files already created. They are distinct from the research records and deliverables produced in a project. Paths under `references/` and `assets/` are illustrative and relative to the owning skill unless identified as shared.

- **References** contain conditional procedures, method-specific guidance, and worked examples loaded only when needed. Keep short essential instructions in `SKILL.md`; do not create references merely to restate them.
- **Assets** are adaptable templates copied or used to generate project records. Concept templates must be rendered with valid OKF metadata before becoming bundle content; they are not empty concepts to create automatically.
- **Scripts:** no custom helper scripts are required or included in the core research package. Use existing OKF/Git operations; project-specific extensions may be developed separately later.
- **Shared resources** have one canonical maintained source in the plugin. Resolve package-relative access during implementation and test installed portability. Skills load only relevant shared sections. Independently distributing each skill is not assumed.
- **UI metadata:** each skill may include `agents/openai.yaml` for its name, short description, and invocation prompt. This is interface metadata, not an agent execution specification.

Recommended shared references cover record and provenance conventions, evidence and uncertainty, document extraction, external links, and artifact-saving preferences, historical references and lineage, journal and overview-summary updates, and capability-aware delegation. Shared assets cover the project overview with its current summary (including directions and disagreements), a journal with an event-entry format, and the decision concept with its impacts and follow-up actions. Show source/evidence and artifact links within these templates; do not provide separate source-note, artifact-description, journal-event, research-direction, or disagreement record templates. Specialized templates below reuse those conventions rather than duplicate them. Briefings, understanding maps, investigation plans, evidence maps, and critiques are section-level guidance within topic concepts, not additional record types. Agents may produce separate reports when requested, choosing the form for the task without prescribed report templates.

Use existing OKF operations for concepts, links/backlinks, provenance, artifact handling, and validation, and Git/Git LFS for history. Native agent tools handle delegated work. Do not build custom transfer, reference-resolution, history, transaction, or recovery helpers as part of this package. If a project later needs additional tooling, develop it separately as a project-specific extension.

**Confirmed direction:** use compact, adaptable templates that expand as research develops; initial guides for the four agreed method families; and existing OKF/Git capabilities for structural operations. Add specialized guides gradually as demonstrated needs emerge. Markdown concept templates remain the proposed format. No dashboard, external service, scheduler, or standalone execution platform is assumed.


### 7.1 `explore-topic`

- **Trigger:** Get a quick orientation, summary, or initial answer about a topic.
- **Workflow:** Interpret the question; bound the search; gather and compare relevant evidence; explain principal findings and uncertainties.
- **Output:** Compact briefing with sources, important caveats, and open questions in the active project.
- **Handoff:** Refinement to select promising follow-up questions; understanding, investigation, or decision support when requested or when the question exceeds the agreed scope.
- **Stop:** The bounded question is adequately addressed or the research budget is reached. Clarification is exceptional when reasonable assumptions suffice.
- **Supporting files:** Shared evidence and persistence references, with optional section-level guidance for a briefing within the relevant topic concept: answer, evidence, uncertainties, and follow-up directions. Do not create a separate briefing record type or report template. No dedicated script is proposed.
- **Options:** Keep the skill otherwise self-contained (recommended), or add `references/search-strategies.md` if recurring search patterns justify guidance beyond normal research capability. Do not mandate a search log for every quick answer.

### 7.2 `understand-problem`

- **Trigger:** Explain or decompose a problem at the researcher's level; build a usable mental model.
- **Workflow:** Establish prior knowledge and desired understanding; map components, mechanisms, assumptions, and competing explanations; use examples and check understanding when interactive.
- **Output:** Explanation or problem map, vocabulary and assumptions as needed, remaining knowledge gaps.
- **Handoff:** Refinement for deeper questions or additional interpretive lenses; investigation for unresolved evidential questions, decision support for choices, or learning plan for sustained development.
- **Stop:** The requested concepts and relationships are explained at the appropriate level. Do not claim mastery without evidence of understanding.
- **Supporting files:** `references/explanation-modes.md` for conceptual decomposition, causal explanation, and close reading when their workflows differ; section-level guidance for concepts, relationships, assumptions, and gaps within the relevant topic concept, not a separate understanding-map record or report template. No dedicated script is proposed.
- **Options:** Use prose and tables by default; include an optional diagram scaffold for complex relationships. Keep relevant learning preferences and starting knowledge in the learning plan or research overview when useful. Do not create a separate learner profile. Scope this context to the project unless broader reuse is requested.

### 7.3 `investigate-question`

- **Trigger:** Conduct sustained inquiry, evaluate a hypothesis, or determine what evidence would resolve a question.
- **Workflow:** Review existing knowledge; choose an appropriate method; formulate questions and create or update separate concepts for competing hypotheses; gather and assess evidence; revise the working explanation.
- **Output:** Updated topic concepts containing the investigation, findings, evidence gaps, and optional experiment proposals; update the research overview and journal where warranted. Do not create a stream for ordinary ongoing research.
- **Handoff:** Experiment management for new observations; independent streams for competing approaches; challenge for critical review.
- **Stop:** The inquiry supports a scoped conclusion, identifies an evidence barrier, or reaches its budget. Inconclusive is valid.
- **Supporting files:** `references/method-selection.md` to route among documentary/historical, interpretive, qualitative, and empirical/comparative inquiry; short method-specific references loaded selectively. Section-level guidance structures questions, approach, source links, rival interpretations, and evidence gaps within topic concepts. Do not create separate investigation-plan or evidence-map record types or report templates; hypotheses remain separate concepts under the record model.
- **Options:** Include concise guides for all four method families in the first version, with a method selector that can incorporate specialized guides gradually. Specialized additions should declare their scope and reuse shared evidence conventions. Analysis code is created for a concrete project when needed, not bundled as a universal research script.

### 7.4 `compare-and-decide`

- **Trigger:** Compare options, tradeoffs, hypotheses, or stream results in support of a decision.
- **Workflow:** Establish the decision, alternatives, criteria, and constraints; assess comparability; evaluate evidence and sensitivity to assumptions; recommend a choice or conditional choices.
- **Output:** Comparison sections in the relevant topic concepts and a proposed decision, including rationale, uncertainties, and conditions that could change the recommendation. Agents may produce comparison reports when useful or requested, choosing the form for the task; reports are not a prescribed record type or hardcoded skill output.
- **Handoff:** Record acceptance and impacts when supplied; request targeted research if a consequential gap prevents a recommendation.
- **Stop:** A reviewable recommendation or explicit account of why the evidence cannot yet distinguish options. Never infer acceptance from silence.
- **Supporting files:** `references/comparison-methods.md` for qualitative tradeoffs, evidence comparability, explicit weighting, and sensitivity analysis; the shared decision template including impacts and follow-up actions where relevant. Do not prescribe a comparison-report template or schema; comparisons remain sections in topic concepts, and any report format is chosen for the task.
- **Options:** Use project-specific calculations where needed; custom calculation helpers are later project-specific extensions, not core package resources. Qualitative comparison remains the default when numbers would suggest unsupported precision.

### 7.5 `manage-experiment`

- **Trigger:** Plan, conduct, log, resume, or assess an experiment or structured observational activity.
- **Workflow:** Identify the activity and session; establish or retrieve the plan; conduct authorized work or guide human work; record observations and deviations; assess accumulated results; recommend next steps.
- **Output:** One updated experiment concept containing its plan, session entries, observations, status, and evolving assessment, with links to supporting artifacts and any hypothesis concept. Add significant milestones to the project journal.
- **Handoff:** Findings to investigation; recommendations to decision support; new versions of the procedure when changes are accepted or authorized.
- **Stop:** The requested session or update is complete, the experiment's stop criterion is reached, or further work requires unavailable input or authorization. A single journal update must not automatically launch a new experiment.
- **Supporting files:** `references/experiment-modes.md` for agent-executed, human-conducted, and observational activities; one compact `assets/experiment.md` template covering plan, session entries, status, and assessment within the same concept. Do not create separate plan, session, outcome, or experiment-journal records. Further structure belongs to explicit workspace- or project-specific extensions.
- **Options:** Add selective protocol templates for interviews, observation, usability trials, computational runs, or practice exercises as needed. Start with a method-neutral core plus a few contrasting examples; do not impose controlled-experiment fields on interpretive or exploratory work.

### 7.6 `build-learning-plan`

- **Trigger:** Turn research into a personal education or training program, or revise an existing program based on progress.
- **Workflow:** Define target capabilities, starting knowledge, time and resource constraints; identify prerequisites; sequence learning and practice; define assessments and adaptation points.
- **Output:** One learning-plan concept containing objectives, resources, activities, exercises, milestones, progress, assessments, and links to relevant research. Update this concept as learning progresses; additional structure is a workspace-specific extension outside the plugin.
- **Handoff:** Understanding for difficult concepts; experiments for structured practice or testing a learning approach; update when research or goals change.
- **Stop:** The plan is usable within the researcher's constraints, or the requested progress review is complete. Completing material is not automatically evidence of competence.
- **Supporting files:** `references/learning-design.md` for prerequisites, practice, evidence of competence, and progress-based revision; one compact `assets/learning-plan.md` template with sections for objectives, milestones, activities, progress, assessments, and adaptations. Do not create separate progress-review records or templates. Reuse existing research references rather than bundling a fixed course catalog.
- **Options:** Choose a lightweight milestone plan (recommended default) or a more detailed session schedule when requested. Calendar export or scheduling helpers are optional future capabilities, not required infrastructure.

### 7.7 `challenge-research`

- **Trigger:** Critique conclusions, assumptions, evidence, or reasoning; actively test whether research holds up.
- **Workflow:** Identify consequential claims; examine provenance and inferential steps; seek counterevidence and alternative explanations; assess which conclusions survive and under what scope.
- **Output:** Critique sections within relevant topic concepts, linked to the claims examined, with severity and implications of weaknesses, surviving claims, and recommended follow-up. If conducted as delegated or parallel work, keep assignment progress and results in its stream concept and integrate the substantive critique into topic concepts.
- **Handoff:** Update for warranted revisions; investigation or experiments for unresolved challenges.
- **Stop:** The agreed claims have been assessed or the budget is reached. Avoid contrarianism without evidence and automatic rewriting of accepted decisions.
- **Supporting files:** `references/challenge-lenses.md` with method-appropriate checks for provenance, inferential gaps, causal claims, interpretation, and counterevidence; section-level guidance linking each issue to a claim, evidence, consequence, and possible remedy within the relevant topic concept. Do not prescribe a separate critique record type or report template.
- **Options:** Use a direct critical review by default, or an independent review assignment using the coordination skill's stream template. No separate delegation machinery or generic automatic “truth score” is proposed.

### 7.8 `update-research`

- **Trigger:** New evidence, changed conditions, corrected errors, stale sources, or a request to refresh existing work.
- **Workflow:** Identify changes; assess affected findings and assets; reconcile meaning; revise grounded content; maintain OKF links/backlinks and verification status for affected content; identify accepted decisions needing reconsideration without adding a separate review queue; record the change and unresolved findings.
- **Output:** Updated topic concepts and research overview, entries in the existing project journal, and impacts or unresolved gaps recorded in the affected concepts. Keep decision-related follow-up actions inside the decision concept, without a separate impact-list record.
- **Handoff:** Decision support when an accepted choice needs reconsideration; reconciliation for inherited material in other projects.
- **Stop:** In-scope changes are reconciled or outstanding dependencies are explicitly recorded. Refreshing metadata alone is not substantive review.
- **Supporting files:** `references/reconciliation.md` for evidence changes, meaning changes, verification consequences, and shared-bundle dependencies. Record affected findings, decisions, revisions, and unresolved work in their owning concepts and significant changes in the project journal; no separate change-assessment record or template is required. Use existing OKF update, link/backlink, and verification capabilities; do not add duplicate helpers.
- **Options:** User-triggered refresh is the baseline. Continuous in-conversation updates are required by the shared persistence contract; this refresh trigger is for revisiting existing research beyond the active discussion. An explicit monitoring or scheduled-refresh mode could be a later project-specific extension; do not assume a persistent background service. The semantic decision to revise a claim remains agent/researcher work.

### 7.9 `coordinate-research`

- **Trigger:** Split research into independent streams, use different models or methods, resume distributed work, or synthesize its outputs.
- **Workflow:** Define the common baseline and stream assignments; select execution arrangements; dispatch authorized work; track progress and limits; collect results; synthesize with disagreements and shared-source dependencies visible.
- **Output:** One updated concept per stream containing assignment, progress, results, and execution references; synthesis updates relevant topic concepts. Summarize the arrangement in the research overview when useful, without requiring a separate coordination-plan record.
- **Handoff:** Compare/decide for a choice; investigation for remaining questions; update for changes to shared findings.
- **Stop:** Assigned streams have usable outputs and synthesis is complete, or failures and incomplete work are explicitly preserved. A failed stream must not be represented as completed.
- **Supporting files:** `references/stream-patterns.md` and `references/execution-arrangements.md` for independence, complementary lenses, capability checks, and resume behavior; one compact `assets/stream.md` template; synthesis updates the relevant topic concepts without a prescribed comparison-report template. Keep each stream’s assignment, baseline references, progress, results, and links to resulting topic concepts in the same OKF concept. Do not create separate assignment/result concepts, manifests, or schemas. Additional stream-related concepts are workspace-specific extensions outside the plugin.
- **Options:** Start with native agent/task tools plus ordinary OKF stream records; keep custom dispatch/resume tooling outside the core package as a possible later project-specific extension. Keep model preferences configurable and capability information current rather than embedding fixed model rankings.

### 7.10 `manage-research-project`

- **Trigger:** Initialize or resume a research project, retrieve current or historical knowledge across projects, organize relationships, branch/fork/clone material, or reconcile project lineage.
- **Workflow:** Resolve the requested operation or retrieval intent; inspect scope and dependencies; retrieve and compare relevant current/historical records when requested; preserve historical baseline and ancestry; initialize a missing project when research intent and placement are clear; perform authorized changes; check references and artifacts; update the workspace overview and the project overview’s current summary as needed.
- **Output:** Usable project context, copied concepts and linked artifacts where applicable, origin links and explanations within the copies, and a clear resumption summary.
- **Handoff:** Any research workflow using the resulting project context.
- **Stop:** The operation is complete and its references are checked, or missing material and unresolved semantic conflicts are recorded.
- **Supporting files:** `references/project-operations.md` for initialization, shared knowledge, branch/fork/clone, and semantic reconciliation, plus `references/retrieval-and-reuse.md` for cross-project and historical queries; the shared overview template with its current-summary section. Keep origin links, provenance, and a short explanation inside each copied concept; record significant copies in the project journal. Do not create transfer-plan or lineage-record templates or separate records. Keep the workspace overview simple. Use existing OKF operations for copying and integrity checks; no catalog or lineage schema is required.
- **Options:** Use existing OKF/Git operations for copies and history; no custom transfer or recovery helpers are included. Git is the required version-history mechanism. Research branches, forks, and clones are ordinary OKF concepts linked to their origins; they do not create Git branches, worktrees, repositories, or separate project identities unless independently requested. A dashboard or general-purpose project server is outside the initial scope.

### 7.11 `refine-research`

- **Trigger:** Develop a short initial request, follow up on prior research, clarify an evolving question, or choose what to investigate next.
- **Workflow:** Recover the current overview and relevant evidence; incorporate the follow-up; identify useful layers or directions; recommend priorities and sequential or parallel arrangements; pursue the selected or already authorized scope; record the resulting refinement.
- **Output:** Refined overview, grounded answers where requested, a direction section in the overview with rationale and dependencies, and an entry in the existing journal for meaningful changes. Directions and disagreements remain within the overview, not separate concepts.
- **Handoff:** Understanding for explanations; investigation for deeper inquiry; coordination for independent or complementary streams; experiments for new observations; project management when a direction merits a fork.
- **Stop:** The current follow-up is answered, a useful refined overview or direction recommendation is ready, or a material choice requires researcher input. Do not expand indefinitely or create streams for every possible lens.
- **Supporting files:** `references/direction-selection.md` for deepening, narrowing, prerequisites, complementary lenses, and research-value judgments; `assets/overview-refinement.md` with sections for candidate paths, dependencies, selected/deferred status, disagreements, and meaningful changes; this updates the overview rather than creating direction records. Include the hypothetical elegy walkthrough in the reference as a concrete example alongside a contrasting practical inquiry.
- **Options:** Keep directions as sections of the research overview. A selected direction may develop topic concepts or lead to an experiment concept, but the direction itself remains part of the overview. No dedicated script is needed: use existing OKF operations to update the journal and relevant concepts.

## 8. Research stream protocol

Streams are reserved for bounded delegated or parallel assignments. They track execution and results of that work; they are not the main document for an ongoing research project. Ordinary research develops directly in topic concepts, with the research overview providing direction and the current summary.

### Supported patterns

- **Replication:** Same assignment and baseline, independently explored, optionally using different models or methods.
- **Competing hypotheses:** Different explanations tested against discriminating evidence.
- **Different objectives:** For example, efficiency versus effectiveness under a shared scenario.
- **Method comparison:** Different research methods applied to a common question.
- **Complementary lenses:** Distinct contextual or interpretive layers, such as language, biography, and form, contributing to a shared understanding without competing for a single winning explanation.
- **Independent challenge:** A separate critical assessment without being coached toward the main stream's conclusion.

### Stream contract

Each stream is one ordinary OKF concept containing assignment, progress, results, and links to resulting topic concepts. Its sections are not separate records. If the researcher needs additional stream-related concepts, those are workspace-specific extensions outside this plugin.

Record the question or objective, starting context (a Git revision when applicable, otherwise identified working content), permitted context sharing, boundaries, method, budget, expected output, comparison criteria, and execution identity where available. Record actual model and settings only when exposed by the runtime; do not invent them.

For independent assignments, give streams the common overview and relevant starting material but no sibling conclusions until synthesis. Use fresh contexts where supported. If isolation cannot be achieved, disclose the limitation rather than claim a blind comparison. Starting from working content does not require committing it or creating a stored snapshot.

For complementary assignments, allow useful context exchange when the assignment permits it. Record consequential exchanges and their influence in the receiving stream concept, linking the originating stream; do not create separate exchange records. Shared context does not make the outputs independent corroboration. A request for independent work takes precedence over the convenience of sharing.

Each stream writes to its own concept and any distinct topic concepts or artifacts assigned to it; separate output directories are not required. The coordinator integrates results and then updates related research, shared content, the project overview, and the project journal. Delegated agents follow the delegated execution boundary even when invoking a skill whose ordinary workflow updates these records, avoiding concurrent writes or disclosure of independent results before synthesis. Practical execution may use subagents, explicitly requested separate tasks, or manually run sessions; the persistent stream record should not depend on one transport. Runtime model selection and task creation must follow available capabilities and authorization rather than being assumed.

### Resuming interrupted streams

Before dispatching resumed work, inspect the stream concept, partial outputs, and any available native runtime state. Distinguish an unknown execution outcome from a confirmed failure or completion. Reconcile what is already recorded and continue from verified progress; do not invent completion or rerun the entire assignment merely because its process is inaccessible. Do not repeat consequential actions until their prior outcome is resolved; record the uncertainty in the stream concept and seek the missing evidence or researcher input when needed. Continue unaffected work where possible. Use existing records and runtime tools, not a custom dispatch, transaction, or recovery system.

### Delegation and model selection

Use subagents for concrete, bounded tasks when independent work, specialization, or parallel execution materially helps. Select models according to actual task requirements and available capability information: more capable models for complex reasoning, ambiguous interpretation, or difficult synthesis; less costly models for suitable extraction, organization, or other bounded work. Check outputs proportionately rather than equating model tier with correctness.

Select among the capabilities actually available, using reasonable defaults appropriate to the assignment. The current or runtime-default model is an acceptable fallback for routine work when model selection is unavailable or another model is not clearly warranted. Do not interrupt routine delegation merely because suitability or availability is uncertain. Honor explicitly requested models for comparative runs; ask before substituting an unavailable requested model, or when a consequential quality/cost tradeoff needs the researcher’s choice. Continue unaffected work while that choice is pending. Disclose material limitations and record actual settings only when exposed by the runtime. Use current capability information rather than fixed model rankings or unsupported price assumptions.

Record requested and actual execution arrangements, and material substitutions or limitations. Execution arrangements describe the use of native runtime tools and preferences; they do not require profile schemas, custom adapters, or configuration infrastructure. Model choice follows the available runtime controls; no model-switching capability is assumed or required. Preserve the separation between authorization to delegate a bounded subtask and authorization to create a new user-owned task.

### Synthesis requirements

Assess whether streams used comparable definitions, scenarios, evidence, and measures. Identify common sources; model agreement is not additional independent evidence. Preserve incompatible assumptions and unresolved disagreements. For competing objectives, show tradeoffs and conditional preferences rather than assuming an arbitrary weighted score. For complementary lenses, explain how the layers interact, where they constrain one another, and which interpretations remain open; do not force a ranking or common numerical measure. Feed newly revealed questions back into refinement.

## 9. Experiment protocol

Each experiment is a single OKF concept containing its plan, session entries, observations, current status, and assessment, with links to supporting artifacts and any separate hypothesis concept. It supports agent-executed and researcher-executed activities, including computational trials, interviews, usability studies, observations, prototypes, and practice exercises. The sections below describe content within that concept, not separate records or files. Any further subdivision is a workspace- or project-specific extension outside the core skills. Choose method-specific details only when relevant.

### Plan

Capture purpose, question, an OKF link to a hypothesis concept when applicable, method, conditions, materials or participants where relevant, measures or observations, interpretation criteria, execution responsibilities, and stopping criteria. Record practical and ethical constraints applicable to the activity. Establish which generated outputs will be retained as OKF artifacts and which are temporary, honoring existing project preferences. Resolve retention needed to conduct or assess the activity during planning, then apply the choice across sessions without asking again for every output.

### Session entries within the experiment

Record timestamp or reported date, actor or reporter, the procedure used (linked to the relevant plan/change entry or Git revision when available), conditions, actions, observations, artifacts, and deviations. Preserve reported uncertainty. Missing data, failed attempts, and unexpected outcomes remain visible.

Never imply that the agent directly observed a human-reported event. Separate verbatim or faithfully summarized observations from interpretation and recommendations. Later corrections identify which earlier entry they amend.

### Status and assessment

Proposed progress states: planned, active, paused, completed, abandoned. Outcome is separate: supportive, contradictory, mixed, inconclusive, or a method-appropriate description.
Skills may propose outcome dimensions that clarify a result, such as effectiveness, efficiency, usability, durability, or a method-specific distinction. For each relevant dimension, record the assessment, supporting observations, uncertainty, and implications. An outcome can be supportive on some dimensions and mixed, contradictory, or unassessed on others.

Distinguish planned dimensions from dimensions discovered during the activity. Do not manufacture precision or choose dimensions after seeing results to make an experiment look successful. Any overall summary should preserve consequential exceptions; a majority of supportive dimensions does not automatically outweigh a critical failure. Use explicit researcher priorities or declared criteria if aggregation is needed.

Each assessment considers the strength of evidence, comparability across sessions, confounding factors, and limits on generalization. Exploratory observations must not be retrospectively presented as prespecified tests. Record substantive procedure changes within the experiment concept, including what changed, why, when it took effect, and which sessions or comparisons are affected. Update the current plan in place while retaining these change entries; do not require a separate plan snapshot, versioning system, or commit for each change.

Accepting an experiment plan does not automatically authorize every possible external action. Execute within the user's actual scope; guide or wait for human activities when necessary. A status update should be useful even when no execution tool is available.

## 10. Research branches, forks, clones, and reconciliation

These terms describe research content operations. Implement them as ordinary OKF concepts in project bundles, using OKF links/backlinks and provenance. They do not invoke Git branch, fork, clone, or worktree operations and do not inherently create a project or repository.

| Operation | Intended behavior |
|---|---|
| Research branch | Develop an alternative line of inquiry in linked concepts while retaining the original inquiry. A research stream may work on that alternative but is not the branch itself. |
| Research fork | Derive a separately editable concept or set of concepts from selected existing research—for example, fork an experiment to vary its procedure. Preserve origin links and explain what differs. |
| Research clone | Copy selected research concepts—for example, duplicate research notes to compare two options. Preserve source provenance and origin links while giving the copies distinct OKF concept identities. |
| Reconcile | Compare related concepts and incorporate selected findings or changes with rationale and provenance. Retain meaningful differences and unresolved conflicts. |

Copied history is inherited evidence, not new authorization or newly performed work. Preserve originating decisions, sessions, observations, and outcomes with clear source attribution. A copied decision does not carry acceptance or authority into a new context; record any target-context proposal separately from inherited acceptance, and require actual acceptance or applicable standing authority before treating it as authorized. Do not copy an implementation-complete status as a claim that the new context has been updated.

A new experiment fork starts as planned, retaining prior sessions and assessments only as attributed history. Those sessions do not count as execution of the fork’s changed procedure. Preserve active execution state only when the researcher explicitly requests continuation and the recorded progress supports it; never infer an ongoing process or completed target-context work merely from copied fields. These distinctions stay within the copied concepts, without separate transfer records or state-management infrastructure.

By default, copy the selected current working content. Use content from a historical Git revision only when requested, recording that revision. If the source contains uncommitted changes, identify it as working state; do not fabricate a commit reference or commit automatically. The resulting concepts are current research content, not archival snapshot objects.

Copy or retain links to supporting concepts and artifacts as appropriate; do not duplicate shared artifacts unnecessarily. Maintain OKF links/backlinks between origins and derivatives, with a short explanation in the copied concept. Record significant copies in the project journal; no separate transfer plan or lineage record is required. Changes to an origin do not silently overwrite a derivative. Use OKF's existing machine and human verification support without adding a separate explicit review stage. A new project is created only when requested by the research scope, not merely because the user says “fork” or “clone.”

## 11. OKF integration and implementation boundaries

- Use the installed OKF skills and CLI for conforming bundle reads, writes, provenance, lifecycle handling, and validation.
- Research record types, journal semantics, statuses, lineage metadata, and the workspace overview are local conventions layered on OKF. Optional ontology rules are advisory, not requirements for OKF conformance.
- Preserve OKF's reserved index and log structures and existing curated content.
- Do not confuse generation provenance, source verification, user acceptance of a decision, and successful experiment execution. These are different events.
- Maintain links/backlinks and machine and human verification through the OKF plugin. Meaningful edits may invalidate prior verification; preserve that distinction without introducing a separate explicit review requirement.
- Use Git and Git LFS for history and artifact version storage. Implement research copies as ordinary OKF concepts, and use existing OKF linking and verification operations. Custom helpers may be developed later for project-specific needs outside the core skills, within the required persistence architecture; this is not authority to create a separate storage or version-control system.
- Follow environment capabilities for agents, tasks, model selection, and execution. Where a requested arrangement is unavailable, report the limitation and offer a compatible arrangement without mislabeling it.

Confirmed packaging: one research plugin containing the eleven focused skills, the selectively loaded references and adaptable templates proposed in section 7. The existing OKF plugin is a dependency; invoke its skills and capabilities rather than copying them into this plugin. Custom helpers are outside the core package. Avoid duplicating OKF's own instructions or adding a mandatory umbrella workflow.

### Workspace maintenance boundary

Workspace upgrades, schema or template version management, and migrations are handled by agents outside these skills. Do not provide an upgrade skill, migration framework, version manifests, compatibility matrix, automatic pre-write version checks, or per-record format/template version tracking. Ordinary research work uses the current OKF concepts and existing OKF validation.

Git history, external source editions or dataset versions, and records of which experiment procedure was used remain research provenance. They do not imply a workspace-format versioning system. Editing a skill or template does not automatically rewrite existing research.

### Method-specific standards of rigor

The four initial guides must define evidence standards and failure checks, not merely procedural steps:

| Method family | Required distinctions and checks |
|---|---|
| Documentary/historical | Source provenance and date; primary versus secondary material; editions and translation; context, archival gaps, and anachronistic assumptions |
| Interpretive | Textual or artifact evidence versus contextual facts and inference; competing readings; speaker versus author; scope and limits of an interpretation |
| Qualitative | Recruitment or case selection, context and researcher influence, transparent interpretation/coding where used, counterexamples, and limits of transferability |
| Empirical/comparative | Operational definitions, measures, baselines and comparison conditions, confounders, uncertainty, missing data, reproducibility details, and exploratory versus prespecified analysis |

Apply checks that fit the actual method and scale; do not force statistical criteria onto close reading or universalize one qualitative methodology. Specialized guides inherit the evidence/provenance contract and declare where their standards differ.

### Skill maintenance boundary

Improving skills, templates, helpers, or package resources is ordinary agent work outside this research-skill family, as are workspace upgrades and migrations. Do not provide a dedicated improvement skill or require improvement proposals, adoption records, release assessments, or a controlled improvement workflow. A research session may mention a problem, but does not silently change the governing skills. Maintenance can be requested and handled separately using appropriate authoring tools.

## 12. Validation scope

Use the OKF plugin’s existing validation, links/backlinks, and machine and human verification capabilities. Use those existing checks for reference preservation and valid concept updates; the core package does not require custom helpers or a helper test suite. These checks do not introduce a separate explicit research-review stage.

Behavioral evaluation is not required. Do not create a behavioral evaluation harness, retained research fixtures, scoring rubrics, model regression suite, or mandatory end-to-end research acceptance runs. The workflows and method guidance in this specification define the intended functionality without requiring that additional infrastructure.

## 13. Proposed implementation sequence

The target is the complete workflow, including all eleven skills and their required supporting assets. Build in dependency order with integration checkpoints; a checkpoint is not a reduced final deliverable.

1. Finalize the record and provenance conventions, package boundaries, and method guidance across all four method families.
2. Implement shared persistence, evidence links, journals and overview summaries, retrieval, identity, artifact handling, and project operations using existing OKF/Git capabilities.
3. Implement all research, refinement, decision, learning, challenge, update, and experiment workflows with compact templates and method guides.
4. Implement independent/complementary stream coordination, documented execution arrangements supported by the available runtime, synthesis, and resumable handoffs.
5. Complete shared-bundle reuse, branch/fork/clone/reconciliation with focused checks for reference preservation and safe copying.
6. Run applicable existing OKF validation. Do not build a behavioral evaluation system.
7. Validate the installable package in its installed location, including shared resource resolution, first-use setup, discovery and handoffs, declared dependencies, and the documented execution arrangements. Record remaining limitations and completion evidence.

Completion means the entire specified workflow is implemented and applicable structural checks completed, not merely that eleven `SKILL.md` files exist.

No skills, bundles, or execution harness are created as part of this specification draft.

## 14. Confirmed design choices and implementation guidance

Use compact records and the record boundaries in section 4. Split topic concepts only when independent linking or development adds value; do not split records explicitly defined as single concepts. Resource paths and examples remain illustrative, not additional record requirements.

### Confirmed supporting-resource choices

Package the family as one plugin containing eleven research skills, shared guidance, and compact templates, with the existing OKF plugin as a dependency rather than a copied implementation.

1. **Method coverage:** Include all four method families initially and support gradual addition of specialized guides.
2. **Tooling scope:** Use existing OKF/Git capabilities. Custom helpers and additional tooling are possible later project-specific extensions, not core deliverables.
3. **Template detail:** Start compact and expand as research develops. Minimal identity, provenance, and historical-reference requirements always apply.

### Confirmed simplifications

- Use a simple workspace overview and ordinary OKF stream records, without separate manifests.
- Keep each project’s current summary in its research overview; detailed state stays in the relevant concepts.
- Leave workspace upgrades, version management, and migrations to agents outside these skills. Leave skill improvement to ordinary agent work as well. The family contains eleven skills, with neither an upgrade skill nor an improvement skill.

### Recommendations for the complete build

- Keep one canonical development package and explicit workspace overrides; avoid editing cached installed copies as the source of truth.
- Keep retrieval inside project management and evidence/rigor standards shared. Keep package maintenance outside the research workflows.
- Use native agent tools and existing OKF/Git capabilities; no always-running service is required for the complete workflow.
- Select available models and reasoning effort by task complexity and observed needs. No runtime-model behavioral evaluation suite is required.

## 15. Revision summary

The specification includes optional shared workspace knowledge, task-appropriate delegation, text extraction, external-resource links, and artifact-saving preferences, dimensional experiment outcomes, iterative refinement, and four initial method guides with gradual specialization.

The confirmed architecture uses one Git repository per workspace and one OKF bundle per project subfolder. Git and Git LFS preserve history and artifacts; commits are explicit actions. Research branches, forks, and clones are ordinary linked OKF concepts. No parallel object, snapshot, transaction, or history store is permitted.

Journals contain event entries. Research overviews contain directions, disagreements, and current project summaries; detailed state remains in the relevant research concepts. Sources/evidence and artifacts use OKF links with context in referring records. Skills maintain links/backlinks and use existing OKF machine and human verification without a separate explicit review stage.

A simple workspace overview describes projects and links to their research overviews. Streams are only for delegated or parallel work, with details in ordinary OKF records; no separate manifests are required. Version tracking and workspace migrations are outside the research skills and are handled by agents separately. The upgrade and improvement skills have been removed, leaving eleven research skills; maintenance is handled separately by agents. Behavioral evaluation infrastructure is outside scope; applicable existing OKF validation remains.

Hypotheses remain separate concepts, linked to topic concepts and any delegated/parallel streams or experiments that investigate them. Findings and comparisons stay in topic concepts; agents may produce comparison reports without a prescribed skill template. Each project has one journal, and each experiment is one concept containing its own plan, sessions, status, and assessment. Further experiment subdivision is a local extension.

Consequential decisions are separate concepts containing their own rationale, acceptance, impacts, and follow-up actions. Ordinary choices stay in the research overview and project journal. Copies retain origin links and explanations within the concepts rather than separate transfer or lineage paperwork. Skill improvement is outside the research-skill family.

The minimal layout adds folders only when useful. The core package uses existing OKF/Git capabilities without custom helpers. Continuous persistence is required: ingestion, substantive conversation, and refinement update relevant and related concepts, the research overview, and the project journal when warranted, without repeated save prompts or automatic Git commits.

Document handling extracts research-relevant text into concepts, links external resources, and asks whether to save other artifacts only when no preference is established. New contradictory evidence updates affected concepts immediately with rationale and uncertainty; accepted choices remain unchanged pending reconsideration. Packaging is confirmed as one research plugin depending on the existing OKF plugin.

When intent and placement are clear, agents initialize a missing project bundle, research overview, and project journal without a separate setup prompt. Each delegated/parallel stream is one concept; additional stream structure is a workspace-specific extension. Shared content is updated during authorized research, and promoted or created when cross-project reuse is evident, preserving scope, provenance, and restrictions.

Exact file history is available at Git commits; meaningful intervening rationale and procedure changes remain in ordinary research records without extra snapshots or forced commits. Independent streams remain isolated until synthesis. Complementary streams may exchange context when their assignments permit it, recording consequential exchanges within their stream concepts.

Learning plans are single concepts containing objectives, activities, progress, and assessments; further structure is a workspace-specific extension. Routine delegation uses available capabilities and reasonable defaults, asking only for an unavailable explicitly requested model or a consequential tradeoff requiring researcher choice. Source context and change assessments stay in their owning concepts rather than separate records.

Briefings, understanding maps, investigation plans, evidence maps, and critiques are guidance for sections in topic concepts, not additional record types or prescribed reports. Learning preferences stay in the learning plan or research overview. Substantive reversals create linked superseding decision concepts; corrections and implementation updates remain in the original decision.

Fresh-context review clarifications scope delegated writes to assigned concepts/artifacts, with coordinator integration into shared and project records. Copied acceptance and experiment outcomes remain attributed history, not new authorization or execution; new experiment forks start planned unless continuation is explicit and supported. Initial workspace creation may initialize Git and the workspace overview when placement is clear, without committing.

Shared disagreements and rationale stay in their shared concepts, with significant changes in the initiating project journal and linked summaries in affected project overviews; the shared bundle has no separate overview or journal. Experiment planning establishes retention for generated outputs across runs. Stream resumption checks actual progress and does not repeat consequential actions with unknown outcomes. Execution arrangements are native runtime choices, not a new profiles framework.
