# Changelog

Notable project changes are recorded here in English. This log starts with the
course-map work recorded on 2026-11-04; it is not a complete project history.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Unreleased entries describe implemented changes that have not been assigned to
a release. Architectural rationale lives in [ADRs](docs/adr/README.md).
Personal learning records remain in their owning course or learner bundle.

## Unreleased

### Added

- Two local architecture views: [KB boundaries, workflows and record links](docs/visualizations/knowledge-architecture.html)
  and a [repository explorer](docs/visualizations/repository-architecture.html)
  with file-purpose tooltips, rationale, search and source connections. These
  documentation snapshots describe the system without embedding personal records.

- Complete the authorized Alex Xu shared-library rollout: retained text and honest
  page coverage, saved source/run linkage, preserved historical learning records,
  and refreshed the existing course map. See [library operations](docs/learning-operations.md#shared-book-library).

- Add the private shared book library workflow: edition-aware retained sources,
  immutable extraction runs, page coverage and scoped passage retrieval with
  guarded publication and packaged helpers. See [library operations](docs/learning-operations.md#shared-book-library);
  live migrations require separate authorization.

- Updated the course knowledge retrieval specification with learner-validated
  full-text indexing, per-run coverage manifests, source-preserving cross-book
  concepts and retrieval defaults.
  See [course knowledge retrieval](learning-spec.md#41-book-library-and-grounded-questions).

- An optional local HTML course map showing planned stages, prepared lessons,
  the current session, previews, and revealed reference material. Search and
  filters help navigate the course; session coverage and assessment evidence
  are displayed separately. See [Course learning maps](docs/learning-operations.md#course-maps).
- A [course-map update command](scripts/update-course-map.mjs) that builds the
  map from saved learning records. A stable link inside the course opens the
  latest snapshot; earlier snapshots remain in the course's
  `references/visualizations/` directory. The command also supports a read-only
  freshness check with `--check`.
- [Course-map checks](tests/course-map.test.mjs) for initial plans, preview and
  completion states, assessment evidence, reference visibility, and local links.

### Changed

- Trim command, operations and plugin references to invocation and practical
  procedures; remove duplicate article, learner-model and map documents. Keep
  incomplete-audit handling in the [learner-model skill](skills/update-learner-model/SKILL.md#reconcile-saved-evidence)
  and update package resources and navigation links.

- Include default policies in plugin packages and link the learner-model skill
  directly to its [support policy](policies/defaults/support-v1.json).

- Keep book-library requirements in the [main specification](learning-spec.md#41-book-library-and-grounded-questions)
  and necessary commands/recovery in [learning operations](docs/learning-operations.md#shared-book-library).
  Remove duplicate library documents and the historical validation report, link
  ADRs and current specification sections in both directions, and package the
  linked specification with its required resources.

- Consolidate architectural rationale in [six ADRs](docs/adr/README.md), move exact
  review rules into [contracts](docs/learning-contracts.md#review-policy-and-event-ordering),
  and reconcile current specifications. Replace the standalone documentation
  workflow and completed implementation plans/task lists with concise agent rules,
  including explicit approval before file or structure changes.

- Remove the standalone shared-library validation report and its package entry;
  retain current requirements and [operating instructions](docs/learning-operations.md#shared-book-library).

- Fix duplicate JSON flags in course-map generation and verify the full CLI
  render, freshness check and repeat-run behavior on an isolated synthetic course.

- Course preparation, teaching, assessment, and reference workflows now refresh
  an enabled course map after relevant saved changes. Maps are enabled per
  course and regenerated during the workflow; they do not update in the
  background while the chat is closed.
- Course setup instructions now define the initial map creation step after a
  requested full course plan is accepted and saved.
