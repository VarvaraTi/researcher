# Architecture decisions

ADRs explain significant architectural choices. Current behavior belongs in the
product specification; exact interfaces and algorithms belong in
[learning contracts](../learning-contracts.md). Operating instructions describe
how to use and recover the system. Personal preferences and rollout receipts
belong in their owning private bundles, not in ADRs.

| ADR | Decision | Status |
| --- | --- | --- |
| [0001](0001-okf-and-local-rust-commands.md) | OKF and focused local Rust commands | Accepted |
| [0002](0002-learning-data-ownership.md) | Ownership of learner, article, course and library data | Accepted |
| [0003](0003-immutable-history-and-guarded-publication.md) | Immutable history and guarded publication | Accepted |
| [0004](0004-shared-book-library-and-retrieval.md) | Shared original book text and scoped OKF retrieval | Accepted |
| [0005](0005-derived-course-visualizations.md) | Course-owned visuals and derived maps | Accepted |
| [0006](0006-plugin-packaging-and-runtime.md) | Allowlisted distribution and separate writable runtime | Accepted |

These records were consolidated on 2026-11-12 from existing project decisions,
specifications and implementation plans under explicit user approval. Original
dates and authority are stated where available. Consolidation does not mean that
every original choice was newly made or every product scenario is verified.
Historical documents did not consistently record alternatives; unrecorded
alternatives or motivations must not be invented during migration.

Link each ADR to the relevant section of `learning-spec.md`, and link that section
back to the ADR. Ordinary requirement edits do not require a new ADR. An
architectural reversal updates the specification and gets a new ADR, retaining
the old rationale. Markdown section links are sufficient; no relationship registry
or requirement-ID scheme is required.

One ADR covers one significant architectural choice. Record status, known decision
date/authority, context, alternatives actually recorded, decision, consequences
and links to current requirements. Keep requirement details in their owning
documents. Use stable increasing numbers; do not reuse identifiers. Status is
Proposed, Accepted, Rejected or Superseded; acceptance does not prove implementation.
A substantive reversal gets a new ADR, with replacement links in both records.
Retain the earlier context instead of rewriting it to justify the new choice.
