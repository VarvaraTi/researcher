# ADR 0001: OKF and focused local Rust commands

Status: Accepted
Recorded foundation date: 2026-10-25. The former decision register associates
this date with foundation-v1 / review-v1 / learning-skills-v1; individual choices
were not separately dated.
Authority: agreed product boundaries; Rust selected in the implementation plan.

## Context

OKF already provides storage, queries, provenance and recoverable changesets.
Scheduling, progress reduction and learning-state checks need deterministic rules
that agent instructions alone cannot enforce.

## Decision and recorded alternatives

Reuse the installed OKF CLI and skills. Keep focused calculations in an independent
Rust library with thin local commands; invoke OKF through its supported CLI,
without coupling to private upstream Rust APIs. The chat agent coordinates work
using its existing model. A new storage engine, mandatory middleware, server or
separate model API connection was explicitly excluded. Rust was an implementation
choice, not an OKF requirement; no detailed language comparison was recorded.

The 2026-11-01 PDF-helper request added optional macOS Swift/PDFKit extraction.
It is a platform-specific helper, not a dependency of the pure learning rules.

## Consequences

Deterministic mechanics can be tested independently of model prose. OKF remains
a separately installed, version-pinned dependency. PDFKit does not supply OCR or
diagram interpretation; extraction capability must be reported honestly.

## Current requirements

- [Current specification](../../learning-spec.md#2-responsibilities-and-architecture).
- [Command contracts](../learning-contracts.md#command-envelope).
- [Dependencies and runtime](../plugin.md#runtime-locations).
