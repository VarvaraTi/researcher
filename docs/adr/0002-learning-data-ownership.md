# ADR 0002: Learning data ownership

Status: Accepted
Recorded dates: article/course boundaries and study isolation, 2026-11-01;
shared-library separation, 2026-11-08.
Authority: explicit learner decisions recorded in the former decision register.

## Context

Preferences must transfer across courses while learning history remains scoped.
Several standalone articles can share storage without sharing attempts or
assessment evidence. A book can support multiple courses without copying its
searchable text into each learning history.

## Decision and recorded alternatives

Use a shared learner bundle, a shared articles bundle with one owned Study folder
per article, one bundle per full course, and an optional shared book library.
Topics and chapters do not become bundles. Use domain labels rather than Domain
records. Library setup occurs only when a selected book requires it.

On 2026-11-01, Study-owned article records superseded the earlier convention of
separate Course records for articles in the shared articles bundle. That earlier
representation is historical, not an alternative active schema. Full courses
retain their Course representation.

The same date's learner-model repair introduced explicit source-digest review
receipts so new learning evidence can be reconciled without silently inferring
lasting preferences. Self-report, observation and hypothesis remain distinct.

## Consequences

Reads and writes must preserve selected-study and course ownership. Shared
references require explicit library scope; they do not transfer learning evidence.
Personal preferences, source selection and migration receipts stay in private
runtime records. This ADR does not authorize adding sources or migrating data.

## Current requirements

- [Current specification](../../learning-spec.md#11-persistence-and-okf-model).
- [Article schema](../../schemas/article-ontology.yaml) and [course preparation](../../skills/prepare-course/SKILL.md).
- [Learner reconciliation](../../skills/update-learner-model/SKILL.md#reconcile-saved-evidence).
