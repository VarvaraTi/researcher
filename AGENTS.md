# Learning assistant project

Write assistant-authored file content in English, including documentation,
learning records, metadata descriptions, and code comments. Communicate with the
user in Russian in chat. Apply this preference to future work; preserve original
source material and verbatim learner submissions when exact retention is required.

Store all learner-specific visualizations, images, animations and HTML cheat sheets
inside their owning course or article study, with versioned artifact paths and
links to the relevant topic, lesson/sheet and sources. Never publish a canonical
learning asset under project-level `docs/visualizations/` or a temporary directory.
Follow [visual learning assets](docs/learning-operations.md#visual-learning-assets).

For a request to start, continue or manage personal learning, read
`skills/coordinate-learning/SKILL.md` and follow its routing. This explicit file
entry point also works when repo skills have not yet appeared in the host list.
For implementation work, use `learning-spec.md`, `docs/learning-contracts.md` and
relevant [ADRs](docs/adr/README.md); run `sh scripts/check-learning.sh` after code changes.
The main specification owns current requirements; schemas/code own exact formats,
and contracts document additional runtime semantics without duplicating schemas.
ADRs own significant architectural choices and their rationale. Link ADRs to
relevant specification sections and link those sections back; ordinary requirement
edits do not require a new ADR.
Update affected specifications and contracts alongside implementation. Distinguish
proposed behavior, accepted requirements and actual implementation status. Keep
plans and task lists temporary; remove completed ones after preserving unique
requirements or rationale in their owning documents. Use Git for earlier versions.
Before modifying files or structure, propose the scope in chat and obtain explicit
user approval. Approval covers that scope across turns; do not request it again
unless the scope changes. Read-only analysis does not require approval.
After a task makes a notable user-facing or developer-facing change, add a concise
English entry under `Unreleased` in `CHANGELOG.md`. Record the outcome and link to
the relevant documentation; do not copy raw commit logs or routine edits.

When an unexpected problem reveals a reusable lesson about the agent harness,
record it in `docs/lessons-learned.md` in English. Harness lessons concern the
agent's instructions, tools, plugins, execution environment or workflow. Capture
the context, cause, resolution and how to avoid or diagnose a repeat. Do not use
this file for ordinary product bugs or learner/course issues. Omit routine
transient errors, secrets and personal data. Keep architectural decisions in
`docs/adr/`.

Keep one writer for active learning data. Personal runtime is `learner/`,
`courses/`, `learning.local.json` and the root catalog/config, all ignored by Git.
`tests/fixtures/` contains synthetic material only. Preserve the upstream `okf/`
checkout and unrelated staged changes. The installed OKF CLI is pinned to 0.6.0.
Use `.local/bin/okf` from this project, or prepend the project's `.local/bin` to
PATH before running plugin preflight/commands. The active OKF plugin is 0.6.0
(CLI range >=0.6.0,<0.7.0); an optional `okf/` checkout is reference material, not the runtime.
