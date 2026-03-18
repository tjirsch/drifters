# ADR-02: System Git over git2 Crate

## Status

Accepted

## Context

Drifters initially used the `git2` Rust crate (libgit2 bindings) for all Git operations. This caused several problems:

1. **OpenSSL dependency**: `git2` links against OpenSSL, which on macOS requires Homebrew's `openssl`. Pre-built binaries failed with `dyld` errors when the user's Homebrew path differed or OpenSSL was absent.
2. **Static linking complexity**: Statically linking OpenSSL solved the dyld issue but increased binary size and build complexity.
3. **SSH agent quirks**: `git2`'s SSH transport behaves differently from the system `git` SSH agent forwarding, causing authentication failures in some environments.
4. **Credential helper incompatibility**: `git2` does not use the system's Git credential helpers, so HTTPS-based repos required extra configuration.

## Decision

Replace all `git2` usage with shell calls to the system `git` binary. A thin `run_git()` wrapper function executes `git` commands via `std::process::Command` and captures stdout/stderr.

## Consequences

### Positive

- **Zero C dependencies**: The binary has no native library requirements; it runs anywhere a Rust binary runs
- **Cross-compilation simplified**: No need to cross-compile OpenSSL or libgit2 for each target
- **Authentication just works**: System `git` uses the user's configured SSH keys, credential helpers, and agent forwarding
- **Smaller binary**: Removing `git2` and its transitive dependencies reduces the compiled binary size
- **Behavior parity**: Users get exactly the Git behavior they configured (e.g. `includeIf`, signing, custom merge drivers)

### Negative

- **Runtime dependency on `git`**: The system must have `git` installed (practically universal)
- **Output parsing**: Some operations require parsing `git` text output rather than using structured APIs
- **Error handling**: Shell command errors produce text messages rather than typed errors

### Mitigations

- Git is a prerequisite anyway (Drifters is a Git-based sync tool)
- The `run_git()` wrapper provides consistent error formatting
