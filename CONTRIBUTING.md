# Contributing to Drifters

Thank you for your interest in contributing to Drifters! This document provides guidelines for contributing.

## Ways to Contribute

### 1. App Presets (Easiest!)

The most valuable contribution is adding app presets for popular applications. These help users get started quickly.

#### Creating an App Preset

1. **Research the app's config locations**
   - Check documentation for config file paths
   - Note differences between macOS, Linux, and Windows
   - Identify which files should be synced vs excluded

2. **Create a preset file** in `presets/<app>.toml`:

```toml
# App Name - Brief description
# https://app-website.com
#
# What this preset syncs and why
#
# Usage:
#   drifters load-preset <app>
#   drifters import-app <app> --file presets/<app>.toml

[apps.<app>]
include = [
    "~/.config/<app>/settings.json",
]

exclude = [
    "~/.config/<app>/cache/**",
    "~/.config/<app>/sessions/**",
]

# Add OS-specific variants
include-macos = [
    "~/Library/Application Support/<app>/settings.json",
]

include-linux = []
include-windows = []

exclude-macos = []
exclude-linux = []
exclude-windows = []

# Document section examples
# Example: Use section tags in settings.json:
# {
#   "shared_setting": true,
#
#   // drifters::exclude::start
#   "machine_specific": "/local/path"
#   // drifters::exclude::stop
# }

# Machine override examples (commented)
# [apps.<app>.machines.laptop]
# exclude = ["**/keybindings.json"]
```

3. **Test the preset**
   - Use it on at least 2 different machines/OSes
   - Verify files sync correctly
   - Check that exclusions work as expected

4. **Submit a PR**
   - Include the preset file
   - Update `presets/README.md` to list the new app
   - Add any special notes or gotchas

#### App Preset Checklist

- [ ] Tested on at least 2 machines
- [ ] Tested on at least 2 different OSes (if applicable)
- [ ] Excludes cache/temporary files
- [ ] Excludes session-specific data
- [ ] Includes comments explaining what's synced and why
- [ ] Includes usage examples for section tags (if applicable)
- [ ] Follows the template format
- [ ] Added to `presets/README.md`

#### Popular Apps Needed

We'd love presets for:

**Editors:**
- VS Code
- Neovim
- Emacs
- Sublime Text
- IntelliJ/JetBrains IDEs

**Terminals:**
- Alacritty
- iTerm2
- Windows Terminal
- Warp
- Kitty

**Shells:**
- Zsh
- Bash
- Fish
- Nushell

**Development:**
- Git (global config)
- SSH config
- Tmux
- Starship
- Docker

**Other:**
- Raycast
- Alfred
- Rectangle/Magnet

### 2. Documentation

Good documentation helps everyone:

- **README improvements** - Clarify confusing sections
- **Examples** - Add real-world use cases
- **Guides** - Write how-to guides for common scenarios
- **Troubleshooting** - Add solutions to common issues

### 3. Bug Reports

Found a bug? Please include:

- Drifters version (`drifters --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs (run with `-v` flag)

**Template:**

```markdown
**Environment:**
- Drifters version: 0.1.0
- OS: macOS 14.2
- Git version: 2.42.0

**Steps to reproduce:**
1. Run `drifters push`
2. ...

**Expected:** ...
**Actual:** ...

**Logs:**
```
drifters -v push
[paste output]
```
```

### 4. Feature Requests

Before requesting a feature:

1. Check existing issues
2. Consider if it fits Drifters' philosophy:
   - Simple configuration over complexity
   - Transparent Git operations
   - Multi-machine intelligence
   - No magic/hidden behavior

**Template:**

```markdown
**Problem:** What problem does this solve?

**Proposed Solution:** How should it work?

**Alternatives Considered:** What else did you think about?

**Example:** Show how it would be used
```

### 5. Code Contributions

#### Setup

```bash
# Fork the repo, then:
git clone git@github.com:YOUR_USERNAME/drifters.git
cd drifters

# Create a branch
git checkout -b feature/my-feature

# Make changes
cargo build
cargo test

# Submit PR
git push origin feature/my-feature
```

#### Code Style

- Follow Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new functionality
- Update documentation

#### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# With verbose output
cargo test -- --nocapture
```

#### Commit Messages

Use conventional commits format:

```
feat: add support for Windows config paths
fix: correct glob pattern matching
docs: update README with merge command
test: add tests for section tag parsing
```

### 6. Pull Request Process

1. **Create an issue first** (for large changes)
2. **Fork and branch** from `main`
3. **Make focused commits** - one logical change per commit
4. **Update tests** - ensure all tests pass
5. **Update docs** - if behavior changes
6. **Submit PR** with clear description

**PR Template:**

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] App preset
- [ ] Documentation
- [ ] Other (explain)

## Testing
How was this tested?

## Checklist
- [ ] Code follows project style
- [ ] Tests pass (`cargo test`)
- [ ] Documentation updated
- [ ] Preset tested on multiple machines (if applicable)
```

## Development Guidelines

### Architecture Principles

1. **Simplicity** - Prefer simple solutions over clever ones
2. **Transparency** - Users should understand what's happening
3. **Safety** - Confirmations before destructive operations
4. **No Magic** - Explicit is better than implicit

### Code Organization

```
src/
├── cli/          # Command implementations
├── config/       # Configuration loading/saving
├── merge/        # Intelligent merging logic
├── parser/       # Section tag parsing
├── git/          # Git operations
├── sync/         # High-level sync operations
└── error.rs      # Error types
```

### Adding New Commands

1. Create `src/cli/your_command.rs`
2. Add command to `src/main.rs` enum
3. Add handler in `match` statement
4. Export from `src/cli/mod.rs`
5. Add tests
6. Update README

### Adding New Config Fields

1. Update structs in `src/config/sync_rules.rs`
2. Add `#[serde(default)]` for optional fields
3. Update schema documentation
4. Add migration notes (if breaking change)
5. Update examples

## Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what's best for the project
- Be patient with new contributors

### Getting Help

- **Questions:** Open a GitHub Discussion
- **Bugs:** Open an Issue
- **Feature Ideas:** Open an Issue for discussion first
- **Chat:** (Coming soon)

## Recognition

Contributors will be:
- Listed in `CONTRIBUTORS.md`
- Mentioned in release notes
- Credited in preset files they create

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Open an issue or discussion - we're happy to help!

---

Thank you for contributing to Drifters! 🚀
