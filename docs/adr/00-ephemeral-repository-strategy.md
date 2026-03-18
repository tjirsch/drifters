# ADR-00: Ephemeral Repository Strategy

## Status

Accepted

## Context

Drifters needs to interact with a remote Git repository to store and synchronize configuration files across machines. The question is whether to maintain a persistent local clone of this repository or to clone it fresh for each command invocation.

A persistent clone would be simpler to implement and faster for repeated operations. However, it introduces stale state: if another machine pushes changes between commands, the local clone becomes outdated. The user would need to manually pull or the tool would need background sync.

## Decision

Use an ephemeral repository strategy: clone the remote repository to a temporary location (`~/.config/drifters/tmp-repo/`) at the start of each command, perform the operation, commit and push, then delete the temporary clone.

## Consequences

### Positive

- **Always fresh state**: Every command starts from the latest remote state; no stale data
- **No persistent disk usage**: The repository only exists during command execution
- **No background processes**: No daemon or watcher needed to keep state in sync
- **Simpler mental model**: Each command is a self-contained transaction
- **Crash-safe**: If a command crashes mid-operation, the next command starts clean (the `unlock` command handles leftover lock files)

### Negative

- **Slower per-command**: Each command pays the cost of a full `git clone`
- **Network dependency**: Every command requires network access to the remote
- **No offline operation**: Commands fail if the remote is unreachable
- **Repeated authentication**: Each clone triggers SSH key authentication

### Mitigations

- The config repositories are typically small (a few TOML and JSON files), so clone time is negligible
- SSH agent caching handles repeated authentication transparently
