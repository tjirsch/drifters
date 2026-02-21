# Drifters App Presets

This directory contains pre-configured app definitions for popular applications. These presets make it easy to start syncing your favorite apps without manual configuration.

## Available Presets

- **[zed.toml](zed.toml)** - Zed Editor (modern code editor)

## How to Use Presets

### Step 1: Clone Your Config Repo

Since Drifters uses ephemeral repositories, clone your config repo separately for editing:

```bash
# Clone your config repo to a permanent location
git clone git@github.com:username/my-configs.git ~/my-drifters-config
```

### Step 2: Add Preset to sync-rules.toml

**Option A: Append entire preset**

```bash
# View the preset first
cat presets/zed.toml

# Append to your sync-rules.toml
cat presets/zed.toml >> ~/my-drifters-config/.drifters/sync-rules.toml
```

**Option B: Copy specific sections**

```bash
# Edit your sync-rules.toml
vim ~/my-drifters-config/.drifters/sync-rules.toml

# Manually copy the [apps.zed] section from presets/zed.toml
# Customize as needed (add machine overrides, change paths, etc.)
```

**Option C: Use during `drifters add` (Limited)**

```bash
drifters add zed
# The command will prompt for file patterns
# Enter them based on the preset
# Note: This doesn't support OS variants or machine overrides
```

### Step 3: Commit and Push

```bash
cd ~/my-drifters-config
git add .drifters/sync-rules.toml
git commit -m "Add Zed app from preset"
git push
```

### Step 4: Apply on All Machines

```bash
# On each machine, re-merge with the new rules
drifters merge --dry-run    # Preview changes
drifters merge              # Apply
```

### Complete Example

```bash
# 1. Clone your config repo
git clone git@github.com:username/my-configs.git ~/my-drifters-config

# 2. Add Zed preset
cat ~/projects/drifters/presets/zed.toml >> ~/my-drifters-config/.drifters/sync-rules.toml

# 3. Customize (optional)
vim ~/my-drifters-config/.drifters/sync-rules.toml
# Add machine overrides, adjust paths, etc.

# 4. Commit and push
cd ~/my-drifters-config
git add .drifters/sync-rules.toml
git commit -m "Add Zed editor config"
git push

# 5. Apply on this machine
drifters merge --app zed

# 6. Apply on other machines
# (run the same merge command on each)
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
#   drifters add <app>  # Then use this configuration

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
