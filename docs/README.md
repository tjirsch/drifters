# Drifters Documentation

Complete documentation for Drifters config synchronization.

## Getting Started

- **[Quick Start Guide](QUICK_START.md)** - Get running in 5 minutes
- **[Main README](../README.md)** - Feature overview and comparison

## Configuration

- **[Editing Sync Rules](EDITING_SYNC_RULES.md)** - How to manually edit sync-rules.toml and add app definitions
- **[Complete Example](../examples/sync-rules-complete.toml)** - Comprehensive sync-rules.toml with all features
- **[App Presets](../presets/)** - Pre-configured definitions for popular apps

## Core Concepts

### Sync Rules Hierarchy

Rules are applied in order (highest priority first):

1. **Machine-specific** (`[apps.app.machines.laptop]`)
2. **OS-specific** (`include-macos`, `include-linux`)
3. **App defaults** (`[apps.app]`)

### Section Tags

Exclude machine-specific content within files:

```bash
# ~/.zshrc
export SHARED_VAR="value"

# drifters::exclude::start
export LOCAL_PATH="/home/user/specific/path"
# drifters::exclude::stop
```

### Intelligent Merging

When pulling, Drifters:
1. Collects versions from ALL machines
2. Finds consensus (majority wins)
3. Prefers current machine on ties
4. Preserves local exclude sections

## Workflows

### Adding Apps

**Interactive:**
```bash
drifters add zed
# Follow prompts
```

**From preset:**
```bash
git clone git@github.com:username/my-configs.git ~/my-drifters-config
cat presets/zed.toml >> ~/my-drifters-config/.drifters/sync-rules.toml
cd ~/my-drifters-config && git commit -am "Add Zed" && git push
drifters merge --app zed
```

**Manual:**
```bash
# Edit sync-rules.toml directly
vim ~/my-drifters-config/.drifters/sync-rules.toml
```

See: [Editing Sync Rules](EDITING_SYNC_RULES.md)

### Syncing Across Machines

**Push local changes:**
```bash
drifters push [app]    # Interactive confirmation
drifters push --yolo   # Skip confirmation
```

**Pull remote changes:**
```bash
drifters pull [app]    # Interactive confirmation
drifters pull --yolo   # Skip confirmation
```

**Check status:**
```bash
drifters status        # See what's synced/changed
drifters list          # List all apps
```

### Re-applying Rules

After editing sync-rules.toml:

```bash
drifters merge --dry-run              # Preview changes
drifters merge                        # Apply to all apps
drifters merge --app zed              # Apply to specific app
drifters merge --machine mac01        # Test merging from one machine
drifters merge --os macos --dry-run   # Test OS-specific rules
```

## Common Patterns

### Multi-Platform Config

```toml
[apps.myapp]
include = ["~/.config/myapp/config.json"]

include-macos = ["~/Library/Application Support/myapp/config.json"]
include-linux = ["~/.config/myapp/config.json"]
include-windows = ["~/AppData/Roaming/myapp/config.json"]
```

### Per-Machine Exceptions

```toml
[apps.myapp]
include = ["~/.config/myapp/**/*.json"]

[apps.myapp.machines.laptop]
exclude = ["**/keybindings.json"]  # Different keyboard
```

### Glob Patterns

```toml
include = [
    "~/.config/app/**/*.json",     # All JSON files recursively
    "~/.config/app/*.conf",         # Conf files in root only
]

exclude = [
    "~/.config/app/cache/**",       # Entire directory
    "**/temp-*.json",               # Pattern anywhere
]
```

### Section Tag Usage

In config files:

```json
// JSON: //
// drifters::exclude::start
{ "local": "value" }
// drifters::exclude::stop
```

```yaml
# YAML: #
# drifters::exclude::start
local_setting: value
# drifters::exclude::stop
```

```lua
-- Lua: --
-- drifters::exclude::start
local config = {}
-- drifters::exclude::stop
```

```vim
" Vim: "
" drifters::exclude::start
set local_option
" drifters::exclude::stop
```

## Architecture

### Repository Structure

```
your-configs-repo/
├── .drifters/
│   └── sync-rules.toml    # All app definitions
└── apps/
    └── zed/
        └── machines/
            ├── mac01/     # Machine 1's config state
            └── linux02/   # Machine 2's config state
```

No `merged/` directory - intelligent merging happens at pull time.

### Ephemeral Strategy

Every command:
1. Clones repo to temp location
2. Performs operation
3. Commits and pushes
4. Deletes temp repo

Benefits: Always fresh, no stale state.

### Local Storage

```
~/.config/drifters/
├── config.toml           # Machine ID and repo URL
└── tmp-repo/            # Temporary (deleted after each command)
```

## Advanced Topics

### Custom Comment Syntax

Drifters auto-detects comment syntax by file extension. To override:

```toml
[apps.myapp.sections]
"custom.conf" = true   # Force section processing
"data.json" = false    # Force full-file sync (ignore tags)
```

### Multiple Exclude Sections

You can have multiple exclude sections per file:

```bash
export SHARED1="value"

# drifters::exclude::start
export LOCAL1="path"
# drifters::exclude::stop

export SHARED2="value"

# drifters::exclude::start
export LOCAL2="path"
# drifters::exclude::stop
```

### Debugging

```bash
# Verbose output
drifters -v push

# Check what's tracked
drifters list

# See status
drifters status

# Preview merge
drifters merge --dry-run
```

## Troubleshooting

See [Quick Start - Troubleshooting](QUICK_START.md#troubleshooting)

Common issues:
- **Authentication errors** - Check SSH setup
- **Repo already exists** - Clear temp: `rm -rf ~/.config/drifters/tmp-repo`
- **Changes not syncing** - Run `drifters merge` to re-apply rules
- **Syntax errors** - Validate TOML syntax

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Adding app presets
- Submitting bug reports
- Contributing code

## Examples

- [Complete sync-rules.toml](../examples/sync-rules-complete.toml)
- [Zed preset](../presets/zed.toml)

## Future Plans

Planned features:
- `drifters import preset <name>` - Import presets automatically
- Community preset registry
- TUI diff viewer
- Config validation
- Shell completion

## Getting Help

- **Issues:** [GitHub Issues](https://github.com/tjirsch/drifters/issues)
- **Questions:** [GitHub Discussions](https://github.com/tjirsch/drifters/discussions)
- **Contributing:** [CONTRIBUTING.md](../CONTRIBUTING.md)
