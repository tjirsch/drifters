# Changelog

All notable changes to Drifters are documented here.

---

## [Unreleased] — claude/tender-napier

### Summary

This development cycle was conducted as an AI-assisted session using Claude (Anthropic). The work covered a comprehensive code review of the entire codebase, a full round of bug fixes, three new CLI commands, and a CLI naming overhaul. All changes were committed to the `claude/tender-napier` branch.

---

### New Commands

#### `rename-machine <old-id> <new-id>`
Renames a machine everywhere in the shared repo:
- Renames `apps/<app>/machines/<old>/` → `<new>/` for every app
- Updates the machine registry (`.drifters/machines.toml`)
- Updates any per-machine overrides in `sync-rules.toml`
- If renaming the current machine, updates the local config automatically

#### `remove-machine <machine-id>`
Removes a machine from the shared repo:
- Deletes `apps/<app>/machines/<id>/` for every app
- Removes the machine from the registry and from sync-rules overrides
- If removing the current machine, prompts with a prominent warning (default: NO) and deletes the local config on confirmation

#### `rename-app <old-name> <new-name>`
Renames an app everywhere in the shared repo:
- Renames `apps/<old>/` → `apps/<new>/`
- Updates the app key in `sync-rules.toml`
- Saves the TOML change before renaming the directory (defensive ordering)
- Affects all machines; they see the new name on their next sync

#### `remove-app` — extended with `--machine` / `--all` scoping
`remove-app` now supports three modes:
- `remove-app <app>` — removes this machine's uploaded configs; app stays in sync-rules for other machines
- `remove-app <app> --machine <id>` — same for the named machine; validates machine in registry
- `remove-app <app> --all` — deletes `apps/<app>/` entirely and removes from sync-rules; requires confirmation (default: NO)

---

### CLI Naming — Verb-Object Convention

All app-management commands were renamed to follow a consistent `verb-object` pattern so the intent is immediately deducible:

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

Commands that operate on a single obvious noun (`status`, `init`, `hook`, `self-update`, `history`, `restore`) retain their existing names.

---

### Stale-Machine-ID Detection

A startup guard (`verify_machine_registration`) was added to `push-app`, `pull-app`, `status`, `diff-app`, `merge-app`, `add-app`, and `remove-app`. If the local machine ID is no longer present in the shared registry (because another machine ran `rename-machine` or `remove-machine` while this machine was offline), the user is warned and offered the choice to continue or exit. This prevents silently writing data under a stale/unregistered ID.

---

### Bug Fixes (Pre-OSS Code Review)

A pre-release code review identified and fixed the following issues:

1. **`rename_machine.rs` — `unwrap()` after `remove()`**
   Replaced `.unwrap()` with `.ok_or_else(...)` so a logic error produces a clear message instead of a panic.

2. **`rename_app.rs` — same `unwrap()` pattern**
   Same fix applied.

3. **`rename_app.rs` — atomicity ordering**
   `sync-rules.toml` is now saved *before* the directory is renamed on disk. If the TOML save fails the directory remains at its original path and nothing gets committed to the remote.

4. **`status.rs` — `unwrap()` inside match arm**
   Replaced with `.expect("…")` to make the invariant explicit and produce a useful diagnostic if it is ever violated.

5. **`add.rs` — missing stale-ID guard**
   `verify_machine_registration()` is now called before any mutation so that adding an app while the local machine ID is stale is caught early.

6. **`remove.rs` — missing stale-ID guard**
   Same guard added to `remove-app`.

7. **`add.rs` — missing `app_name` validation**
   `add-app` now rejects names that are empty or contain `/` or `\` (which would create unintended directory nesting).

8. **`add.rs` — silent no-op for duplicate app**
   The "already configured" exit now prints `— no changes made.` to make the no-op explicit.

---

### Earlier Improvements (Pre-Review Fixes — same branch)

These fixes were applied before the major new features, addressing issues found in an initial codebase review:

| # | Area | Fix |
|---|------|-----|
| 1 | `config/local.rs` | `LocalConfig::load()` returns `Err` for missing file instead of silently defaulting |
| 2 | `config/machines.rs` | `MachineRegistry::load()` same improvement |
| 3 | `cli/init.rs` | `resolve_machine_id` validates ID is non-empty and contains no path separators |
| 4 | `git/operations.rs` | `commit_and_push` skips empty commits (tree OID comparison) |
| 5 | `git/safety.rs` | `confirm_operation` retries up to 3 times on unrecognised input; defaults to `false` after 3 failures |
| 6 | `git/operations.rs` | `index.add_all(["*"])` → `index.add_all(["."])` to avoid staging files outside the working tree |
| 7 | `cli/pull.rs` | Diff preview raised from 10 lines to 40; shows exact count of hidden lines |
| 8 | `parser/sections.rs` | Returns `Err` for unclosed `drifters::exclude::start` blocks (previously silently ignored) |
| 9 | `config/local.rs` | `last_update_check` changed from `Option<String>` to `Option<u64>`; migration deserialiser handles old string-format values |
| 10 | `self_update.rs` | Removed now-unnecessary `.parse().unwrap_or(0)` roundtrip |

---

### Security

- **Self-update checksum verification**: `drifters self-update` now downloads a SHA-256 sidecar (`drifters-installer.sh.sha256`) from the release and verifies the installer before executing it. Releases without a sidecar are rejected unless `--skip-checksum` is passed explicitly. The GitHub Actions workflow (`release.yml`) auto-generates and attaches the sidecar on every published release.

---

## [0.1.0] — Initial Release

- Core sync operations: `init`, `add-app`, `push-app`, `pull-app`
- Intelligent consensus-based merging
- Section tags (`drifters::exclude::start` / `stop`) with multi-comment-style support
- Glob pattern includes/excludes
- OS-specific rules (`include-macos`, `include-linux`, `include-windows`)
- Per-machine overrides in `sync-rules.toml`
- `merge-app` command for re-applying rules after config edits
- Ephemeral repository strategy (clone → operate → push → delete)
- Import/export commands for app definitions and entire rule sets
- Version history (`history`) and point-in-time restore (`restore`)
- `diff-app` for previewing pending changes without applying them
- `status` for per-file sync state overview
- `hook` for optional background auto-pull on shell startup
- `self-update` mechanism with automatic periodic update checks
- App presets (Zed, VS Code, Cursor, Windsurf)
- `list-presets` / `load-preset` for community preset loading
