# Drifters

> Intelligent configuration file synchronization across multiple machines

Drifters helps you keep your application configurations synchronized across all your machines (macOS, Linux, Windows) using Git as the transport layer. Unlike simple dotfile managers, Drifters provides **intelligent merging**, per-machine exceptions, and section-level control.

## Features

- 🔄 **Intelligent Merging** - Consensus-based merging from multiple machines, not just last-write-wins
- 🎯 **Selective Sync** - Use glob patterns and section tags for fine-grained control
- 🖥️ **Multi-OS Support** - OS-specific configurations with fallbacks
- 🔒 **Per-Machine Overrides** - Exclude files or sections on specific machines
- 📦 **No Daemon** - Manual control over when configs sync (with optional auto-pull hook)
- 🚀 **Fast & Safe** - Written in Rust with built-in safety checks
- 📝 **Section Tags** - Exclude sensitive or machine-specific sections within files
- ⚡ **Ephemeral Repository** - Cloned fresh on each command, zero persistence

## Quick Start

### Installation

```bash
# From GitHub releases (recommended)
curl -sSL https://github.com/tjirsch/drifters/releases/latest/download/drifters-installer.sh | sh

# From source
git clone https://github.com/tjirsch/drifters
cd drifters
cargo install --path .

# From crates.io (coming soon)
cargo install drifters
```

### Staying Updated

```bash
# Check for updates
drifters self-update --check-only

# Install latest release
drifters self-update
```

Drifters automatically checks for updates on most commands (configurable). Updates are installed via the same installer script used for initial installation.

### Prerequisites

- Git repository for storing configs (e.g., GitHub, GitLab)
- SSH keys configured for your Git hosting service

### Initialize on First Machine

```bash
# Create a GitHub repo first, then:
drifters init git@github.com:username/my-configs.git

# Add an app to sync (interactive)
drifters add-app zed
# Enter file patterns: ~/.config/zed/settings.json
# Or use a preset from presets/

# Push your current configs
drifters push-app
```

### Initialize on Additional Machines

```bash
# Clone and pull configs
drifters init git@github.com:username/my-configs.git

# Pull configs from other machines
drifters pull-app
```

## Core Concepts

### Section Tags

Mark sections of files that should NOT be synced:

```json
// ~/.config/zed/settings.json
{
  "theme": "One Dark",
  "vim_mode": true,

  // drifters::exclude::start
  "working_directory": "/Users/thomas/projects",
  "recently_opened": [...]
  // drifters::exclude::stop
}
```

Everything outside the `exclude` tags gets synced. The exclude sections stay local to each machine.

**Tag placement rules:**
- Tags must be on their **own line** — inline tags (after other content) are not recognized
- Leading whitespace before the comment character is allowed: `    # drifters::exclude::start` ✅
- The comment character must match the file type (auto-detected from extension; see [Supported Comment Styles](#supported-comment-styles))

### Three-Level Rule Hierarchy

Rules are resolved in this order:
1. **Machine-specific** overrides (highest priority)
2. **OS-specific** rules
3. **App defaults** (base configuration)

```toml
# .drifters/sync-rules.toml

[apps.zed]
# Base rules (all platforms)
include = ["~/.config/zed/settings.json"]

# macOS-specific
include-macos = ["~/Library/Application Support/Zed/settings.json"]

# Machine override
[apps.zed.machines.laptop]
exclude = ["**/keymap.json"]  # Different keyboard
```

### Intelligent Merging

When you pull, Drifters:
1. Collects config versions from ALL machines
2. Finds consensus (majority wins)
3. On ties, prefers current machine's version
4. Merges while preserving your local `drifters::exclude` sections

No "last write wins" - true multi-machine intelligence.

## Commands

| Command | Description |
|---------|-------------|
| `drifters init <repo-url>` | Initialize drifters on a machine |
| **App management** | |
| `drifters add-app <app>` | Add an app to sync (interactive) |
| `drifters remove-app <app>` | Remove this machine's configs for an app |
| `drifters remove-app <app> --machine <id>` | Remove a specific machine's configs |
| `drifters remove-app <app> --all` | Remove an app from all machines entirely |
| `drifters rename-app <old> <new>` | Rename an app everywhere in the repo |
| **Sync** | |
| `drifters push-app [app]` | Push local configs to repo |
| `drifters pull-app [app]` | Pull and merge configs from all machines |
| `drifters merge-app [app]` | Re-merge configs using current rules |
| `drifters diff-app [app]` | Show diff without applying changes |
| `drifters status` | Show per-file sync status |
| `drifters exclude-app <app> <file>` | Exclude a file on this machine |
| **Listing** | |
| `drifters list-app [app]` | List all configured apps (or details for one) |
| `drifters list-apps` | List app names with file counts |
| `drifters list-rules` | Print current sync-rules.toml |
| **Machine management** | |
| `drifters rename-machine <old> <new>` | Rename a machine everywhere in the repo |
| `drifters remove-machine <id>` | Remove a machine and delete its configs |
| **Import/Export** | |
| `drifters import-app <name> [--file <path>]` | Import app from file (defaults to ./<name>.toml) |
| `drifters export-app <name> [--file <path>]` | Export app to file (defaults to ./<name>.toml) |
| `drifters import-rules [--file <path>]` | Import rules (defaults to ./sync-rules.toml) |
| `drifters export-rules [--file <path>]` | Export rules (defaults to ./sync-rules.toml) |
| **Presets** | |
| `drifters list-presets` | List available presets from GitHub |
| `drifters load-preset <name>` | Load preset from GitHub repo |
| **History** | |
| `drifters history rules` | Show history of sync rules |
| `drifters history app <name>` | Show history of app definition |
| `drifters restore rules --commit <hash>` | Restore previous rules version |
| `drifters restore app <name> --commit <hash>` | Restore previous app version |
| **Automation** | |
| `drifters hook` | Generate shell hook for auto-pull |
| `drifters self-update` | Check for and install updates from GitHub |
| `drifters self-update --check-only` | Check for updates without installing |
| `drifters self-update --no-download-readme` | Install without downloading README |
| `drifters self-update --no-open-readme` | Download README but do not open it |
| `drifters open-readme` | Download latest README and open it |
| `drifters completion <shell>` | Print shell completion script to stdout |
| `drifters completion <shell> --install` | Install completion script to default location |
| `drifters set-preferred-editor <editor>` | Set preferred editor in `drifters.toml` |
| `drifters set-preferred-editor --clear` | Clear the preferred editor setting |
| `drifters set-preferred-editor` | Show current preferred editor setting |

### The `merge-app` Command

Useful when you've changed sync rules:

```bash
# Re-apply current rules to all configs
drifters merge-app

# Test merge for specific app
drifters merge-app zed --dry-run

# Debug: merge from only one machine
drifters merge-app --machine mac01 --dry-run

# Test OS-specific rules
drifters merge-app --os linux
```

### Flags

- `--yolo` - Skip all confirmations (use with caution)
- `-v, --verbose` - Show detailed logging

## Configuration (~/.config/drifters/drifters.toml)

User-level parameters live in **`~/.config/drifters/drifters.toml`**. This file is created automatically on first run with default values. If it is missing it is recreated with defaults.

| Option | Default | Description |
|--------|---------|-------------|
| `self_update_frequency` | `"always"` | When to auto-check for updates: `never`, `always`, or `daily` (at most once per 24 hours). The check is check-only — no install, no README. |
| `preferred_editor` | *(none)* | Editor command used to open files (e.g. `"zed"`, `"code"`, `"vim"`). Falls back to `$EDITOR` env var, then the OS default app. |

Example (optional; the file is created automatically):

```toml
self_update_frequency = "daily"
preferred_editor = "zed"
```

Use `drifters set-preferred-editor <editor>` to set the editor from the command line instead of editing the file directly.

### Shell Completion

Generate tab-completion for your shell (`bash`, `zsh`, `fish`, `powershell`):

```bash
# Print to stdout and add manually
drifters completion bash >> ~/.bash_completion
drifters completion zsh >> ~/.zshrc

# Auto-install to the canonical shell location
drifters completion zsh --install
# → installs to ~/.zsh/completions/_drifters
# → prints fpath setup instructions

drifters completion fish --install
# → installs to ~/.config/fish/completions/drifters.fish
```

**Install locations for `--install`:**

| Shell | Path |
|-------|------|
| bash | `~/.local/share/bash-completion/completions/drifters` |
| zsh | `~/.zsh/completions/_drifters` |
| fish | `~/.config/fish/completions/drifters.fish` |
| powershell | `%USERPROFILE%\Documents\PowerShell\Completions\drifters.ps1` |

For zsh, add this to `~/.zshrc` if not already present:
```zsh
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

## Configuration Example

```toml
# .drifters/sync-rules.toml

[apps.zed]
# Glob patterns supported
include = [
    "~/.config/zed/settings.json",
    "~/.config/zed/keymap.json",
    "~/.config/zed/**/*.json",
]

exclude = [
    "~/.config/zed/workspace-*.json",  # Session-specific
    "~/.config/zed/cache/**",          # Temporary files
]

# macOS-specific paths
include-macos = [
    "~/Library/Application Support/Zed/settings.json",
]

# Per-machine overrides
[apps.zed.machines.laptop]
exclude = ["**/keymap.json"]  # Laptop has different keyboard
```

## Use Cases

### 1. Sync Vim Config with Local Plugins

```vim
" ~/.vimrc
set number
set expandtab

" drifters::exclude::start
" Machine-specific plugins
call plug#begin('~/.local/share/nvim/plugged')
Plug 'local-only-plugin'
call plug#end()
" drifters::exclude::stop
```

### 2. Shared Aliases, Local Paths

```bash
# ~/.zshrc
alias g="git"
alias d="docker"

# drifters::exclude::start
export PROJECT_PATH="/Users/thomas/dev"
export LOCAL_BIN="/usr/local/custom/bin"
# drifters::exclude::stop
```

### 3. Different Keybindings per Machine

```toml
[apps.zed.machines.laptop]
exclude = ["**/keymap.json"]  # Laptop uses built-in keyboard

[apps.zed.machines.desktop]
# Desktop uses external mechanical keyboard - sync keymaps
```

## Automation

### Optional Auto-Pull Hook

Add to your `.zshrc` or `.bashrc`:

```bash
eval "$(drifters hook)"
```

Runs `drifters pull-app --yolo` in background on shell startup. Changes applied silently.

**Note:** Pushes are always manual for safety.

## App Presets

Check out [presets/](presets/) for pre-configured definitions:

- **[Cursor](presets/cursor.toml)** - AI-powered code editor built on VS Code
- **[Visual Studio Code](presets/vscode.toml)** - Popular code editor with settings, keybindings, and snippets
- **[Windsurf](presets/windsurf.toml)** - Codeium's agentic code editor
- **[Zed Editor](presets/zed.toml)** - Modern code editor
- More coming soon!

### Using Presets

**Recommended method (from GitHub):**

```bash
# List available presets
drifters list-presets

# Load a preset from GitHub
drifters load-preset vscode

# Apply on this machine
drifters merge-app vscode

# On other machines, just pull and merge
drifters pull-app
drifters merge-app vscode
```

**Using local preset files:**

```bash
# Import VS Code preset from local file
drifters import-app vscode --file presets/vscode.toml

# Or let it import from config repo (if already exported there)
drifters import-app vscode
```

**Customize and re-import:**

```bash
# Export current config, edit, re-import
drifters export-app vscode --file ~/vscode-custom.toml
vim ~/vscode-custom.toml
drifters import-app vscode --file ~/vscode-custom.toml
```

See [docs/IMPORT_EXPORT.md](docs/IMPORT_EXPORT.md) for complete guide.

### Contributing Presets

Submit your own app definitions via Pull Request! See [CONTRIBUTING.md](CONTRIBUTING.md).

## Repository Structure

```
your-configs-repo/
├── .drifters/
│   └── sync-rules.toml    # Central configuration (synced)
└── apps/
    └── zed/
        └── machines/
            ├── mac01/
            │   ├── settings.json
            │   └── keymap.json
            └── linux02/
                └── settings.json
```

**No `merged/` directory** - Drifters merges intelligently at pull time from all machine states.

## Comparison with Alternatives

| Feature | Drifters | chezmoi | yadm | Dotbot | Bare Git |
|---------|----------|---------|------|--------|----------|
| Intelligent multi-machine merge | ✅ | ❌ | ❌ | ❌ | ❌ |
| Section-level control | ✅ | ⚠️ (templates) | ❌ | ❌ | ❌ |
| Glob patterns | ✅ | ✅ | ✅ | ⚠️ | ❌ |
| OS-specific rules | ✅ | ✅ | ✅ | ⚠️ | ❌ |
| Per-machine exceptions | ✅ | ⚠️ (templates) | ⚠️ (classes) | ❌ | ❌ |
| Git-based | ✅ | ✅ | ✅ | ✅ | ✅ |
| No templating needed | ✅ | ❌ | ✅ | ✅ | ✅ |
| Consensus merging | ✅ | ❌ | ❌ | ❌ | ❌ |

**Drifters is ideal if you:**
- Work across multiple machines regularly
- Want intelligent merging, not just last-write-wins
- Need to exclude specific sections without complex templating
- Want fine-grained control with simple configuration

## Architecture

### Ephemeral Repository Strategy

On every command:
1. Clone/pull repo to `~/.config/drifters/tmp-repo`
2. Perform operation
3. Commit and push changes
4. Delete temporary repo

**Benefits:**
- No persistent repo taking up space
- Always starts fresh from remote
- No stale state issues

### How Merging Works

When you `pull`:
1. Drifters collects versions from ALL machines in `apps/*/machines/*/`
2. Runs consensus algorithm (majority wins)
3. On ties, prefers current machine's version
4. Applies merged content while preserving local exclude sections

### Supported Comment Styles

Section tags work with any comment syntax:

- Shell/Python/YAML: `# drifters::exclude::start`
- JavaScript/Rust/C++: `// drifters::exclude::start`
- Vim: `" drifters::exclude::start`
- Lua: `-- drifters::exclude::start`
- SQL: `-- drifters::exclude::start`

## Security

### Config Repository

Drifters uses your own private Git repository as storage. **Never commit secrets to it.** Use `drifters::exclude` sections to keep sensitive values (API keys, tokens, passwords) local to each machine.

### Self-Update

`drifters self-update` downloads an installer shell script from GitHub releases over HTTPS and verifies it against a SHA-256 sidecar file before executing. The sidecar is automatically generated by the CI workflow on every release. If the sidecar is missing or the checksum does not match, the update is aborted.

- Only use `drifters self-update` against the official repository (`github.com/tjirsch/drifters`)
- Alternatively, install updates manually via `cargo install drifters` or by downloading a release binary directly
- You can disable automatic update checks entirely: set `self_update_frequency = "never"` in `~/.config/drifters/drifters.toml`
- Use `--skip-checksum` only if you are installing from a release that predates sidecar support

### Shell Hook

The optional shell hook (`eval "$(drifters hook)"`) runs `drifters pull-app --yolo` on every new shell session. This means remote config changes are applied automatically and without confirmation. Only enable it if you fully trust the machines pushing to your config repo.

## FAQ

**Q: Does drifters require a daemon?**
A: No. All operations are manual unless you add the optional shell hook.

**Q: What if two machines have conflicting changes?**
A: Drifters uses consensus-based merging. If 2+ machines agree, that version wins. On ties, your current machine's version is preferred. You'll see a warning in logs.

**Q: Can I sync secrets?**
A: No. Drifters repos are Git repos - never commit secrets. Use `drifters::exclude` sections for sensitive data, or use a proper secret manager.

**Q: Does it work with private repos?**
A: Yes. Drifters uses your system's Git with SSH. Set up SSH keys as normal.

**Q: Can I use it without section tags?**
A: Yes. Section tags are optional. Without them, entire files are synced (with glob-based exclusions).

**Q: What happens if I edit sync-rules.toml on multiple machines?**
A: Run `drifters pull` on each machine to get the latest rules, or use `drifters merge` to re-apply current rules.

**Q: How do I use community app presets?**
A: Use `drifters list-presets` to see available presets, then `drifters load-preset <name>` to import from GitHub. Or use `drifters import-app <name> --file <path>` for local files. See [docs/IMPORT_EXPORT.md](docs/IMPORT_EXPORT.md).

**Q: How do I disable update checks?**
A: Edit `~/.config/drifters/drifters.toml` and set `self_update_frequency = "never"`. Options are: `"never"`, `"daily"`, `"always"` (default).

## Troubleshooting

### Authentication Errors

Ensure SSH works:
```bash
ssh -T git@github.com
```

### Repository Directory Already Exists

Clean up and retry:
```bash
rm -rf ~/.config/drifters/tmp-repo
drifters <command>
```

### Check Configuration

```bash
# View local config
cat ~/.config/drifters/drifters.toml

# View sync rules
drifters list-app

# Verbose output
drifters -v push-app
```

## Development

```bash
# Clone
git clone https://github.com/tjirsch/drifters
cd drifters

# Build
cargo build

# Test
cargo test

# Install locally
cargo install --path .
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas for Contribution

- **App Presets** - Add definitions for popular apps
- **Documentation** - Improve guides and examples
- **Features** - See roadmap below
- **Bug Fixes** - Help squash bugs

## Roadmap

### ✅ Implemented (v0.1 → current)
- Core sync operations (`init`, `add-app`, `push-app`, `pull-app`)
- Intelligent consensus-based merging
- Section tags with exclude-only logic; error on unclosed blocks
- Glob pattern support
- OS-specific rules
- Per-machine overrides
- `merge-app` command for rule re-application
- Ephemeral repository strategy
- Import/export commands for app definitions and rules
- Version history and restore commands
- `diff-app` command for previewing changes
- Self-update with SHA-256 checksum verification
- `rename-machine` / `remove-machine` with cross-machine stale-ID detection
- `rename-app` / extended `remove-app` with `--machine` and `--all` scoping
- Consistent verb-object CLI naming convention
- Shell completion (`bash`, `zsh`, `fish`, `powershell`) via `completion <shell> [--install]`
- `open-readme` command to download and open latest README
- `preferred_editor` config option with `set-preferred-editor` command

### 🔜 Planned
- TUI diff viewer with syntax highlighting (currently text-based)
- Improved conflict resolution UI
- Community preset registry
- Config validation and linting
- Integration tests with real repos

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a full history of changes, including new commands, bug fixes, and the pre-OSS review improvements.

## License

MIT License - see [LICENSE](LICENSE)

## Credits

Built with ❤️ using Rust

- [clap](https://github.com/clap-rs/clap) - CLI parsing
- [git2](https://github.com/rust-lang/git2-rs) - Git operations
- [serde](https://github.com/serde-rs/serde) - Serialization
- [glob](https://github.com/rust-lang/glob) - Pattern matching
- [similar](https://github.com/mitsuhiko/similar) - Diff generation
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client for updates
- [cargo-dist](https://github.com/axodotdev/cargo-dist) - Release distribution

---

**⭐ Star this repo if you find it useful!**

**Questions?** Open an [Issue](https://github.com/tjirsch/drifters/issues)
**Want to contribute?** Check out [CONTRIBUTING.md](CONTRIBUTING.md)
