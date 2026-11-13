# Researcher plugin

Requires local Codex with shell access, Rust/Cargo, the OKF skills and pinned OKF
CLI 0.6.0. Node 20+ is needed for book/map helpers; PDF extraction additionally
requires macOS Swift/PDFKit. The plugin does not require its own model API key.

## Build and install

From the source checkout:

```sh
sh scripts/package-plugin.sh
codex plugin marketplace add "$PWD"
codex plugin add researcher@researcher-local
codex plugin marketplace add abatyuk/okf --ref v0.6.0
codex plugin add okf@okf-marketplace
```

Build before registering the marketplace. It points at `.local/plugin-package/researcher`,
not the checkout root; packaging must exclude personal runtime and build caches.
OKF skills and its executable are separate dependencies. Start a new chat after
installation and select the intended local or installed skill copy. Rebuild and
refresh/reinstall the package after changes; do not edit installed caches.

For repository-only development, `sh scripts/install-learning-skills.sh` installs
local discovery links. The fallback entry point is `skills/coordinate-learning/SKILL.md`.

## Runtime locations

Derive `LEARNING_PLUGIN_ROOT` from the loaded `skills/<name>/SKILL.md`: resolve
symlinks and use the root containing `skills/`. `LEARNING_WORKSPACE` is the selected
data directory, never the installed plugin/cache. Export both for commands that
need them; exports do not persist between unrelated shell sessions.

| Resource | Location |
| --- | --- |
| Skills, docs, schemas, templates, policies and helpers | `$LEARNING_PLUGIN_ROOT/…` |
| Learner/course bundles, catalog and `learning.local.json` | `$LEARNING_WORKSPACE/…` |
| Installed commands | `$LEARNING_WORKSPACE/.local/bin/…` |
| Builds and caches | Workspace `.local/` |

Resolve Markdown links from their containing document. Run commands from the
workspace and prepend its `.local/bin` to PATH before OKF preflight. Use absolute
executable/bundle paths in JSON: shell variables are not expanded there. A source
checkout may use `target/debug/learning-rules` instead of the installed command.
The checkout's AGENTS.md does not automatically apply to another workspace;
respect that workspace and the learner's explicit preferences.

## First-run setup

Select the data folder before initializing. For a new workspace or missing tools:

```sh
sh "$LEARNING_PLUGIN_ROOT/scripts/setup-learning.sh" "$LEARNING_WORKSPACE"
```

Setup verifies/downloads OKF, builds the commands and initializes only missing
learner storage/catalog/configuration. It creates no selected course or inferred
preferences. It updates `.gitignore` but does not untrack already committed files.
Existing source-checkout tools do not require setup solely because their paths differ.
To reuse a verified binary and cached dependencies:

```sh
sh "$LEARNING_PLUGIN_ROOT/scripts/setup-learning.sh" "$LEARNING_WORKSPACE" \
  --okf /absolute/path/to/okf --offline
```

An explicit `CARGO_HOME` is honored; otherwise workspace `.local/cargo` is used.
Setup does not install Rust. Missing registered data, schema/catalog conflicts or
local OKF overrides require inspection rather than reset. Setup is single-writer,
not an atomic multi-file migration: inspect partial failures before retrying.
Repeated setup preserves saved selection/data; adding a learner registration saves
an exact catalog backup and refuses conflicting backups. Upgrades rebuild workspace
commands; uninstalling the plugin does not delete learning data.

Book-library initialization remains on demand through
[library operations](learning-operations.md#shared-book-library). Course/study
creation follows [prepare-course](../skills/prepare-course/SKILL.md); runtime
publication and recovery follow [operations](learning-operations.md).
