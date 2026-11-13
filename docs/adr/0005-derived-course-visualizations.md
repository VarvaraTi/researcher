# ADR 0005: Course-owned visuals and derived maps

Status: Accepted
Recorded dates: visual ownership, 2026-11-02; optional evolving course map,
2026-11-04.
Authority: explicit learner requests recorded in the former decision register.

## Context

Learning visuals need their topic, lesson and source context. A dashboard must
reflect saved learning records without becoming an independent progress store.

## Decision and recorded alternatives

Keep learner-specific visuals in versioned artifact paths within the owning
course or article Study. Link them from relevant learning records. An optional
local course map is generated from the saved plan and learning history and refreshed
during authorized chat workflows. Project-level canonical learning visuals,
manual progress checkboxes, a server and background refresh were excluded.

## Consequences

Exports must account for local visual dependencies. Earlier snapshots remain
available, while the map exposes a stable navigation entry. Planned coverage,
prepared content, completed activity and demonstrated knowledge remain distinct.
The original map request applied to one selected course; it did not enable maps
for every course.

## Current requirements

- [Current specification](../../learning-spec.md#10-visuals-and-cheat-sheets).
- [Visual ownership contract](../learning-contracts.md#visual-artifact-ownership).
- [Visual operating instructions](../learning-operations.md#visual-learning-assets).
- [Course-map operation](../learning-operations.md#course-maps).
