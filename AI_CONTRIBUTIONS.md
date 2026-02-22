# AI Contributions

This project was built with significant help from **Claude** (Anthropic's AI assistant), working as a software architect, code reviewer, and implementation partner throughout the development cycle. This document acknowledges that work and gives a transparent account of what was produced.

---

## The AI

**Model:** Claude Sonnet (Anthropic)
**Role:** Software architect, code reviewer, implementation partner
**Interface:** Claude Code (terminal-based agentic coding environment)

---

## What Claude Did

The collaboration covered the full development lifecycle — from initial codebase review through to feature implementation, safety fixes, and documentation:

### 1. Two Full Code Reviews

Two pre-release reviews were conducted, modelled on the kind of review a senior engineer would do before opening a codebase to the public.

**First review** identified 18 issues and produced fixes for all of them, spanning:
- Correctness bugs (consensus algorithm edge cases, pull failure propagation, registration logic)
- Safety issues (file locking, temp directory cleanup, hostname collision handling)
- Security (SHA-256 checksum verification for the self-update installer)
- Code quality (type fixes, deduplication, comment syntax detection, empty-commit guard)

**Second review** (pre-OSS, targeting new code) identified 14 issues, 8 of which were fixed immediately:
- `unwrap()` after `remove()` replaced with `ok_or_else(...)` in two places
- Defensive operation ordering in `rename-app` (TOML saved before directory rename)
- `expect()` added to a logically-safe match arm in `status.rs` for diagnostics
- Missing stale-machine-ID guard added to `add-app` and `remove-app`
- App name validation added (`/` and `\` rejected)
- Silent no-op message clarified in `add-app`

The remaining 6 issues were assessed as mitigated by the ephemeral-repo model (operations that fail mid-way are never committed to the remote) and deferred.

### 2. New Commands Implemented

| Command | Description |
|---|---|
| `rename-machine <old> <new>` | Renames a machine in the registry and across all app directories; updates local config if renaming self |
| `remove-machine <id>` | Removes a machine, deletes its uploaded configs, warns prominently if removing self |
| `rename-app <old> <new>` | Renames an app in sync-rules and the repo directory |
| `remove-app --machine <id>` | New flag: remove a specific machine's configs for an app |
| `remove-app --all` | New flag: remove an app from all machines entirely, with confirmation |

### 3. Stale Machine-ID Guard

Designed and implemented a cross-machine safety mechanism: if another machine runs `rename-machine` or `remove-machine` while the current machine is offline, the current machine's local config will reference a stale ID. The guard (`verify_machine_registration`) detects this on the next command, warns the user, lists the currently registered machines, and offers a choice to continue or exit. Added to `push-app`, `pull-app`, `status`, `diff-app`, `merge-app`, `add-app`, and `remove-app`.

### 4. CLI Naming Convention

Proposed and implemented a consistent `verb-object` naming convention across all app-management commands so that the intent of any command is immediately deducible:

`push` → `push-app` · `pull` → `pull-app` · `diff` → `diff-app` · `merge` → `merge-app` · `list` → `list-app` · `exclude` → `exclude-app` · `add` → `add-app` · `remove` → `remove-app`

### 5. Documentation

- Wrote `CHANGELOG.md` documenting the full development cycle
- Updated `README.md`: corrected all command names, expanded command table, updated security notes, updated roadmap

---

## Statistics

These numbers are drawn directly from the git history on the `claude/tender-napier` branch.

| Metric | Count |
|---|---|
| Pull requests opened & merged | **3** |
| Commits authored | **23** |
| Files modified | **34** |
| New files written from scratch | **7** |
| Lines of code added | **1,741** |
| Lines of code removed / replaced | **348** |
| Net new lines of code | **~1,393** |
| Bugs / issues identified (across 2 reviews) | **32** |
| Bugs / issues fixed | **26** |
| New CLI commands added | **5** |

### New Files Written From Scratch

| File | Lines | Purpose |
|---|---|---|
| `src/cli/rename_machine.rs` | 167 | `rename-machine` command |
| `src/cli/remove_machine.rs` | 138 | `remove-machine` command |
| `src/cli/rename_app.rs` | 118 | `rename-app` command |
| `src/cli/common.rs` | 71 | Shared stale-ID guard (`verify_machine_registration`) |
| `src/git/repo_layout.rs` | 53 | Deduplicated `collect_machine_versions` helper |
| `.github/workflows/release.yml` | 37 | CI workflow: auto-generate SHA-256 checksum on release |
| `CHANGELOG.md` | 160 | Full project changelog |
| **Total** | **744** | |

### Timeline

All work was completed on a single calendar day: **22 February 2026**.

| Period | Work |
|---|---|
| Previous sessions (carried over) | First code review (18 issues), fixes 1–11, SHA-256 self-update, initial PRs #1 and #2 |
| 10:58 – 11:34 | Fixes 12–18 (remaining review items), PR #3 merged |
| 11:34 – 13:56 | `rename-machine`, `remove-machine`, stale-ID guard |
| 13:56 – 14:55 | `remove-app` rewrite, `add-app` rename, `rename-app` |
| 14:55 – 15:28 | Second pre-OSS review; 8 additional fixes |
| 15:28 – 15:51 | `CHANGELOG.md`, `README.md` update |
| **Active coding span (this session)** | **~4 hours 53 minutes** |

The active commit span across both sessions is approximately **15–16 hours** of elapsed time (first session started on Feb 21), though wall-clock time included breaks and user discussion.

---

## Acknowledgement

> Working with Claude on this project was like having a senior engineer available around the clock — one who reads the entire codebase before touching a line, catches edge cases before they become bugs, and explains every decision. The new command structure, the safety guards, and the review process all came out of that collaboration.

— Thomas Jrsch, project author

---

## About Claude Code

Claude Code is Anthropic's terminal-based AI coding environment. It gives Claude direct access to the filesystem, git, and build tools, allowing it to operate as an autonomous coding agent — reading code, writing implementations, running builds, and iterating on feedback — within a conversation.

See: [claude.ai/claude-code](https://claude.ai/claude-code)
