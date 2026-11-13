# ADR 0003: Immutable history and guarded publication

Status: Accepted
Recorded foundation date: 2026-10-25; individual publication choices were not
separately dated in the former register.
Authority: retained-answer requirements and recorded engineering rules.

## Context

Retries, assessment corrections and interrupted writes must preserve the original
answer and the evidence used for past judgments. Ordinary successful storage calls
do not establish all application invariants.

## Decision and recorded alternatives

Keep original submissions, judgments and cited content versions immutable through
the supported workflow. Save answers before assessment; deduplicate retries by
stable identity and content. Represent corrections as linked judgments, then
derive progress and schedules from complete, ordered evidence.

Stage proposed changes, apply structural and learning checks, and activate through
OKF changesets with expected-digest guards. Reuse OKF recovery rather than building
another transaction journal. Maintain one writer. A shared-profile failure must
not discard a successfully saved course answer.

## Consequences

Old evidence remains inspectable and derived state can be rebuilt. Guarantees are
workflow guarantees, not filesystem immutability. Publication is per bundle and
journaled per file; no cross-bundle atomicity or concurrent-reader snapshot is
claimed. Colocated ontologies and ordinary catalogs are supported; custom
interpretation overrides require staging-equivalence checks before use.

## Current requirements

- [Current specification](../../learning-spec.md#111-persistence-protocol-and-recovery).
- [History and retained content](../learning-contracts.md#history-and-retained-content).
- [Publication protocol](../learning-contracts.md#publication-protocol).
- [Review policy and ordering](../learning-contracts.md#review-policy-and-event-ordering).
