# Learning command interface

This is the invocation reference for `learning-rules`, which has no full command
help. Product behavior is in the [specification](../learning-spec.md); record fields
are defined by [schemas](../schemas/learning-input.schema.json) and the bundle
ontologies in `schemas/`. Resolve executable paths through [runtime locations](plugin.md#runtime-locations).

## Command envelope

Pass one command argument and one JSON request through stdin:

```sh
"$LEARNING_WORKSPACE/.local/bin/learning-rules" inspect-state < request.json
```

A source checkout can use `target/debug/learning-rules`. Request shape:

```json
{"schema_version":1,"data":{"okf":"/absolute/workspace/.local/bin/okf","bundle":"/absolute/workspace/courses/COURSE","scope_bundles":["learning.learner"]}}
```

This example is for `inspect-state`; replace paths and scope with the selected
workspace. Other commands use their named `$defs` in the
[input schema](../schemas/learning-input.schema.json). Successful output contains
`result`; errors contain `error.code` and `error.message` and exit nonzero. JSON
paths do not expand shell variables. Use pinned OKF 0.6.0.

| Commands | Effect |
| --- | --- |
| `inspect-state` | Read the complete selected course/study state for authoritative calculations. |
| `audit-learner-model` | Read confirmed settings and pending source-digest reviews. |
| `resolve-settings`, `check-prerequisites`, `resolve-assessments` | Pure resolution/validation; no persistence. |
| `grade-answer`, `compute-progress`, `compute-schedule`, `review-support`, `check-learning-state` | Pure calculation/checks; callers save accepted results separately. |
| `save-attempt`, `save-observation`, `save-strategy-application` | Persist by stable identity; identical retry succeeds, conflicting retry is rejected. |
| `publish` | Validate a staged OKF changeset and apply it with the reviewed baseline guard. |

## History and retained content

Use `inspect-state` rather than a sampled search to construct calculation history.
For article studies pass the selected Study ID in `context.study`; course and
learner contexts omit it. `history` and `objectives` feed the pure calculations;
`next_sequence` is bundle-wide. Coverage is separate from assessed evidence.
`complete:true` supplied directly to a pure function is a caller assertion.

Save the original answer before assessment. On failure retry the pending operation,
not the learner submission. Stored frontmatter omits absent `judgment.supersedes`;
unlike pure calculation input it cannot use an explicit null reference. Wrap
LearnerPreference values as `{data: ...}` before passing their data to settings
resolution. See the [typed requests](../crates/learning-rules/src/persistence.rs)
for persistence details not represented by standalone JSON validation.

## Review policy and event ordering

Use the active versioned policy. Default values are in
[review-v1](../policies/defaults/review-v1.json) and
[support-v1](../policies/defaults/support-v1.json). Exact calendar, event-ordering
and budget semantics are implemented in [schedule.rs](../crates/learning-rules/src/schedule.rs)
and [rules.rs](../crates/learning-rules/src/rules.rs). Do not compute authoritative
schedules from prose or silently relabel historical policy versions.

## Publication protocol

Plan the exact OKF changeset, retain `changeset-summary.base_digest`, then pass
`context`, `changeset` and `expected_digest` to `publish`. For a library-only
publication add `mode: "library"` beside context. See
[publication and recovery](learning-operations.md#publication-and-recovery).

The publisher checks staged state before guarded application. Single-bundle
publication is journaled per file; it is not a concurrent-reader snapshot or a
cross-bundle transaction. One writer is required. Immutability is enforced by the
supported workflow, not by filesystem protection. Colocated ontologies and ordinary
catalog settings are supported; custom interpretation overrides require staging
checks. Structural success does not establish semantic source review.

## Visual artifact ownership

Retain visuals through the [asset procedure](learning-operations.md#visual-learning-assets).
This is an authoring requirement, not a new Rust filesystem gate.

## Shared library contract

Fields and references are defined in the [library ontology](../schemas/library-ontology.yaml).
Application validation in [persistence.rs](../crates/learning-rules/src/persistence.rs)
requires coverage arrays even when empty; ontology declarations alone do not enforce
that condition. SourceSelection activation and PassageReview evidence are separate
from immutable extracted text. Retrieval selection, stale/blocked exclusions and
source-byte checks are implemented in [library-retrieval.mjs](../scripts/lib/library-retrieval.mjs).
Use [library operations](learning-operations.md#shared-book-library) for ingestion,
retrieval and retries. An empty or incomplete search does not prove absence.
