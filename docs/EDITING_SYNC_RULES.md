# Editing Sync Rules and Adding App Definitions

This guide explains how to manually edit your sync-rules.toml configuration and integrate app presets.

## Understanding the Configuration Files

### Where Configs Live

**Your Config Repository** (e.g., `git@github.com:username/my-configs.git`):
```
my-configs/
├── .drifters/
│   └── sync-rules.toml    ← Your app definitions (THIS is what you edit)
└── apps/
    └── (synced config files stored here)
```

**Local Drifters Config** (`~/.config/drifters/config.toml`):
```toml
machine_id = "laptop"
repo_url = "git@github.com:username/my-configs.git"
```

This just points to your config repository. You rarely need to edit this.

## Method 1: Direct Edit (Recommended)

Since Drifters uses an ephemeral repository strategy (clones fresh on each command), the best way to edit sync-rules.toml is to **clone your config repo separately**.

### Setup a Persistent Clone

```bash
# Clone your config repo somewhere convenient
cd ~/
git clone git@github.com:username/my-configs.git my-drifters-config

# Edit the sync rules
cd my-drifters-config
vim .drifters/sync-rules.toml
```

### Adding an App Definition

**Option A: Copy from a preset**

```bash
# View the preset
cat ~/projects/drifters/presets/zed.toml

# Append to your sync-rules.toml
cat ~/projects/drifters/presets/zed.toml >> .drifters/sync-rules.toml

# Or manually copy the [apps.zed] section
vim .drifters/sync-rules.toml
# Copy the relevant sections from the preset
```

**Option B: Write from scratch**

```toml
# .drifters/sync-rules.toml

# ... existing apps ...

[apps.zsh]
include = [
    "~/.zshrc",
    "~/.zshenv",
]

exclude = [
    "~/.zsh_history",
]
```

### Commit and Push

```bash
# After editing
git add .drifters/sync-rules.toml
git commit -m "Add zed app configuration"
git push
```

### Sync to All Machines

On each of your other machines:

```bash
# Pull the updated rules
drifters pull

# The sync-rules.toml is automatically synced as part of the repo

# Re-apply the new rules to your configs
drifters merge --dry-run    # Preview what would change
drifters merge              # Apply the new rules
```

## Method 2: Edit During Ephemeral Operations

You can edit sync-rules.toml during a drifters command, but it's trickier because the repo is temporary.

```bash
# Start an operation that clones the repo
drifters push &

# Quickly edit before it completes (not recommended - race condition)
# Better to use Method 1
```

**Not recommended** - use Method 1 instead.

## Method 3: Using `drifters add` (Interactive)

The `drifters add` command provides an interactive way to add apps:

```bash
drifters add zed
```

**Prompts:**
```
Adding app 'zed'
Enter file patterns to include (one per line, empty line to finish):
> ~/.config/zed/settings.json
> ~/.config/zed/**/*.json
>

Enter file patterns to exclude (optional, empty line to skip):
> ~/.config/zed/workspace-*.json
> ~/.config/zed/cache/**
>
```

This automatically updates sync-rules.toml and commits it.

**Limitation:** You can't add OS-specific variants or machine overrides interactively. For those, use Method 1.

## Integrating Community Presets

### From the Drifters Repository

```bash
# 1. Clone your config repo
cd ~/
git clone git@github.com:username/my-configs.git my-drifters-config

# 2. View available presets
ls ~/projects/drifters/presets/

# 3. Copy a preset
cat ~/projects/drifters/presets/zed.toml >> ~/my-drifters-config/.drifters/sync-rules.toml

# 4. Edit if needed
vim ~/my-drifters-config/.drifters/sync-rules.toml

# 5. Commit and push
cd ~/my-drifters-config
git add .drifters/sync-rules.toml
git commit -m "Add Zed editor config from preset"
git push
```

### From a Community Preset Repository

```bash
# Example: community presets (future)
curl https://raw.githubusercontent.com/community/drifters-presets/main/editors/vscode.toml \
  >> ~/my-drifters-config/.drifters/sync-rules.toml

git commit -am "Add VSCode from community preset"
git push
```

## Customizing Presets

After copying a preset, customize it for your needs:

```toml
# Start with preset
[apps.zed]
include = [
    "~/.config/zed/settings.json",
    "~/.config/zed/keymap.json",
]

exclude = [
    "~/.config/zed/workspace-*.json",
]

# Add your machine-specific overrides
[apps.zed.machines.laptop]
exclude = [
    "**/keymap.json"  # Laptop has different keyboard
]

# Add your OS-specific paths
include-linux = [
    "~/.local/share/zed/custom-plugin.conf",
]
```

## Sync-Rules.toml Structure

```toml
# Multiple apps in one file
[apps.zed]
include = [...]
exclude = [...]

[apps.zed.machines.laptop]
exclude = [...]

[apps.zsh]
include = [...]

[apps.git]
include = [...]
include-macos = [...]
include-linux = [...]
```

## Best Practices

### 1. Keep a Persistent Clone

```bash
# Clone once
git clone git@github.com:username/my-configs.git ~/my-drifters-config

# Edit anytime
cd ~/my-drifters-config
vim .drifters/sync-rules.toml
git commit -am "Update config"
git push
```

### 2. Test Changes with Dry-Run

```bash
# After editing sync-rules.toml
drifters pull              # Get the updated rules
drifters merge --dry-run   # See what would change
drifters merge             # Apply if it looks good
```

### 3. Comment Your Config

```toml
[apps.zed]
# Syncing core Zed settings across all machines
include = [
    "~/.config/zed/settings.json",
    "~/.config/zed/keymap.json",
]

# Don't sync workspace state or caches
exclude = [
    "~/.config/zed/workspace-*.json",
    "~/.config/zed/db/**",
]
```

### 4. Organize by Category

```toml
# === Editors ===
[apps.zed]
# ...

[apps.vscode]
# ...

# === Shell ===
[apps.shell]
# ...

# === Development Tools ===
[apps.git]
# ...
```

## Common Workflows

### Adding Multiple Apps at Once

```bash
# Clone config repo
cd ~/my-drifters-config

# Append multiple presets
cat ~/projects/drifters/presets/zed.toml >> .drifters/sync-rules.toml

# Or manually add zsh config
cat >> .drifters/sync-rules.toml << 'EOF'
[apps.zsh]
include = ["~/.zshrc", "~/.zshenv"]
EOF

# Commit
git add .drifters/sync-rules.toml
git commit -m "Add Zed and Zsh configs"
git push
```

### Removing an App

```bash
# Edit sync-rules.toml
vim ~/my-drifters-config/.drifters/sync-rules.toml

# Delete the entire [apps.appname] section
# (Including all subsections like [apps.appname.machines.x])

# Commit
git commit -am "Remove appname from sync"
git push
```

### Changing an App's Config

```bash
# Edit the app's section
vim ~/my-drifters-config/.drifters/sync-rules.toml

# Example: add a new file pattern
[apps.zed]
include = [
    "~/.config/zed/settings.json",
    "~/.config/zed/keymap.json",
    "~/.config/zed/tasks.json",  # ← Added this
]

# Commit and push
git commit -am "Add tasks.json to Zed sync"
git push

# On all machines, re-merge with new rules
drifters merge --app zed
```

## Troubleshooting

### "App not found" after editing

```bash
# Make sure you pushed the changes
cd ~/my-drifters-config
git push

# Pull on the machine where you're getting the error
drifters pull

# Or just run any drifters command (it auto-pulls the repo)
drifters list
```

### Syntax errors in TOML

```bash
# Validate TOML syntax
cat .drifters/sync-rules.toml | python3 -c "import sys, toml; toml.load(sys.stdin)"

# Or use an online TOML validator
```

### Changes not taking effect

```bash
# Pull latest rules
drifters pull

# Re-apply rules to configs
drifters merge --dry-run
drifters merge
```

## Future: Import Command (Planned)

We're planning an `import` command to make this easier:

```bash
# Future command
drifters import preset zed
drifters import url https://example.com/app.toml
drifters import file ~/Downloads/zsh.toml
```

For now, use manual copy/paste as described above.

## Summary

**Quick Reference:**

1. **Clone your config repo persistently:**
   ```bash
   git clone git@github.com:username/my-configs.git ~/my-drifters-config
   ```

2. **Edit sync-rules.toml:**
   ```bash
   vim ~/my-drifters-config/.drifters/sync-rules.toml
   ```

3. **Commit and push:**
   ```bash
   cd ~/my-drifters-config
   git commit -am "Update config"
   git push
   ```

4. **Apply on all machines:**
   ```bash
   drifters merge --dry-run
   drifters merge
   ```

That's it! You now have full control over your sync configuration.
