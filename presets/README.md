# Drifters App Presets

This directory contains pre-configured app definitions for popular applications. These presets make it easy to start syncing your favorite apps without manual configuration.

## Available Presets

- **[cursor.toml](cursor.toml)** - Cursor IDE (AI-powered code editor built on VS Code)
- **[vscode.toml](vscode.toml)** - Visual Studio Code (popular code editor)
- **[windsurf.toml](windsurf.toml)** - Windsurf IDE (Codeium's agentic code editor)
- **[zed.toml](zed.toml)** - Zed Editor (modern code editor)

## How to Use Presets

### Recommended Method: Load from GitHub

**Step 1: List available presets**

```bash
# See all available presets
drifters list-presets
```

**Step 2: Load the preset**

```bash
# Load Cursor preset directly from GitHub
drifters load-preset cursor

# Or VS Code
drifters load-preset vscode

# Or Windsurf
drifters load-preset windsurf

# This automatically:
# 1. Fetches the preset from GitHub
# 2. Adds the app to your sync-rules.toml
# 3. Commits and pushes to your repo
# 4. Makes it available to all machines
```

**Step 3: Apply on this machine**

```bash
# Preview what will change
drifters merge --app vscode --dry-run

# Apply the configuration
drifters merge --app vscode
```

**Step 4: Apply on other machines**

```bash
# On each other machine:
drifters pull              # Get updated sync-rules.toml
drifters merge --app vscode  # Apply the new app config
```

### Alternative Method 1: Import from Local File

If you have the drifters repository cloned locally:

**Step 1: Import the preset**

```bash
# Import from local preset file
drifters import-app cursor --file presets/cursor.toml

# Or VS Code
drifters import-app vscode --file presets/vscode.toml

# This automatically:
# 1. Adds the app to your sync-rules.toml
# 2. Commits and pushes to your repo
# 3. Makes it available to all machines
```

**Step 2: Apply on this machine**

```bash
drifters merge --app vscode
```

**Step 3: Apply on other machines**

```bash
# On each other machine:
drifters pull
drifters merge --app vscode
```

### Alternative Method 2: Manual Edit

If you prefer manual control:

**Step 1: Clone your config repo**

```bash
# Clone to a permanent location for editing
git clone git@github.com:username/my-configs.git ~/my-drifters-config
```

**Step 2: Add preset to sync-rules.toml**

```bash
# View the preset first
cat presets/vscode.toml

# Append to your sync-rules.toml
cat presets/vscode.toml >> ~/my-drifters-config/.drifters/sync-rules.toml

# Or manually edit
vim ~/my-drifters-config/.drifters/sync-rules.toml
# Copy the [apps.vscode] section and customize
```

**Step 3: Commit and push**

```bash
cd ~/my-drifters-config
git add .drifters/sync-rules.toml
git commit -m "Add VS Code preset"
git push
```

**Step 4: Apply on all machines**

```bash
drifters merge --app vscode
```

### Quick Example (Recommended: Load from GitHub)

```bash
# 1. List available presets
drifters list-presets

# 2. Load the preset
drifters load-preset vscode

# 3. Apply on this machine
drifters merge --app vscode

# 4. On other machines:
drifters pull
drifters merge --app vscode
```

### Quick Example (Import from Local File)

```bash
# 1. Import the preset
drifters import-app vscode --file ~/projects/drifters/presets/vscode.toml

# 2. Apply on this machine
drifters merge --app vscode

# 3. On other machines:
drifters pull
drifters merge --app vscode
```

### Quick Example (Manual Edit Method)

```bash
# 1. Clone your config repo
git clone git@github.com:username/my-configs.git ~/my-drifters-config

# 2. Add preset
cat ~/projects/drifters/presets/vscode.toml >> ~/my-drifters-config/.drifters/sync-rules.toml

# 3. Commit and push
cd ~/my-drifters-config
git add .drifters/sync-rules.toml
git commit -m "Add VS Code config"
git push

# 4. Apply on this machine
drifters merge --app vscode

# 5. On other machines
# (run the same merge command)
```

## Customizing Presets

Presets are starting points. After adding a preset, customize it for your needs:

1. **Add machine-specific exclusions:**
   ```toml
   [apps.zed.machines.laptop]
   exclude = ["**/keymap.json"]
   ```

2. **Add OS-specific files:**
   ```toml
   include-linux = [
       "~/.local/share/zed/custom-plugin.json"
   ]
   ```

3. **Use section tags for fine-grained control:**
   In your settings.json:
   ```json
   {
     "theme": "dark",

     // drifters::exclude::start
     "local_only_setting": true
     // drifters::exclude::stop
   }
   ```

## Contributing Presets

Want to add a preset for your favorite app? See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

### Preset Template

```toml
# App Name - Brief description
# https://app-website.com
#
# Brief explanation of what this preset syncs
#
# Usage:
#   drifters load-preset <app>
#   drifters import-app <app> --file presets/<app>.toml

[apps.<app>]
include = [
    "~/.config/<app>/**/*.conf",
]

exclude = [
    "~/.config/<app>/cache/**",
]

# Add OS-specific variants as needed
include-macos = []
include-linux = []
include-windows = []

# Document any machine-specific customization examples
```

## Community Presets

For more app presets, check out:
- [Community Presets Repository](https://github.com/your-org/drifters-presets) (Coming soon)
- [Awesome Drifters](https://github.com/your-org/awesome-drifters) - Curated list of presets

## Need Help?

- Create an [Issue](https://github.com/anthropics/drifters/issues) if a preset isn't working
- Submit a [Pull Request](https://github.com/anthropics/drifters/pulls) with fixes or new presets
