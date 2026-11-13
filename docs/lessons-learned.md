# Agent harness lessons learned

Record reusable lessons about the agent harness: its instructions, tools,
plugins, execution environment and work process. Keep entries concise and in
English. Add an entry when an unexpected harness problem reveals how future
agent runs should be changed; routine transient errors do not need an entry.
This file is not for ordinary bugs in the learning project or issues in course
content and learner data.

For each lesson, include the date, affected harness component, symptom, cause,
resolution and a practical check or prevention step. Do not include secrets or
personal data.
Put lasting architecture choices in [ADRs](adr/README.md)
and notable project changes in [the changelog](../CHANGELOG.md).

<!-- Add dated lessons below using this structure:
## YYYY-MM-DD — Short title

- Context: ...
- Symptom: ...
- Cause: ...
- Resolution: ...
- Future check: ...
-->

## Approval scope before documentation restructuring — 2026-11-12

A request questioning document placement was interpreted as authorization to move
files, although the discussion concerned a proposed documentation model. Confirm
the intended scope in chat before mutations; read-only repository analysis can
continue independently. Once the user approves a concrete scope, carry it through
without repeated approval prompts. The project entry-point instructions now make
this boundary explicit for future sessions.

## Swift module cache path identity — 2026-11-11

A PDFKit helper check reused a Swift module cache through lexically different
workspace paths (including `scripts/..`). Swift's cached module identity retained
one path spelling and rejected reuse through the other. Normalize the workspace
with `pwd -P` before choosing the helper cache directory, and give isolated PDF
checks their own workspace/cache. When extraction fails during module loading,
inspect cache path identity before diagnosing the PDF or changing book records.
Normalizing the path does not repair an already incompatible cache: retain that
cache separately and let Swift build a fresh cache at the canonical path.

## OKF required empty lists — 2026-11-11

Guarded publication of a synthetic extraction manifest failed because OKF 0.6.0
ontology lint treats a required empty list as missing. Coverage lists must be
allowed to be empty (for example, a complete run has no failed pages). Declare
these fields optional in the ontology and enforce their actual array presence,
disjoint ranges and exhaustive coverage in the library publication adapter. Test
helper-generated records through the real staging/lint path rather than inferring
compatibility from `ontology apply` success. Do not weaken the application contract
or populate empty lists with invented page ranges to satisfy lint.

## OKF stored ontology versus apply input — 2026-11-11

A live schema extension prepared from an existing ontology file was rejected by
OKF 0.6.0 because `ontology apply --from` accepts a changes document, not every
stored-file key. In particular, the stored `okf_ontology` header is rejected as an
unknown changes key. Preserve the original ontology in the backup, send the
compatible `concepts` mapping as the apply payload, and let OKF retain its storage
header. Compare the schema's semantic changes and reload bundle context after
application. Do not treat a valid stored configuration as proof of a valid write
request, and do not bypass the CLI with an unreviewed whole-file replacement.
