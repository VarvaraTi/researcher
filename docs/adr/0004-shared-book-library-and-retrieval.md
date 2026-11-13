# ADR 0004: Shared book text and scoped retrieval

Status: Accepted
Recorded dates: product choice, 2026-11-08; runtime conventions, 2026-11-11.
Authority: learner-validated library requirements and recorded implementation rules.

## Context

A retained opaque PDF and a few prepared lessons do not provide searchable access
to the whole book. Questions can concern material outside those lessons. Extraction
can be partial and is not evidence that diagrams or claims were reviewed.

## Decision and recorded alternatives

Retain each selected edition and all extractable original text in a shared OKF
library. Preserve source/page provenance and immutable extraction-run coverage.
Use scoped OKF lexical retrieval initially. Cross-book Concepts provide navigation;
answers remain grounded in opened source passages. Default to the selected course's
book; broader comparison is explicit.

Generated KnowledgeEntry summaries were rejected as the canonical searchable layer.
A separate vector index or GraphRAG pipeline is deferred until representative
questions demonstrate a need beyond the existing retriever. Preserve original
passages separately from any future generated search context.

On 2026-11-11, runtime conventions selected separate SourceSelection and
PassageReview records to keep run activation and source review distinct from
immutable extraction. Exact selection and failure rules belong in contracts.

## Consequences

Source text remains available beyond the prepared curriculum, without becoming
learner performance evidence. Lexical retrieval may miss paraphrases; empty search
results do not establish absence. Partial coverage must stay visible. Existing
course citations must survive source linkage or later migrations. Per-course
rollout authorization and receipts belong to private runtime, not architecture.

## Current requirements

- [Current specification](../../learning-spec.md#41-book-library-and-grounded-questions).
- [Shared-library contract](../learning-contracts.md#shared-library-contract).
- [Library operations](../learning-operations.md#shared-book-library).

## Background retained from the original specification

- [Retrieval-Augmented Generation, Lewis et al.](https://arxiv.org/abs/2005.11401).
- [Anthropic Contextual Retrieval](https://www.anthropic.com/research/contextual-retrieval).
- [Microsoft GraphRAG indexing dataflow](https://microsoft.github.io/graphrag/index/default_dataflow/).

These background sources informed the recorded alternatives; they do not verify
this project's retrieval quality or authorize adding another engine.
