# Import & Export Commands

Drifters provides commands to easily import and export app definitions and sync rules, eliminating the need to manually clone and edit your config repository.

## Commands Overview

| Command | Purpose |
|---------|---------|
| `drifters import app <name> --file <path>` | Import app definition from file |
| `drifters export app <name> --file <path>` | Export app definition to file |
| `drifters import rules --file <path>` | Import entire sync-rules.toml |
| `drifters export rules --file <path>` | Export entire sync-rules.toml |
| `drifters history rules` | Show history of sync rules |
| `drifters history app <name>` | Show history of specific app |
| `drifters restore app <name> --commit <hash>` | Restore app from previous version |
| `drifters restore rules --commit <hash>` | Restore all rules from previous version |

## Import Commands

### Import App from Preset

```bash
# Import Zed editor preset
drifters import app zed --file presets/zed.toml

# Output:
# ✓ Added 'zed' from "presets/zed.toml"
# Committing changes...
# ✓ Changes committed and pushed
# Run 'drifters merge --app zed' to apply the new rules
```

This automatically:
1. Reads the app definition from the file
2. Adds/updates it in sync-rules.toml
3. Commits and pushes to your repo
4. Prompts you to run merge

### Import Custom App

```bash
# Create your own app definition
cat > ~/my-nvim.toml << 'EOF'
[apps.nvim]
include = [
    "~/.config/nvim/**/*.lua",
    "~/.config/nvim/**/*.vim",
]
exclude = [
    "~/.config/nvim/lazy-lock.json",
]
EOF

# Import it
drifters import app nvim --file ~/my-nvim.toml
```

### Import Entire Rules File

```bash
# Import a complete sync-rules.toml
drifters import rules --file ~/backup/sync-rules.toml

# Output:
# ✓ Imported rules from "~/backup/sync-rules.toml"
#   3 app(s) imported
# Committing changes...
# ✓ Changes committed and pushed
# Run 'drifters merge' to apply the new rules
```

**Warning:** This overwrites your entire sync-rules.toml!

## Export Commands

### Export App for Editing

```bash
# Export current Zed config
drifters export app zed --file ~/zed-config.toml

# Edit it
vim ~/zed-config.toml

# Import it back
drifters import app zed --file ~/zed-config.toml
```

### Export App for Sharing

```bash
# Export your carefully crafted config
drifters export app nvim --file ~/my-nvim-preset.toml

# Share it:
# - Post to GitHub
# - Send to a friend
# - Submit as PR to drifters presets/
```

### Export Entire Config

```bash
# Backup your complete configuration
drifters export rules --file ~/backup/sync-rules-$(date +%Y%m%d).toml

# This is your complete sync config - keep it safe!
```

## History Commands

### View Rules History

```bash
# Show last 10 commits affecting sync-rules.toml
drifters history rules

# Output:
# Sync Rules History
# ============================================================
# abc1234 Add Zed editor config
# def5678 Update nvim patterns
# ghi9012 Restore nvim app from commit xyz
# ...
#
# To see details:
#   drifters history rules --commit <hash>
#
# To restore a version:
#   drifters restore rules --commit <hash>
```

### View App History

```bash
# Show commits affecting a specific app
drifters history app zed

# Show more commits
drifters history app zed --limit 20
```

### View Specific Commit

```bash
# See what changed in a commit
drifters history rules --commit abc1234

# Or for a specific app
drifters history app zed --commit abc1234

# This shows the full git diff
```

## Restore Commands

### Restore App to Previous Version

```bash
# View history first
drifters history app zed

# Restore from a specific commit
drifters restore app zed --commit abc1234

# Output:
# ✓ Restored 'zed' from commit abc1234
# Committing changes...
# ✓ Changes committed and pushed
# Run 'drifters merge --app zed' to apply the restored rules
```

**Note:** This creates a NEW commit with the old content - it doesn't revert Git history.

### Restore Entire Rules

```bash
# View history
drifters history rules

# Restore everything to a previous version
drifters restore rules --commit abc1234

# Output:
# ✓ Restored all rules from commit abc1234
#   3 app(s) restored
# Committing changes...
# ✓ Changes committed and pushed
# Run 'drifters merge' to apply the restored rules
```

## Complete Workflows

### Workflow 1: Using Community Presets

```bash
# 1. Export a preset from drifters repo
drifters export app zed --file ~/zed-preset.toml
# (Or just use the preset file directly)

# 2. Import it
drifters import app zed --file ~/drifters/presets/zed.toml

# 3. Apply it
drifters merge --app zed
```

### Workflow 2: Customize and Share

```bash
# 1. Export your current config
drifters export app nvim --file ~/my-nvim.toml

# 2. Customize it
vim ~/my-nvim.toml
# Add machine overrides, OS variants, etc.

# 3. Test it
drifters import app nvim --file ~/my-nvim.toml
drifters merge --app nvim --dry-run

# 4. Share it
# - Create PR to drifters repo
# - Post on GitHub Gist
# - Share with team
```

### Workflow 3: Rollback After Bad Change

```bash
# Oh no! You broke your config
drifters push zed

# Check history
drifters history app zed

# Restore previous version
drifters restore app zed --commit abc1234

# Apply it
drifters merge --app zed
```

### Workflow 4: Backup and Restore

```bash
# Regular backup
drifters export rules --file ~/backups/drifters-$(date +%Y%m%d).toml

# Later, if needed:
drifters import rules --file ~/backups/drifters-20260215.toml
drifters merge
```

### Workflow 5: Multiple Machine Configs

```bash
# On laptop, export your config
drifters export app zed --file ~/zed-laptop.toml

# Edit for desktop
cp ~/zed-laptop.toml ~/zed-desktop.toml
vim ~/zed-desktop.toml
# Add [apps.zed.machines.desktop] overrides

# On desktop, import
drifters import app zed --file ~/zed-desktop.toml
```

## File Format

App definition files use standard TOML format:

```toml
# my-app.toml
[apps.myapp]
include = [
    "~/.config/myapp/**/*.conf",
]

exclude = [
    "~/.config/myapp/cache/**",
]

include-macos = [
    "~/Library/Application Support/myapp/**/*.conf",
]

[apps.myapp.machines.laptop]
exclude = [
    "**/specific-file.conf",
]
```

Complete rules file:

```toml
# sync-rules.toml
[apps.app1]
include = [...]

[apps.app2]
include = [...]
```

## Tips

### 1. Always Preview Before Importing

```bash
# View the file first
cat preset.toml

# Or use merge --dry-run after import
drifters import app zed --file preset.toml
drifters merge --app zed --dry-run
```

### 2. Keep Backups

```bash
# Before major changes
drifters export rules --file ~/backup.toml
```

### 3. Use History to Track Changes

```bash
# See what changed over time
drifters history rules --limit 50
```

### 4. Test Restores with Dry-Run

```bash
# Restore and preview
drifters restore app zed --commit abc1234
drifters merge --app zed --dry-run
```

### 5. Share Safely

When sharing configs, remove sensitive data:

```toml
# Before sharing:
[apps.myapp]
# ❌ Don't share:
# include = ["~/.ssh/config"]

# ✅ Safe to share:
include = ["~/.config/myapp/settings.json"]
```

## Differences from Manual Editing

### Old Way (Manual)

```bash
git clone git@github.com:user/configs.git ~/my-configs
vim ~/my-configs/.drifters/sync-rules.toml
# Edit manually
git -C ~/my-configs commit -am "Update"
git -C ~/my-configs push
drifters merge
```

### New Way (Import/Export)

```bash
drifters export app zed --file ~/zed.toml
vim ~/zed.toml
drifters import app zed --file ~/zed.toml
drifters merge --app zed
```

**Benefits:**
- ✅ No need to clone repo separately
- ✅ Automatic commit/push
- ✅ Works with ephemeral strategy
- ✅ Simpler workflow
- ✅ Less error-prone

## Troubleshooting

**"App not found in file"**
```bash
# Make sure the TOML has the correct app name
cat myfile.toml
# Should have: [apps.appname]
```

**"Failed to parse TOML"**
```bash
# Validate TOML syntax
python3 -c "import toml; toml.load(open('myfile.toml'))"
```

**"Commit not found"**
```bash
# Use shorter commit hash
drifters restore app zed --commit abc1234  # First 7 chars
```

## Next Steps

- See [CONTRIBUTING.md](../CONTRIBUTING.md) for preset contribution guidelines
- Check [presets/](../presets/) for available presets
- Read [EDITING_SYNC_RULES.md](EDITING_SYNC_RULES.md) for manual editing (still works!)
