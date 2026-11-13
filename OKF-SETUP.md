# OKF dependency setup

This project uses the external [abatyuk/okf](https://github.com/abatyuk/okf) CLI, pinned to **0.6.0** (OKF document specification 0.2). An optional upstream checkout at `okf/` is reference material only; it is not required at runtime. The runtime is `.local/bin/okf`; the active plugin lives in Codex’s plugin cache.

## Install the CLI

From the project root:

```sh
sh scripts/setup-okf.sh
export PATH="$PWD/.local/bin:$PATH"
okf version
```

The script downloads the matching macOS or Linux release, checks its SHA-256 against the release's checksum file, verifies its reported version, and installs it in `.local/bin/`. It requires internet access, curl, tar, and shasum. Rust is unnecessary. Downloaded tools are excluded from Git.

The PATH export applies to the current terminal. Start your agent from that terminal so its tools inherit this PATH. Otherwise use the absolute path to `.local/bin/okf`.

## Optional agent skills

The CLI works independently. To also install the upstream OKF skills in Codex, run:

```sh
codex plugin marketplace add abatyuk/okf --ref v0.6.0
codex plugin add okf@okf-marketplace
```

These commands install into Codex's own configuration/cache, not this Git repository. Start a new thread after installation. The upstream plugin supports CLI versions >=0.6.0 and <0.7.0. These are general OKF skills, not the learning-specific skills proposed in `learning-spec.md`.

## Verification and project data

```sh
okf version
okf schema
```

Create course bundles separately from `okf/`, for example under `courses/`, when implementing the learning workflow. No course bundle is initialized by this setup. Keep our course ontology and project configuration in Git; decide separately whether personal answers should be committed.

Installation is complete only after the executable version check succeeds. Preparing this script does not mean the binary or agent plugin has been installed.

## Workspace configuration

Use catalog v2 in `okf-catalog.yaml`, including `default_bundle` and local bundle
registrations. A legacy `okf.toml` is unnecessary when it only selects that catalog
and default. `ontology.yaml` remains in each bundle. Add `okf-bundle.yaml` only
when a bundle needs settings, guidance or query presets; an empty manifest is not
required. Personal configuration and migration backups remain Git-ignored.
