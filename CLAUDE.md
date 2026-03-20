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

### Core Modules

- **`src/main.rs`** — CLI definition using clap derive. All commands defined in `Commands` enum, dispatched in `run()`. Global flags: `--verbose`, `--yolo`.
- **`src/cli/`** — One file per command (e.g., `push.rs`, `pull.rs`, `add.rs`). `common.rs` has shared helpers. Entry points are public functions like `push_command()`, `pull_command()`.
- **`src/config/`** — Configuration types:
  - `local.rs` — `LocalConfig`: per-machine config at `~/.config/drifters/drifters.toml` (machine_id, repo_url, update settings, editor)
  - `sync_rules.rs` — `SyncRules`/`AppConfig`/`MachineOverride`: the shared repo config at `.drifters/sync-rules.toml`
  - `fileset.rs` — Glob pattern resolution for include/exclude rules
  - `machines.rs` — `MachineRegistry` for machine ID tracking
- **`src/git/`** — Git operations:
  - `operations.rs` — Low-level git commands via `git_run()` helper (clone, pull, commit, push)
  - `ephemeral.rs` — `EphemeralRepoGuard` (RAII): clones repo to `~/.config/drifters/tmp-repo`, acquires a lock file, cleans up on drop. This is the central pattern — most commands create a guard, operate on the repo, then let it drop.
  - `repo_layout.rs` — Reads `apps/<app>/machines/<machine_id>/` directory structure, collects `MachineVersion` (content + commit timestamp)
  - `safety.rs` — File safety checks, user confirmation prompts
- **`src/parser/sections.rs`** — Section tag parsing (`drifters::exclude::start/stop`). Extracts syncable content, merges synced content back preserving local exclude blocks. Auto-detects comment syntax from file extension.
- **`src/merge/intelligent.rs`** — Merge strategy: last-write-wins by git commit timestamp, with tiebreakers (prefer current machine, then lexicographic).
- **`src/error.rs`** — `DriftersError` enum with `thiserror`, custom `Result<T>` type.
- **`src/sync/`** — Placeholder module (sync logic lives in `cli/` modules).

### Key Patterns

- **Ephemeral repo**: Every command clones/pulls the repo fresh, operates, commits+pushes, then deletes. `EphemeralRepoGuard` manages this lifecycle with a lock file to prevent concurrent corruption.
- **Three-level rule hierarchy**: App defaults → OS-specific rules → Machine-specific overrides. Resolved in `fileset.rs`.
- **Section tags**: Files can contain `drifters::exclude::start/stop` blocks (comment-style varies by file type). Content inside these blocks stays local and is never synced.
- **Repo layout**: Remote repo stores configs at `apps/<app>/machines/<machine_id>/<filename>`. Rules live at `.drifters/sync-rules.toml`.

### Error Handling

Uses `thiserror` for the error enum and the crate's own `Result<T>` type (`error::Result`). Not `anyhow` — errors are structured variants of `DriftersError`.

## Documentation

- ADRs in `docs/adr/` using MADR format
- Presets for apps in `presets/` directory (TOML files)
- Detailed import/export guide at `docs/IMPORT_EXPORT.md`
