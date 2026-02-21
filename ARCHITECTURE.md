# Drifters Architecture

## Repository Storage & Isolation

### Ephemeral Repository Strategy

Drifters uses an **ephemeral repository** approach for maximum isolation and safety:

1. **On every command** (push, pull, add):
   - Clone/pull the repo to a temporary location: `~/.config/drifters/tmp-repo`
   - Perform the requested operation
   - Commit and push changes
   - **Delete the temporary repo**

2. **Benefits**:
   - No persistent repo taking up space
   - Zero chance of interfering with working directories
   - Always starts fresh from remote
   - No stale state issues

### Storage Locations

- **Config**: `~/.config/drifters/config.toml` (stores machine ID and repo URL)
- **Temporary repo**: `~/.config/drifters/tmp-repo` (cloned on each command, deleted after)

All platforms (macOS, Linux, Windows) use `~/.config/drifters/` for consistency.

### Safety Guarantees

1. **Complete isolation**: Temp repo exists only during command execution
2. **No modification of source files**: Files are copied, not moved
3. **Git history preserved**: All changes are committed to remote, allowing rollback
4. **Confirmation prompts**: By default, all destructive operations require confirmation
5. **RAII cleanup**: Even if command fails or panics, temp repo is cleaned up

## File Selection Mechanisms

Different file types require different approaches for partial file synchronization.

### 1. Full File Sync (Default)

Syncs the entire file as-is. Simple and reliable.

```toml
[apps.git]
files = ["~/.gitconfig"]
sync_mode = "full"
```

### 2. Comment Markers

For files that support comments (shell scripts, Python, Ruby, YAML, etc.):

```bash
# .zshrc example
export LOCAL_PATH="/local/bin"

#-start-sync-
export EDITOR="nvim"
export LANG="en_US.UTF-8"
alias g="git"
#-stop-sync-

# Machine-specific aliases
alias vpn="connect-work"
```

```toml
[apps.zsh]
files = ["~/.zshrc"]
sync_mode = "markers"
```

### 3. JSONPath Selection *(Planned)*

For JSON and structured data files:

```json
{
  "local_theme": "dark",
  "fontSize": 14,
  "keybindings": [
    {"key": "cmd+k", "command": "search"}
  ]
}
```

```toml
[apps.vscode]
files = ["~/.config/Code/settings.json"]
sync_mode = "jsonpath"

[apps.vscode.selectors]
"settings.json" = ".fontSize, .keybindings"  # Only sync these paths
```

**JSONPath Examples**:
- `.theme` - Select theme key
- `.keybindings[*]` - Select all keybindings
- `.editor.*` - Select all editor settings
- `.fonts.size, .fonts.family` - Select multiple specific keys

### 4. Line Ranges *(Planned)*

For files where specific line ranges are stable:

```toml
[apps.custom]
files = ["~/.config/app/config"]
sync_mode = "lines"

[apps.custom.selectors]
"config" = "10-50,100-150"  # Sync lines 10-50 and 100-150
```

### 5. Regex Patterns *(Planned)*

For complex selection based on content matching:

```toml
[apps.custom]
files = ["~/.bashrc"]
sync_mode = "regex"

[apps.custom.selectors]
"config" = "^export.*|^alias.*"  # Only sync export and alias lines
```

## Selection Strategy by File Type

| File Type | Recommended Strategy | Alternative |
|-----------|---------------------|-------------|
| JSON | JSONPath | Full file |
| YAML | Markers or JSONPath | Full file |
| TOML | Markers (# comments) | Full file |
| Shell scripts | Markers (# comments) | Regex |
| Python | Markers (# comments) | Regex |
| Vim/Neovim | Markers (" comments) | Full file |
| Plain text | Full file | Line ranges |

## Merge Strategy (MVP)

Current implementation (MVP):
- Files are copied to `apps/[app]/machines/[machine-id]/`
- Merged state is updated in `apps/[app]/merged/`
- Simple last-write-wins for MVP

Future (intelligent merging):
- Three-way merge with conflict detection
- Per-section merging based on selectors
- Conflict resolution UI

## Repository Structure

```
dr-configs/                          # Your GitHub repo
├── .drifters/
│   ├── sync-rules.toml              # Global sync configuration
│   └── machines.toml                # Registered machines
├── apps/
│   ├── zed/
│   │   ├── machines/
│   │   │   ├── tm1/                 # Machine-specific configs
│   │   │   │   ├── settings.json
│   │   │   │   └── keymap.json
│   │   │   └── desktop01/
│   │   │       └── settings.json
│   │   └── merged/                  # Canonical merged state
│   │       ├── settings.json
│   │       └── keymap.json
│   └── nvim/
│       ├── machines/
│       └── merged/
```

## Workflow

### Push Workflow
1. User modifies local config file
2. `drifters push`
3. Pull latest changes from remote (avoid conflicts)
4. Safety check (warn if file is suspiciously small)
5. Copy file to `machines/[machine-id]/`
6. Update `merged/` state (MVP: simple copy, future: intelligent merge)
7. Commit and push to GitHub

### Pull Workflow
1. `drifters pull`
2. Pull latest changes from remote
3. For each app:
   - Check exceptions (skip files marked for this machine)
   - Compare local vs merged state
   - Prompt for confirmation if different
   - Apply merged state to local file

## Safety Features

1. **Empty file detection**: Warns if pushing a file <10 bytes when repo has larger version
2. **Size ratio warning**: Warns if file is 10x smaller than repo version
3. **Interactive confirmation**: Shows diffs and asks before applying changes
4. **Git history**: All changes are committed, enabling rollback
5. **`--yolo` flag**: Available for batch operations (use with caution)

## Future Enhancements

### Intelligent Merging
- Detect conflicts when same section modified on multiple machines
- Interactive conflict resolution UI
- Section-level merging based on selectors

### Advanced Selection
- Code execution for dynamic selection (custom scripts)
- Compound selectors (combine JSONPath + regex)
- Template variables in configs

### TUI Improvements
- Rich diff viewer with syntax highlighting
- Side-by-side comparison
- Interactive file selection

### OS-Specific Handling
```toml
[apps.zed.files]
macos = ["~/Library/Application Support/Zed/settings.json"]
linux = ["~/.config/zed/settings.json"]
windows = ["%APPDATA%\\Zed\\settings.json"]
```
