# Testing Drifters

## Prerequisites

1. **GitHub repository set up**: You need a GitHub repository for storing configs
   - Example: https://github.com/tjirsch/dr-configs
   - Can be empty initially
   - Recommend making it private for sensitive configs

2. **Git authentication configured**:
   - SSH key added to GitHub, OR
   - Personal access token configured

## Testing with Your Repository

### Initial Setup

```bash
# Build the project
cd /Users/thomas/projects/drifters
cargo build --release

# Add to PATH for convenience (optional)
export PATH="$PWD/target/release:$PATH"

# Initialize drifters with your test repo
drifters init https://github.com/tjirsch/dr-configs
# OR with SSH:
drifters init git@github.com:tjirsch/dr-configs.git
```

The init process will:
1. Detect your machine ID (hostname)
2. Clone the repo to a **temporary location**: `~/.config/drifters/tmp-repo`
3. Register your machine in the repo
4. Push changes to remote
5. **Delete the temporary repo**
6. Create local config at: `~/.config/drifters/config.toml`
7. Optionally set up a shell hook

### Repository Isolation Verification

```bash
# Check the local config
cat ~/.config/drifters/config.toml

# It should show:
# machine_id = "tm1"
# repo_url = "https://github.com/tjirsch/dr-configs"

# Check that NO persistent repo exists
ls ~/.config/drifters/
# Should show ONLY: config.toml
# (tmp-repo only exists during command execution)
```

**Important**: The repo is **ephemeral**:
- Cloned fresh on every `push`, `pull`, or `add` command
- Deleted immediately after the command completes
- Zero persistence = zero interference with your working environment

### Test Scenario 1: Simple Config Sync

```bash
# Add a simple app (git config)
drifters add git
# When prompted:
# > ~/.gitconfig
# > (press Enter to finish)
# Choice: 1 (Full sync)

# Push your gitconfig
drifters push git

# Check the repo structure
tree ~/Library/Application\ Support/drifters/repo/apps/git
# Should show:
# apps/git/
# ├── machines/
# │   └── tm1/
# │       └── gitconfig
# └── merged/
#     └── gitconfig
```

### Test Scenario 2: Multi-Machine Sync

```bash
# On machine 1: push some configs
drifters push

# On machine 2: initialize and pull
drifters init git@github.com:tjirsch/dr-configs.git
drifters pull

# Configs from machine 1 are now on machine 2
```

### Test Scenario 3: Marker-Based Sync (Shell Config)

Create a test shell config:

```bash
# Create a test .testrc file
cat > ~/.testrc << 'EOF'
# Local settings
export LOCAL_VAR="machine-specific"

#-start-sync-
export EDITOR="nvim"
export LANG="en_US.UTF-8"
alias g="git"
alias d="docker"
#-stop-sync-

# More local stuff
alias vpn="connect-local"
EOF

# Add it with marker mode
drifters add testshell
# > ~/.testrc
# > (Enter)
# Choice: 2 (Markers)

drifters push testshell
```

Expected behavior:
- Only content between `#-start-sync-` and `#-stop-sync-` gets synced
- Local settings and VPN alias remain machine-specific

### Test Scenario 4: Per-Machine Exceptions

Edit `.drifters/sync-rules.toml` in the repo:

```toml
[apps.zed]
files = [
    "~/.config/zed/settings.json",
    "~/.config/zed/keymap.json",
]
sync_mode = "full"

# Don't sync settings.json to tm1
[apps.zed.exceptions]
tm1 = ["settings.json"]
```

Now on machine "tm1", `settings.json` won't be synced, but `keymap.json` will.

### Test Scenario 5: Safety Features

Test the empty file warning:

```bash
# Create an empty test file
touch ~/test-empty.txt

# Add it to an app
drifters add testapp
# > ~/test-empty.txt

# Try to push (should warn)
drifters push testapp
# Expected: Warning about empty file
```

### Cleanup

To completely remove drifters and start fresh:

```bash
# Remove local config (repo is already ephemeral, no cleanup needed)
rm -rf ~/.config/drifters/

# Optionally: remove test configs from GitHub
# (delete the repository or reset it)
```

**Note**: Since the repo is ephemeral, there's no persistent repo directory to clean up!

## Troubleshooting

### Authentication Errors

If you see `remote authentication required but no callback set`:

**Option 1: Use SSH**
```bash
drifters init git@github.com:tjirsch/dr-configs.git
```

**Option 2: Use HTTPS with credential helper**
```bash
git config --global credential.helper osxkeychain  # macOS
git config --global credential.helper store        # Linux
```

### Repository Already Exists

If you see "Repository directory already exists":

```bash
# Clean up and retry
rm -rf ~/Library/Application\ Support/drifters
drifters init <repo-url>
```

### Already Initialized

If you see "Drifters already initialized":

```bash
# Check current config
cat ~/Library/Application\ Support/drifters/config.toml

# To reinitialize: remove config and try again
rm ~/.config/drifters/config.toml
```

## What to Test

- [ ] Init on first machine
- [ ] Add an app
- [ ] Push configs
- [ ] Init on second machine (or VM)
- [ ] Pull configs on second machine
- [ ] Modify config on second machine
- [ ] Push from second machine
- [ ] Pull updates on first machine
- [ ] Test with `--yolo` flag
- [ ] Test safety warnings (empty files)
- [ ] Test per-machine exceptions
- [ ] Test marker-based sync

## Reporting Issues

If you encounter any issues:

1. Check the logs (run with `-v` flag for verbose output)
2. Verify the repo structure in `~/Library/Application Support/drifters/repo`
3. Check GitHub to see what was committed
4. Report issues at: https://github.com/tjirsch/drifters/issues
