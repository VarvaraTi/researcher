# Learning assistant project

## Working with contributors

Follow the contributor's stated language preference in chat. If none is
available, reply in the language of their request. Write assistant-authored
file content in English, including learner records and metadata. Preserve
original source material and verbatim learner submissions.

Before modifying files or project structure, propose the scope in chat and
obtain explicit approval. Approval carries across turns unless the scope
changes. Read-only analysis can proceed without approval.

## Personal learning

To start, continue, or manage learning, follow the
[learning coordinator](skills/coordinate-learning/SKILL.md) even if the host
has not listed the skill. Store learner-specific visuals and HTML cheat sheets
with their course or article study. Follow the
[visual asset procedure](docs/learning-operations.md#visual-learning-assets)
for versioning, links, and publication.

## Development

- Use the [specification](learning-spec.md) for current requirements, schemas
  and code for exact formats, and [contracts](docs/learning-contracts.md) for
  runtime semantics. Update affected documents alongside implementation and
  distinguish proposals, accepted requirements, and implemented behavior.
- Record significant architectural choices in [ADRs](docs/adr/README.md).
  Link each decision and its specification section to one another.
- Keep plans temporary. Transfer lasting requirements and rationale to their
  owning documents before removing completed plans. Use Git for prior versions.
- Run `sh scripts/check-learning.sh` after code changes. For notable changes,
  record the outcome under `Unreleased` in the [changelog](CHANGELOG.md)
  and link to the relevant documentation.
- Record reusable lessons from unexpected agent, tool, or workflow problems
  following the [harness lessons guide](docs/lessons-learned.md).

## Local workspace

Keep one writer for active learning data. Keep `learner/`, `courses/`,
`learning.local.json`, and the root runtime catalog and configuration out of
Git. Use only synthetic material in `tests/fixtures/`.

Use the project-local `.local/bin/okf` for commands and plugin preflight.
See [OKF setup](OKF-SETUP.md) for the pinned versions. Preserve the optional
upstream `okf/` reference checkout and unrelated staged changes.
