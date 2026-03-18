# ADR-01: Verb-Object CLI Naming Convention

## Status

Accepted

## Context

The original CLI used single-word commands (`push`, `pull`, `list`, `add`, `remove`, `merge`, `diff`, `exclude`). As the command set grew with machine management (`rename-machine`, `remove-machine`) and app management (`rename-app`), the single-word commands became ambiguous. `remove` could mean removing an app, a machine, or a file. `list` could list apps, rules, or presets.

## Decision

Adopt a consistent `verb-object` naming convention for all commands that operate on a specific resource type:

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

Commands that operate on a single obvious noun retain their existing names: `status`, `init`, `hook`, `self-update`, `history`, `restore`.

## Consequences

### Positive

- **Self-documenting**: `remove-app` vs `remove-machine` is immediately clear
- **Scalable**: New resource types (e.g. `list-presets`, `list-rules`) follow naturally
- **Consistent**: All app-related commands share the `-app` suffix
- **Tab-completion friendly**: Typing `drifters remove-` shows both `remove-app` and `remove-machine`

### Negative

- **Breaking change**: Users of the original commands must update scripts and muscle memory
- **Longer to type**: `push-app` is 4 characters longer than `push`
