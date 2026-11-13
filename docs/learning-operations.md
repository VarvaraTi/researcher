# Learning operations

Install and resolve workspace/plugin paths using [plugin setup](plugin.md).
Use [command contracts](learning-contracts.md) for JSON requests; teaching and
record creation follow the selected skill, not a second workflow in this document.

## Publication and recovery

1. Read applicable bundle guidance. Prepare the exact OKF changeset in a private
   JSON file and run `okf changeset plan BUNDLE --from @CHANGES.json --json`.
   Check successful complete operation envelopes; retain `changeset-summary.base_digest`.
2. Pass that changeset and digest to `learning-rules publish` through stdin. Normal
   course publication includes any learner/library dependencies in context scope.
   For library-only publication use this shape, replacing the empty example
   changeset with the exact prepared document:

   ```json
   {"schema_version":1,"data":{"mode":"library","context":{"okf":"OKF_PATH","bundle":"LIBRARY_PATH"},"changeset":{},"expected_digest":"BASE_DIGEST"}}
   ```

   Direct OKF apply does not run the learning publisher's staged application checks.
3. If interrupted, stop writes and inspect `okf changeset recover BUNDLE --help`.
   Recover the pending journal through OKF; never delete it by hand or force
   rollback over intervening edits. Inspect the resulting state before retrying.
4. Reload `inspect-state`. Resume assessment from the retained answer; recompute
   stale derived progress/schedules after judgment success. Retry a failed shared
   observation independently, retaining its origin ID. If the baseline changed,
   review it and obtain a fresh plan rather than resubmitting a stale digest.

For source drift, use `okf:update` to inspect affected content before publishing new
versions. For schema/policy migration, back up dependencies, review the semantic
diff and replay histories on staging before guarded activation. Preserve historical
policy/content meanings; restoring a backup requires checking for later edits.

## Shared book library

Use `okf:retrieval` to inspect matching Sources and verify edition/byte identity.
Reuse an existing usable index for identical bytes. Changed bytes/editions need an
explicit update choice. Resolve paths as described in [runtime locations](plugin.md#runtime-locations).

1. Create missing library storage only on demand:
   `learning-workspace "$LEARNING_WORKSPACE" "$LEARNING_WORKSPACE/.local/bin/okf" --library`.
2. Choose stable source/run IDs and a new private staging directory:

   ```sh
   node "$LEARNING_PLUGIN_ROOT/scripts/ingest-book.mjs" prepare \
     --pdf BOOK.pdf --staging PRIVATE_STAGING_DIR \
     --source-id SOURCE --run-id RUN --title 'Book title' \
     --edition 'Edition' --language en
   ```

   Use `--existing-source` only after verifying that Source. Inspect `receipt.json`
   and page failures. `preflight_failed` has no usable digest/run; other prepared
   receipts remain unpublished. Do not rerun preparation over an existing receipt.
3. With the same helper, run `retain --staging PRIVATE_STAGING_DIR --bundle LIBRARY_PATH`,
   then `verify --staging PRIVATE_STAGING_DIR --retained RETAINED_PDF_PATH` immediately
   before publication. Retain and verify source bytes; reject drift.
4. Plan/publish the receipt's exact changeset using the procedure above. Publish
   course linkage separately with `learning.library` and applicable learner scope.
   If linkage fails after library success, retry linkage without re-indexing.

After interruption, inspect actual publication before retrying the same receipt/IDs.
Keep pending retained artifacts and receipts for recovery. A deliberately new
extraction gets a new run ID; explicit activation uses SourceSelection. Persist
review evidence in separate PassageReview records, never by rewriting source text.

### Retrieve book evidence

After reading bundle guidance, use the optional read-only helper:

```sh
node "$LEARNING_PLUGIN_ROOT/scripts/retrieve-book.mjs" --text 'query'
```

It uses the saved course selection. Resolve missing/ambiguous sources first;
`--help` describes explicit source and comparison options. Check complete result
summaries and open candidate passages with their page markers before grounding
an answer. Truncated/partial delivery cannot establish support or absence; narrow
scope or complete the scan. Query handling and attribution follow the
[book requirements](../learning-spec.md#41-book-library-and-grounded-questions).

## Visual learning assets

Retain visuals under the owning course or article Study's versioned
`references/visualizations/` paths, with CSS/JS/images alongside them. Publish the
artifact through OKF and link it from the relevant Topic/Lesson/CheatSheet to the
supporting sources and locators. Include title, topic, source attribution and a
readable explanation in the visual. Check navigation and relative dependencies
from the retained location; return that link. Temporary previews and project-level
documentation diagrams are not canonical learner assets.

For an update, use a new version path. When cleaning a duplicate, compare copies,
retain the owning canonical artifact, repair referring links and remove only the
confirmed redundant copy. Include the visual and dependencies in exports.

### Course maps

For a requested full course, enable `Course.course_map.enabled` through guarded
publication and use its accepted plan (or explicitly labeled `proposed_plan`).
The renderer also accepts optional display title/subtitle, topic labels and lesson
order; see [the renderer](../scripts/lib/course-map.mjs) for configuration handling.
After saved plan/lesson/session/assessment/revealed-sheet changes run:

```sh
node "$LEARNING_PLUGIN_ROOT/scripts/update-course-map.mjs" --bundle courses/COURSE
```

Use `--help` for options; `--check` is read-only and exit 1 means stale. Deliver the
course-local `course-map.html` entry. Keep the previous map if refresh fails and
report it stale. Maps are not supported for article studies. Resolve reported
bundle guidance before retrying. Export both the entry and retained snapshots.

## Export and explicit deletion

For a requested export, inventory outgoing references, provenance and artifacts
through scoped OKF retrieval. Copy dependencies with their relative layout or name
omitted targets in a manifest. Preserve bundle IDs, source locators, schema/policy
versions and exact learner text; exclude machine-local overrides. Validate copied
bundles and links before delivery. Library dependencies and visual assets must be
accounted for; do not imply a course-only export contains omitted book evidence.

For explicit deletion, inspect incoming references and clarify any additional
removal scope. Use an authorized OKF removal changeset and expected digest; normal
learning publication intentionally protects retained history. Retirement of an
assertion is not erasure. Recompute affected derived state afterwards. Inspect
backups/exports separately when erasure of those copies is requested; do not claim
to remove copies outside inspected locations.
