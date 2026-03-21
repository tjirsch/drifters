# ADR 05: Selective merge-app and no_merge flag

## Status

Accepted

## Context

With the branch-per-machine architecture (ADR 04), `merge-app` uses `git merge` to merge an entire machine branch into main. This creates a problem: all apps on the machine branch are merged, even those with machine-specific configurations that should not be on main.

Users need:
1. The ability to merge a single app's files without affecting others
2. A way to mark apps that should never be merged to main

## Decision

### Selective merge

When `merge-app <app>` is given a specific app name, use `git checkout origin/<branch> -- apps/<app>/` (pathspec checkout) instead of `git merge`. This copies only that app's files from the machine branch to main — no three-way merge, no conflict resolution needed.

When `merge-app` is called without an app name and no `no_merge` apps exist, the existing full `git merge` behavior is preserved (with conflict resolution via `git mergetool`).

### no_merge flag

Add `no_merge = true` to `AppConfig` (not `MachineOverride` — this is per-app, not per-machine):

```toml
[apps.claude-code]
include = ["~/.claude/*"]
no_merge = true
```

When any `no_merge` apps exist, `merge-app` without an app name merges only the remaining apps selectively (using pathspec checkout for each), rather than doing a full branch merge that would include the `no_merge` apps.

## Consequences

### Positive

- **Consistent**: `merge-app <app>` is now selective like `push-app <app>` and `pull-app <app>` already are
- **Machine-specific apps**: Apps like claude-code or machine-specific tools stay on their branch without polluting main
- **Simple mental model**: "with app name = selective, without = everything mergeable"

### Negative

- **No three-way merge on selective**: Selective merge takes the machine branch version wholesale — no conflict detection for individual apps
- **Two merge strategies**: Full merge (git merge) and selective merge (pathspec checkout) behave differently

### Mitigations

- `--dry-run` works for both modes, showing what would change before applying
- Users who need conflict resolution for a single app can do a full merge and resolve conflicts via mergetool
