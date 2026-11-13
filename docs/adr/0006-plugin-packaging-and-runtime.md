# ADR 0006: Allowlisted packaging and separate runtime

Status: Accepted
Recorded date: 2026-11-04.
Authority: existing plugin-distribution and setup decision.

## Context

The reusable plugin must be distributable without personal courses, learner data,
caches or an upstream reference checkout. Installing or updating it must preserve
the user's existing workspace and selections.

## Decision and recorded alternatives

Package an explicit allowlist of skills, supporting documentation, schemas,
templates, helpers and Rust source. Build writable runtime commands outside the
installed plugin cache. Initialize only missing workspace storage and leave source
selection and personal preferences unset. Keep the OKF plugin and pinned CLI as
separate prerequisites. No broader packaging alternative comparison was recorded.

## Consequences

Documentation moves must update package inputs and relative resource links.
Installed resources must resolve without the source checkout. Setup and upgrades
must preserve saved registrations and learning records; library setup remains
on demand. Source code and tests can be shared without distributing personal data.

## Current requirements

- [Current specification](../../learning-spec.md#22-plugin-distribution-and-runtime).
- [Plugin setup, packaging and runtime locations](../plugin.md).
- [Operating instructions](../learning-operations.md).
