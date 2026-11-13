# Researcher

A local learning assistant for Codex that teaches from your selected sources and
keeps courses, notes, answers, assessments and progress between conversations.
Researcher combines agent skills, file-based knowledge bases managed by
[OKF](https://github.com/abatyuk/okf), and Rust commands that validate learning
records and calculate progress and review schedules.

Use it to study an article, work through a book, resume a lesson, save an exact
note, or review previously studied material. Plans require learner acceptance;
discussing a topic and demonstrating understanding are tracked separately.
This is a local agent workflow, not a standalone web application. Nothing runs
while the chat is closed.

## Requirements

- Local Codex with shell access and plugin support.
- Rust/Cargo to build the learning commands; setup does not install Rust.
- macOS or Linux on ARM64 or x86_64, with Git, `curl`, `tar` and `shasum`.
- Internet access for the initial dependency downloads.
- Node.js 20+ for optional book retrieval and HTML course-map helpers.
- macOS with Swift/PDFKit for the included PDF text extraction helper. It does
  not perform OCR; scanned pages and diagrams require separate inspection.

Researcher does not require a separate model API key. It runs through your Codex
environment.

## Install and set up

### 1. Clone and install the Researcher plugin

```sh
git clone https://github.com/VarvaraTi/researcher.git
cd researcher
sh scripts/package-plugin.sh
codex plugin marketplace add "$PWD"
codex plugin add researcher@researcher-local
```

The repository is private, so cloning requires an account with access. Packaging
creates `.local/plugin-package/researcher` from an explicit list of project files;
it excludes personal learning data and build caches. Keep the checkout available
as the local marketplace source.

### 2. Install the OKF skills

OKF is required in two forms: its executable manages knowledge-base files, and
its Codex skills guide retrieval and changes to those files.

```sh
codex plugin marketplace add abatyuk/okf --ref v0.6.0
codex plugin add okf@okf-marketplace
```

**The next setup step automatically downloads and verifies OKF CLI 0.6.0. It does
not install the OKF skills above.** No upstream OKF checkout is needed. The OKF
plugin version is 0.6.0 and supports CLI versions `>=0.6.0,<0.7.0`; this project
pins the executable to 0.6.0. See [OKF setup](OKF-SETUP.md) for CLI-only installation.

### 3. Create a separate learning workspace

Run the following from the cloned repository. Choose a data directory outside
the repository and installed plugin directory:

```sh
export LEARNING_PLUGIN_ROOT="$PWD"
export LEARNING_WORKSPACE="$HOME/researcher-data"
sh "$LEARNING_PLUGIN_ROOT/scripts/setup-learning.sh" "$LEARNING_WORKSPACE"
export PATH="$LEARNING_WORKSPACE/.local/bin:$PATH"
"$LEARNING_WORKSPACE/.local/bin/okf" version
```

Expected version: `okf 0.6.0 (OKF spec 0.2)`.

Setup downloads and checksum-verifies OKF, builds `learning-rules` and
`learning-workspace`, and initializes missing learner storage, catalog,
configuration and Git exclusions. It does not create a course, select sources,
or infer your preferences. Repeated setup preserves existing data and selection;
conflicts require inspection rather than a reset.

For this command, `LEARNING_PLUGIN_ROOT` points at the source checkout. Installed
skills resolve their own package root; `LEARNING_WORKSPACE` always points at your
data directory. Environment exports apply only to the current shell and its
children. See [runtime locations and offline setup](docs/plugin.md).

### 4. Start learning

Start a new Codex chat after installing the plugins, using the learning workspace
as the working directory. For a terminal session, launch Codex from the shell
where you set PATH:

```sh
cd "$LEARNING_WORKSPACE"
codex
```

Ask Codex to use the Researcher `coordinate-learning` skill, and provide your
selected material and goal. For example:

> Use Researcher to help me study this article: [URL]. My goal is to explain its
> main ideas and apply them to a practical example. Propose a learning plan first.

For a local book, provide its file path and the topics you want to study. The
assistant checks available source content, proposes a plan, and saves it after
your acceptance. Books, articles and courses are created only when needed.

## Everyday use

| Task | Example request |
| --- | --- |
| Resume | “Use Researcher to continue my selected course from the saved session.” |
| Study a book | “Help me study `/absolute/path/book.pdf`; focus on these chapters.” |
| Ask about a source | “Explain this concept using my selected book and cite the pages.” |
| Save a note | “Save this exact note under the current topic: …” |
| Review | “Review the topics that are due, using my saved learning history.” |
| Get a reference sheet | “Create a cheat sheet for the material we have covered.” |
| See a course map | “Enable an HTML learning map for this course.” |
| Update preferences | “Save my preference for shorter explanations and practical examples.” |

The coordinator routes these requests to the appropriate skill. Saved answers
can be assessed after a chat restart without asking you to submit them again.
Source answers should cite retained passages and disclose extraction gaps.
HTML maps are optional and supported for full courses, not standalone article
studies. Visuals and sheets stay with the course or study that owns them.

## Knowledge bases and their structure

An OKF **bundle** is a local knowledge base with Markdown records, structured
metadata, an `ontology.yaml` describing record types and relationships, and
optional `okf-bundle.yaml` settings. The workspace's `okf-catalog.yaml` registers
bundles so records can refer to sources and other records across bases.

| Knowledge base | Location in the workspace | Contents |
| --- | --- | --- |
| Shared learner profile | `learner/` | Preferences, goals, questionnaire responses and evidence-linked observations shared across courses. |
| Full courses | `courses/<course>/` | One bundle per curriculum: sources, topics, objectives, plans, lessons, exercises, sessions, original answers, assessments, notes and review records. |
| Standalone articles | `courses/articles/` (`learning.articles`) | One shared bundle; each article has a separate `studies/<slug>/study.md` and its own study-owned learning records. |
| Shared book library | `courses/library/` (`learning.library`) | Book editions and source identity, retained PDFs, extraction runs, page coverage and passages that multiple courses can reference. |

The layout below illustrates a workspace after these optional bases have been
created; initial setup creates only the learner base and workspace configuration.

```text
researcher-data/
├── okf-catalog.yaml          # Bundle registrations and default bundle
├── learning.local.json      # Selected course and, for articles, selected study
├── learner/                 # Shared learner knowledge base
├── courses/
│   ├── <course>/            # A full course and its learning history
│   ├── articles/
│   │   └── studies/<slug>/  # An individual article study and its records
│   └── library/             # Shared book sources and extraction evidence
└── .local/                  # Installed commands, build products and caches
```

A course links to shared book passages instead of owning a separate full-book
index. Learner preferences are shared, while answers and assessments stay with
their course or article study. A new chapter stays in its existing course.
Historical answers and source versions are retained; derived progress can be
recomputed from that history.

Canonical learner visuals use versioned `references/visualizations/` paths inside
their owning course or article study, with links to topics, lessons and sources.
Project diagrams under `docs/visualizations/` explain the software and contain no
personal learning records.

Personal bases, local configuration, exports and caches are Git-ignored and are
not included in the plugin package. GitHub stores the software, schemas,
templates and synthetic test fixtures. Keep separate backups of your learning
workspace: Git exclusions are not a backup, and do not untrack previously
committed files.

## Repository guide

| Path | Purpose |
| --- | --- |
| `skills/` | Coordinator and specialized learning workflows. |
| `crates/learning-rules/` | Rust validation, persistence, scheduling and workspace commands. |
| `schemas/`, `templates/`, `policies/` | Bundle models, starting configuration and versioned policies. |
| `scripts/` | Setup, packaging, validation, book ingestion/retrieval and course-map helpers. |
| `tests/` | Automated tests and synthetic fixtures; no personal runtime. |
| `docs/` | Operations, command contracts, plugin instructions and architecture decisions. |
| `learning-spec.md` | Current requirements; consult implementation and contracts for exact supported behavior. |

## Development and updates

For repository-only skill discovery, run `sh scripts/install-learning-skills.sh`.
The explicit fallback entry point is `skills/coordinate-learning/SKILL.md`.
Follow [AGENTS.md](AGENTS.md) before editing the project.

The check script uses `.local/cargo` and runs offline. Populate that dependency
cache on a fresh checkout, then run the checks:

```sh
CARGO_HOME="$PWD/.local/cargo" cargo fetch --locked
sh scripts/check-learning.sh
```

Checks cover Rust formatting, tests and Clippy; JavaScript tests run when Node
is available. Install Node 20+ to include the book and map checks.

After pulling project updates, rebuild with `sh scripts/package-plugin.sh` and
refresh/reinstall the Researcher plugin in Codex. Re-run workspace setup when
upgrading its commands, and start a new chat to load updated skills. Do not edit
installed plugin caches. Uninstalling the plugin does not delete learning data.

## Further documentation

- [Plugin installation, paths and recovery considerations](docs/plugin.md)
- [Pinned OKF dependency](OKF-SETUP.md)
- [Learning operations, shared books, maps and exports](docs/learning-operations.md)
- [Command contracts](docs/learning-contracts.md)
- [Learning specification](learning-spec.md)
- [Architecture decisions](docs/adr/README.md)
- [Changelog](CHANGELOG.md)
