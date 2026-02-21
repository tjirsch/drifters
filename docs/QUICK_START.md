# Quick Start Guide

Get Drifters running in 5 minutes.

## Prerequisites

- Git installed
- SSH key configured for GitHub/GitLab
- Rust (for building from source)

## Step 1: Install Drifters

```bash
# Clone and build
git clone https://github.com/tjirsch/drifters
cd drifters
cargo install --path .

# Verify installation
drifters --version
```

## Step 2: Create a Config Repository

On GitHub/GitLab, create a **private** repository:
- Name: `my-configs` (or whatever you prefer)
- Private: ✅ (recommended)
- Initialize: Empty (no README)

```bash
# Example URL
git@github.com:username/my-configs.git
```

## Step 3: Initialize on First Machine

```bash
drifters init git@github.com:username/my-configs.git
```

**Output:**
```
Cloning repository...
✓ Repository cloned
Detected machine: macbook-pro (macos)
✓ Local config saved to ~/.config/drifters/config.toml
✓ Registered machine 'macbook-pro'
```

Your machine ID (e.g., `macbook-pro`) will be used to identify this machine.

## Step 4: Add Your First App

Let's sync Zed editor configs:

### Option A: Interactive

```bash
drifters add zed
```

**Prompts:**
```
Adding app 'zed'
Enter file patterns to include (one per line, empty line to finish):
> ~/.config/zed/settings.json
> ~/.config/zed/keymap.json
>

Enter file patterns to exclude (optional, empty line to skip):
> ~/.config/zed/workspace-*.json
>

✓ Added 'zed' to sync rules
✓ Changes committed and pushed
```

### Option B: Use Preset

```bash
# View the preset
cat presets/zed.toml

# Then add it manually:
cd ~/.config/drifters/tmp-repo
# (repo is cloned temporarily during commands)
# Or clone your repo separately and edit .drifters/sync-rules.toml
```

## Step 5: Push Your Configs

```bash
drifters push
```

**Output:**
```
Setting up repository...
Pushing configs for 'zed'...
  ✓ settings.json
  ✓ keymap.json

Pushed 2 file(s) for 1 app(s)
Commit and push these changes? [Y/n] y

Committing changes...
✓ Successfully pushed 2 file(s)
```

Your configs are now in the repo!

## Step 6: Pull on Another Machine

On your second machine:

```bash
# Initialize
drifters init git@github.com:username/my-configs.git

# Detected machine: linux-desktop (linux)
# ✓ Repository cloned
# Found existing apps in repo: zed
# ✓ Local config saved

# Pull the configs
drifters pull

# Pulling configs for 'zed'...
#   settings.json - ✓ Created
#   keymap.json - ✓ Created
# ✓ Successfully pulled 2 file(s)
```

Done! Your configs are synced.

## Step 7: Make a Change

On your second machine, edit the config:

```bash
# Edit settings
vim ~/.config/zed/settings.json

# Push the change
drifters push zed

# On first machine, pull it
drifters pull zed
```

## Using Section Tags

For machine-specific settings within files:

```json
// ~/.config/zed/settings.json
{
  "theme": "One Dark",
  "vim_mode": true,

  // drifters::exclude::start
  "working_directory": "/home/user/projects",
  "local_setting": true
  // drifters::exclude::stop
}
```

Now when you push/pull:
- `theme` and `vim_mode` sync across machines
- `working_directory` and `local_setting` stay machine-specific

## Optional: Auto-Pull on Shell Startup

Add to `~/.zshrc` or `~/.bashrc`:

```bash
eval "$(drifters hook)"
```

Now your configs auto-pull silently when you open a terminal.

## Common Workflows

### Add Another App

```bash
drifters add zsh
# > ~/.zshrc
# > ~/.zshenv
# >

drifters push zsh
```

### Exclude a File on One Machine

```bash
# On laptop: don't sync keymap (different keyboard)
drifters exclude zed keymap.json

# Check it worked
drifters list
```

### Check Status

```bash
drifters status

# Shows:
# - Which files are up to date
# - Which have local changes
# - Which are available from other machines
```

### Re-apply Rules After Changes

```bash
# Edit sync-rules.toml with new patterns
# Then re-merge all configs with current rules
drifters merge --dry-run    # Preview
drifters merge              # Apply
```

## Next Steps

- **Read [README.md](../README.md)** for full feature list
- **Browse [presets/](../presets/)** for more app configs
- **Check [examples/](../examples/)** for complex configurations
- **See [CONTRIBUTING.md](../CONTRIBUTING.md)** to add your own presets

## Troubleshooting

**"Repository directory already exists"**
```bash
rm -rf ~/.config/drifters/tmp-repo
drifters <command>
```

**"Authentication failed"**
```bash
# Test SSH
ssh -T git@github.com

# Ensure your SSH key is added
ssh-add -l
```

**"No changes to push"**
- Make sure files exist locally
- Check file paths are correct
- Use `drifters status` to see what's tracked

**"App not found"**
```bash
# List apps
drifters list

# Add the app first
drifters add <app>
```

## Tips

1. **Start small** - Sync one app first, then add more
2. **Use presets** - Check `presets/` for popular apps
3. **Test with --dry-run** - Preview changes before applying
4. **Use section tags** - For API keys, local paths, etc.
5. **Check status regularly** - `drifters status` shows sync state

Happy syncing! 🚀
