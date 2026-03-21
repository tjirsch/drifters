# ADR-03: Consensus-Based Merging

## Status

Superseded by [ADR 04: Branch-per-machine architecture](04-branch-per-machine.md)

## Context

When multiple machines modify the same configuration file, a merge strategy is needed. Common approaches:

1. **Last-write-wins**: The most recent push overwrites all others. Simple but destructive.
2. **Three-way merge**: Uses a common ancestor to resolve conflicts. Requires tracking merge bases per file, complex for multi-machine scenarios (more than two divergent versions).
3. **Consensus (majority wins)**: Collect all machine versions, group identical content, pick the version with the most supporters.

## Decision

Use consensus-based merging: when pulling, Drifters collects the file version from every machine in `apps/<app>/machines/*/`, groups them by content, and picks the version held by the majority. On ties, the current machine's version is preferred. Exclude sections (`drifters::exclude::start` / `stop`) are stripped before comparison and reinserted from the local copy after merge.

## Consequences

### Positive

- **Multi-machine aware**: Handles 3+ machines naturally, not just pairwise
- **Predictable**: The "most machines agree" heuristic is intuitive
- **Non-destructive**: A single machine's accidental change doesn't override a stable consensus
- **Exclude-section safe**: Local-only sections are preserved regardless of merge outcome
- **No merge base tracking**: Each merge is computed from the current state, no history graph required

### Negative

- **Majority bias**: If 2 of 3 machines have stale data and 1 has the correct update, the stale version wins
- **No line-level merge**: The comparison is per-file (with exclude sections stripped), not per-line
- **Tie-breaking is implicit**: On a 1-vs-1 tie the current machine wins, which may not always be the desired outcome

### Mitigations

- Users can use `diff-app` to preview merge results before applying
- `merge-app --machine <id>` allows merging from a specific machine's state
- `merge-app --dry-run` shows what would change without applying
