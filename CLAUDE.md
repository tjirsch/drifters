# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
cargo test -- --nocapture # Run tests with stdout output
cargo install --path .   # Install locally
```

Releases use `cargo-dist` (config in `dist-workspace.toml`) and `cargo-release` (config in `release.toml`). CI runs via `.github/workflows/release.yml`.

## Architecture

Drifters is a Rust CLI that syncs config files across machines using Git as the transport layer. It shells out to the system `git` binary (no git2/libgit2 dependency).

### Branch-per-machine model

Each machine gets its own git branch (`machines/<machine_id>`). `main` is the merged/common state. The workflow is:

1. `push-app` — pushes local configs to the machine's branch only
2. `merge-app` — merges the machine branch into main (uses git's native merge + mergetool for conflicts)
3. `pull-app` — pulls from main (default) or `--from <machine>` to pull from a specific branch

Machines marked `singular: true` in sync-rules.toml can push/pull but `merge-app` refuses to merge them into main.

When `merge-app <app>` is given a specific app name, it performs a selective merge (checkout pathspec) instead of a full git merge. Apps with `no_merge = true` in `AppConfig` are automatically excluded from full-branch merges.

### Core Modules

- **`src/main.rs`** — CLI definition using clap derive. All commands defined in `Commands` enum, dispatched in `run()`. Global flag: `--verbose`.
- **`src/cli/`** — One file per command (e.g., `push.rs`, `pull.rs`, `add.rs`). `common.rs` has shared helpers.
- **`src/config/`** — Configuration types:
  - `local.rs` — `LocalConfig`: per-machine config at `~/.config/drifters/drifters.toml` (machine_id, repo_url, update settings, editor)
  - `sync_rules.rs` — `SyncRules`/`AppConfig`/`MachineOverride`: the shared repo config at `.drifters/sync-rules.toml`. `MachineOverride` has a `singular: bool` field. `AppConfig` has a `no_merge: bool` field.
  - `fileset.rs` — Glob pattern resolution for include/exclude rules
  - `machines.rs` — `MachineRegistry` for machine ID tracking. `MachineInfo` includes `branch: Option<String>`.
- **`src/git/`** — Git operations:
  - `operations.rs` — Low-level git commands via `git_run()` helper (clone, pull, commit, push, branch operations, merge, mergetool)
  - `ephemeral.rs` — `EphemeralRepoGuard` (RAII): clones repo to `~/.config/drifters/tmp-repo`, acquires a lock file, cleans up on drop. Supports `new()` (stays on main) and `new_on_branch()` (checks out a specific branch).
  - `repo_layout.rs` — `read_app_files()` reads flat `apps/<app>/` directory on current branch
  - `safety.rs` — File safety checks, user confirmation prompts
- **`src/parser/sections.rs`** — Section tag parsing (`drifters::exclude::start/stop`). Extracts syncable content, merges synced content back preserving local exclude blocks.
- **`src/error.rs`** — `DriftersError` enum with `thiserror`, custom `Result<T>` type. Includes `MergeConflict` variant.
- **`src/sync/`** — Placeholder module.

### Key Patterns

- **Ephemeral repo**: Every command clones/pulls the repo fresh, operates, commits+pushes, then deletes. `EphemeralRepoGuard` manages this lifecycle with a lock file to prevent concurrent corruption.
- **Branch-per-machine**: Each machine's configs live on `machines/<machine_id>` branch. Files stored flat at `apps/<app>/<filename>`. Rules live at `.drifters/sync-rules.toml` on main.
- **Three-level rule hierarchy**: App defaults → OS-specific rules → Machine-specific overrides. Resolved in `fileset.rs`.
- **Section tags**: Files can contain `drifters::exclude::start/stop` blocks. Content inside these blocks stays local and is never synced.
- **Git-native merging**: `merge-app` uses `git merge` + `git mergetool` for conflict resolution instead of custom merge logic.

### Error Handling

Uses `thiserror` for the error enum and the crate's own `Result<T>` type (`error::Result`). Not `anyhow` — errors are structured variants of `DriftersError`.

## Documentation

- ADRs in `docs/adr/` using MADR format
- Presets for apps in `presets/` directory (TOML files)
- Detailed import/export guide at `docs/IMPORT_EXPORT.md`

## Migration
- No code to support older versions or state is needed
