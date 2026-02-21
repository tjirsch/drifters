# Drifters

**Config file synchronization across machines using Git as a backend.**

Drifters helps you keep configuration files synchronized across multiple machines (macOS, Linux, Windows) with intelligent merging, per-machine exceptions, and version control.

## Features

- 🔄 **Sync config files** across multiple machines
- 📦 **Git backend** for version control and history
- 🎯 **Per-machine exceptions** for files that should differ
- 🔒 **Safety checks** to prevent accidental overwrites
- ⚡ **Ephemeral repository** (cloned fresh on each command, zero persistence)
- 🚀 **Auto-sync on shell startup** (optional)
- ✅ **Interactive confirmations** (or use `--yolo` to skip)
- 🎨 **Marker-based syncing** to sync only specific sections of files
- 📋 **List and manage** apps and exclusions

## Installation

### From Source

```bash
git clone https://github.com/tjirsch/drifters
cd drifters
cargo build --release
# Binary will be at target/release/drifters

# Optional: Add to PATH
sudo cp target/release/drifters /usr/local/bin/
```

### From crates.io (coming soon)

```bash
cargo install drifters
```

## Quick Start

### Prerequisites

- Git repository for storing configs (e.g., GitHub, GitLab)
- SSH keys configured for your Git hosting service
- Git installed on your system

### 1. Initialize on your first machine

```bash
# Create a GitHub repository first (e.g., my-configs)
drifters init git@github.com:username/my-configs.git

# Detected machine: macbook-pro (macos). Use this? [Y/n] y
# ✓ Repository cloned successfully
# ✓ Local config saved
# ✓ Registered machine 'macbook-pro' (macos)
```

### 2. Add an app to sync

```bash
drifters add zed

# Adding app 'zed'
# Enter config file paths to sync (one per line, empty line to finish):
# > ~/.config/zed/settings.json
# > ~/.config/zed/keymap.json
# >
# Sync mode:
#   1. Full - sync entire files (default)
#   2. Markers - sync only content between markers
# Choice [1]: 1
# ✓ Added 'zed' to sync rules
# ✓ Changes committed and pushed
```

### 3. Push your configs

```bash
drifters push

# Pulling latest changes...
# Pushing configs for 'zed'...
#   ✓ settings.json
#   ✓ keymap.json
# Commit and push these changes? [Y/n] y
# ✓ Successfully pushed 2 file(s)
```

### 4. On another machine, pull the configs

```bash
# First, initialize on the new machine
drifters init git@github.com:username/my-configs.git

# Then pull the configs
drifters pull

# Pulling latest changes from repository...
# Pulling configs for 'zed'...
#   ✓ settings.json
#   ✓ keymap.json
# ✓ Successfully pulled 2 file(s)
```

## Commands

### Core Commands

```bash
# Initialize drifters on a new machine
drifters init <repo-url>

# Add an app to sync
drifters add <app>

# Push local configs to repository
drifters push [app]

# Pull remote configs to local machine
drifters pull [app]

# List all apps and their configuration
drifters list

# Exclude a file from syncing on this machine
drifters exclude <app> <filename>

# Show sync status (coming soon)
drifters status

# Show what would change without applying (coming soon)
drifters diff [app]

# Generate shell hook for auto-sync
drifters hook
```

### Flags

```bash
--yolo        # Skip all confirmations (use with caution!)
-v, --verbose # Show detailed logging
```

## Per-Machine Exceptions

Keep specific files different on each machine:

```bash
# On your laptop: Keep zed settings.json local
drifters exclude zed settings.json

# Now when you push/pull:
# - keymap.json syncs normally
# - settings.json stays machine-specific
```

### How it works

Exclusions are stored in `.drifters/sync-rules.toml` in your repository:

```toml
[apps.zed]
files = ["~/.config/zed/settings.json", "~/.config/zed/keymap.json"]
sync_mode = "full"

[apps.zed.exceptions]
macbook-pro = ["settings.json"]  # Don't sync settings.json to macbook-pro
```

This syncs across all machines, so other machines know not to overwrite excluded files.

## Marker-Based Syncing

Sync only specific sections of files using comment markers:

### Example: Shell Config

```bash
# ~/.zshrc
export LOCAL_PATH="/usr/local/bin"
alias vpn="connect-work"

#-start-sync-
export EDITOR="nvim"
export LANG="en_US.UTF-8"
alias g="git"
alias d="docker"
#-stop-sync-

# Machine-specific aliases
alias backup="rsync -av /data /backup"
```

### Setup

```bash
drifters add shell
# > ~/.zshrc
# >
# Sync mode:
#   1. Full - sync entire files (default)
#   2. Markers - sync only content between markers
# Choice [1]: 2
```

Only content between `#-start-sync-` and `#-stop-sync-` will be synced. Local settings and machine-specific aliases remain untouched.

### Supported Comment Styles

Markers work with any comment syntax:

- Shell/Python/Ruby/YAML: `#-start-sync-` / `#-stop-sync-`
- JavaScript/C/C++/Rust: `//-start-sync-` / `//-stop-sync-`
- Vim: `"-start-sync-` / `"-stop-sync-`
- Lua: `---start-sync-` / `---stop-sync-`

## Auto-Sync on Shell Startup

Add this to your `.zshrc` or `.bashrc`:

```bash
eval "$(drifters hook)"
```

This will automatically pull latest configs when you open a new terminal (runs in background, non-blocking).

## Repository Structure

```
my-configs/                        # Your Git repository
├── .drifters/
│   ├── sync-rules.toml            # App configurations
│   └── machines.toml               # Machine registry
├── apps/
│   ├── zed/
│   │   ├── machines/
│   │   │   ├── macbook-pro/
│   │   │   │   ├── settings.json
│   │   │   │   └── keymap.json
│   │   │   └── linux-desktop/
│   │   │       └── settings.json
│   │   └── merged/
│   │       ├── settings.json      # Canonical merged state
│   │       └── keymap.json
│   └── nvim/
│       ├── machines/
│       │   └── macbook-pro/
│       │       └── init.vim
│       └── merged/
│           └── init.vim
```

## How It Works

### Ephemeral Repository Strategy

Drifters uses an **ephemeral repository** approach:

1. On every command (`push`, `pull`, `add`, etc.):
   - Clone/pull the repo to `~/.config/drifters/tmp-repo`
   - Perform the requested operation
   - Commit and push changes
   - **Delete the temporary repo**

2. Benefits:
   - No persistent repo taking up space
   - Zero chance of interfering with working directories
   - Always starts fresh from remote
   - No stale state issues

### Storage Locations

- **Config**: `~/.config/drifters/config.toml` (stores machine ID and repo URL)
- **Temporary repo**: `~/.config/drifters/tmp-repo` (cloned on each command, deleted after)

### Sync Workflow

**Push:**
1. User modifies local config file
2. `drifters push` pulls latest changes from remote
3. Safety check (warn if file is suspiciously small)
4. Copy file to `apps/[app]/machines/[machine-id]/`
5. Update `apps/[app]/merged/` state
6. Commit and push to Git

**Pull:**
1. `drifters pull` clones fresh repo
2. For each app:
   - Check exceptions (skip files marked for this machine)
   - Compare local vs merged state
   - Prompt for confirmation if different
   - Apply merged state to local file

## Advanced Configuration

### Sync Rules Format

`.drifters/sync-rules.toml`:

```toml
[apps.myapp]
files = ["~/.config/myapp/config.json"]
sync_mode = "full"  # or "markers"

# Exclude files on specific machines
[apps.myapp.exceptions]
machine1 = ["config.json"]
machine2 = ["theme.json"]

# Selectors for advanced modes (future)
[apps.myapp.selectors]
# JSONPath, regex, line ranges (coming soon)
```

### Machine Registry

`.drifters/machines.toml`:

```toml
[machines.macbook-pro]
os = "macos"
last_sync = "2025-02-21T01:23:45Z"

[machines.linux-desktop]
os = "linux"
last_sync = "2025-02-20T15:30:00Z"
```

## Examples

### Sync Entire Gitconfig

```bash
drifters add git
# > ~/.gitconfig
# >
# Choice [1]: 1

drifters push git
```

### Sync Shell Config with Markers

```bash
# Add markers to ~/.zshrc
cat >> ~/.zshrc << 'EOF'

#-start-sync-
export EDITOR="nvim"
alias g="git"
#-stop-sync-
EOF

drifters add shell
# > ~/.zshrc
# >
# Choice [1]: 2

drifters push shell
```

### Exclude File on Laptop

```bash
# Keep VSCode settings local on laptop
drifters exclude vscode settings.json

# Verify exclusion
drifters list
```

## Troubleshooting

### Repository Directory Already Exists

If you see "Repository directory already exists":

```bash
# Clean up and retry
rm -rf ~/.config/drifters/tmp-repo
drifters <command>
```

(This shouldn't happen due to ephemeral cleanup, but if it does...)

### Authentication Errors

Drifters uses system `git` commands, so ensure:

```bash
# SSH works
ssh -T git@github.com

# OR use HTTPS with credential helper
git config --global credential.helper osxkeychain  # macOS
git config --global credential.helper store        # Linux
```

### Checking Configuration

```bash
# View local config
cat ~/.config/drifters/config.toml

# View sync rules
drifters list

# Verbose output
drifters -v push
```

## Roadmap

### MVP (Current) ✅
- Init, add, push, pull commands
- Git integration with ephemeral repos
- Safety checks
- Per-machine configs and exceptions
- Marker-based syncing
- List and exclude commands

### Planned Features
- 🎨 TUI diff viewer with syntax highlighting
- 🔀 Intelligent three-way merge with conflict resolution
- 📊 Status command showing what needs syncing
- 🔍 JSONPath selection for JSON/YAML files
- 🎯 Regex pattern matching for selective sync
- 📏 Line range selection
- 🖥️ OS-specific file paths
- 🔄 Bidirectional sync detection

## Contributing

Issues and PRs welcome at https://github.com/tjirsch/drifters

## License

MIT

## Credits

Built with Rust 🦀
