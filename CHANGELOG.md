# Changelog

All notable changes to Drifters are documented here.

---

## [0.6.9] — 2026-03-20

### Changed

- All file listings (push, pull, status, diff, merge) now display the full local path alongside the filename
  - Example: `✓ settings.json (~/.config/zed/settings.json)` instead of just `✓ settings.json`
- Added `--dry-run` flag to `pull-app` command — shows what would change without writing any files (matching existing `merge-app --dry-run`)

---

## [0.6.7] — 2026-03-19

### Changed

- Renamed `set-preferred-editor` command to `set-editor`
- Renamed `preferred_editor` config key to `editor`
- Shell completion defaults to `zsh --install` on macOS when no shell is specified

---

## [0.6.0] — Editor, Edit-Rules, and Config Rename

### New Commands

- **`edit-rules`** — Opens `sync-rules.toml` in your editor and optionally saves changes to the repository
- **`set-editor`** (originally `set-preferred-editor`) — Set, show, or clear the preferred editor
- **`unlock`** — Force-remove a stale lock file left behind after a crash or Ctrl-C

### Changed

- Renamed config file from `config.toml` to `drifters.toml` for consistency
- Updated README with new command documentation

---

## [0.5.8] — Discover Presets, Open-Readme, Completion

### New Commands

- **`discover-presets`** — Auto-detect installed apps on the current machine and offer to add matching presets
- **`open-readme`** — Download and open the latest README from the repository
- **`completion [shell] [--install]`** — Generate or install shell completion scripts (bash, zsh, fish, powershell)

### Changed

- Removed the separate `list-apps` command (use `list-app` instead)
- Added `-V / --version` flag to print version and exit
- Added `editor` (originally `preferred_editor`) config option in `drifters.toml`

---

## [0.5.4] — Static Linking and Git Refactor

### Changed

- **Replaced `git2` crate with shell calls to system `git`** — eliminates C library dependencies (libgit2, OpenSSL), simplifies cross-compilation, and removes Homebrew dylib issues on macOS
- Statically linked OpenSSL to eliminate Homebrew dyld dependency (prior to removing git2)

### Infrastructure

- Added CI workflow to attach SHA-256 checksum sidecar (`drifters-installer.sh.sha256`) to each release for self-update verification

---

## [0.5.0] — CLI Overhaul, Machine Management, Code Review

### Summary

This release covers a comprehensive code review, bug fixes, three new CLI commands, and a CLI naming overhaul. All changes were developed as an AI-assisted session.

### New Commands

#### `rename-machine <old-id> <new-id>`
Renames a machine everywhere in the shared repo:
- Renames `apps/<app>/machines/<old>/` to `<new>/` for every app
- Updates the machine registry (`.drifters/machines.toml`)
- Updates any per-machine overrides in `sync-rules.toml`
- If renaming the current machine, updates the local config automatically

#### `remove-machine <machine-id>`
Removes a machine from the shared repo:
- Deletes `apps/<app>/machines/<id>/` for every app
- Removes the machine from the registry and from sync-rules overrides
- If removing the current machine, prompts with a prominent warning (default: NO)

#### `rename-app <old-name> <new-name>`
Renames an app everywhere in the shared repo:
- Renames `apps/<old>/` to `apps/<new>/`
- Updates the app key in `sync-rules.toml`
- Saves the TOML change before renaming the directory (defensive ordering)

#### `remove-app` — extended with `--machine` / `--all` scoping
- `remove-app <app>` — removes this machine's uploaded configs
- `remove-app <app> --machine <id>` — removes a specific machine's configs
- `remove-app <app> --all` — deletes the app entirely and removes from sync-rules

### CLI Naming — Verb-Object Convention

All app-management commands renamed to follow a consistent `verb-object` pattern:

| Old name | New name |
|---|---|
| `push` | `push-app` |
| `pull` | `pull-app` |
| `list` | `list-app` |
| `exclude` | `exclude-app` |
| `diff` | `diff-app` |
| `merge` | `merge-app` |
| `add` | `add-app` |
| `remove` | `remove-app` |

### Stale-Machine-ID Detection

A startup guard (`verify_machine_registration`) added to `push-app`, `pull-app`, `status`, `diff-app`, `merge-app`, `add-app`, and `remove-app`. If the local machine ID is no longer present in the shared registry, the user is warned and offered the choice to continue or exit.

### Bug Fixes (Pre-OSS Code Review)

1. `rename_machine.rs` — replaced `.unwrap()` after `remove()` with `.ok_or_else(...)`
2. `rename_app.rs` — same `.unwrap()` fix
3. `rename_app.rs` — `sync-rules.toml` saved before directory rename (atomicity)
4. `status.rs` — replaced `.unwrap()` with `.expect("...")` for clearer diagnostics
5. `add.rs` — added stale-ID guard before mutation
6. `remove.rs` — added stale-ID guard
7. `add.rs` — added `app_name` validation (rejects empty, `/`, `\`)
8. `add.rs` — duplicate app now prints "no changes made" instead of silent no-op
9. `config/local.rs` — `LocalConfig::load()` returns `Err` for missing file instead of silent default
10. `config/machines.rs` — `MachineRegistry::load()` same improvement
11. `cli/init.rs` — validates machine ID is non-empty and contains no path separators
12. `git/operations.rs` — skips empty commits (tree OID comparison)
13. `git/safety.rs` — `confirm_operation` retries up to 3 times; defaults to `false`
14. `git/operations.rs` — `index.add_all(["*"])` changed to `["."]`
15. `cli/pull.rs` — diff preview raised from 10 to 40 lines with accurate hidden-line count
16. `parser/sections.rs` — returns `Err` for unclosed `drifters::exclude::start` blocks
17. `config/local.rs` — `last_update_check` changed from `Option<String>` to `Option<u64>` with migration

### Security

- **Self-update checksum verification**: `drifters self-update` downloads a SHA-256 sidecar and verifies the installer before executing. Releases without a sidecar are rejected unless `--skip-checksum` is passed.

---

## [0.1.0] — Initial Release

- Core sync operations: `init`, `add-app`, `push-app`, `pull-app`
- Intelligent consensus-based merging
- Section tags (`drifters::exclude::start` / `stop`) with multi-comment-style support
- Glob pattern includes/excludes
- OS-specific rules (`include-macos`, `include-linux`, `include-windows`)
- Per-machine overrides in `sync-rules.toml`
- `merge-app` command for re-applying rules after config edits
- Ephemeral repository strategy (clone, operate, push, delete)
- Import/export commands for app definitions and entire rule sets
- Version history (`history`) and point-in-time restore (`restore`)
- `diff-app` for previewing pending changes without applying them
- `status` for per-file sync state overview
- `hook` for optional background auto-pull on shell startup
- `self-update` mechanism with automatic periodic update checks
- App presets (Zed, VS Code, Cursor, Windsurf)
- `list-presets` / `load-preset` for community preset loading
