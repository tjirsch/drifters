# ADR 04: Branch-per-machine architecture

## Status

Accepted

## Context

Drifters originally stored all machine configs in a flat directory structure (`apps/<app>/machines/<machine_id>/`) on a single branch (`main`). Merging was automatic using a last-write-wins strategy based on git commit timestamps.

This design had several limitations:

- **No selective sync**: All machines were always merged together; there was no way to exclude a machine from the merged state.
- **No interactive conflict resolution**: The automatic merge strategy silently resolved conflicts with timestamps, with no option for manual review.
- **No per-machine history**: All machines' changes were interleaved on a single branch, making it hard to track what a specific machine changed.
- **No explicit control**: Merging to main was automatic on push, with no deliberate merge step.

## Decision

Adopt a **branch-per-machine** architecture:

- Each machine gets its own git branch: `machines/<machine_id>`
- `main` serves as the merged/common state
- **push-app** only pushes to the machine's branch (never directly to main)
- **merge-app** is a separate, explicit command that merges a machine branch into main using git's native merge
- **pull-app** pulls from main by default, with `--from <machine>` to pull from a specific machine's branch
- Conflict resolution uses git's native `merge` + `mergetool`
- Files are stored flat on each branch: `apps/<app>/<filename>` (no `machines/` subdirectory)

### Singular machines

A `singular: bool` flag on `MachineOverride` marks machines that should never merge to main. The `merge-app` command refuses to merge singular branches. These machines can still push to their branch and pull from main or other branches.

## Consequences

### Positive

- **No-sync machines**: Machines can stay on their branch and never pollute main
- **Real interactive merging**: Uses git's native merge and mergetool for conflict resolution
- **Per-machine history**: Each branch tracks its own commit history independently
- **Explicit control**: Merging to main is a deliberate action, not automatic
- **Simpler repo layout**: Flat `apps/<app>/` per branch, no nested `machines/` directories

### Negative

- **More branches**: Repository accumulates one branch per machine
- **Extra step**: Users must explicitly run `merge-app` to share configs

### Extended by

- [ADR 05: Selective merge-app and no_merge flag](05-selective-merge-and-no-merge.md) — adds per-app selective merging and `no_merge` flag

### Removed

- `--yolo` flag removed (no more automatic merge without confirmation)
- `src/merge/intelligent.rs` removed (git handles merging natively)
- `MachineVersion` and `collect_machine_versions` removed (branch replaces directory scanning)
