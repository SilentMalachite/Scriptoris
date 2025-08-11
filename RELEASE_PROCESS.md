# Release Process

This document describes the release process for Scriptoris, including versioning strategy, branching model, and deployment procedures.

## Versioning Strategy

Scriptoris follows [Semantic Versioning (SemVer) 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Incompatible API changes or significant architectural changes
- **MINOR**: New features, backward-compatible functionality additions
- **PATCH**: Bug fixes, security patches, backward-compatible changes

### Version Examples

- `0.1.0` - Initial release
- `0.1.1` - Bug fixes
- `0.2.0` - New features (buffer management, LSP support)
- `1.0.0` - First stable release
- `1.1.0` - New features on stable branch
- `2.0.0` - Breaking changes (major UI overhaul, API changes)

### Pre-release Versions

For pre-release versions, we use:

- `0.2.0-alpha.1` - Alpha releases (early development)
- `0.2.0-beta.1` - Beta releases (feature complete, testing)
- `0.2.0-rc.1` - Release candidates (ready for release)

## Branching Model

We use a simplified Git flow:

### Main Branches

- **`main`** - Stable, production-ready code
- **`develop`** - Integration branch for features

### Supporting Branches

- **Feature branches** - `feature/feature-name`
- **Release branches** - `release/0.2.0`
- **Hotfix branches** - `hotfix/0.1.1`

### Branch Rules

1. **`main`** branch is protected and requires PR reviews
2. **Direct commits to `main`** are not allowed
3. **All features** must be developed in feature branches
4. **Release branches** are created from `develop`
5. **Hotfixes** are created from `main` and merged to both `main` and `develop`

## Release Types

### Minor/Major Releases

1. **Create release branch** from `develop`
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b release/0.2.0
   ```

2. **Update version numbers** in:
   - `Cargo.toml` (workspace and all crates)
   - `CHANGELOG.md`
   - Documentation examples

3. **Final testing** on release branch
4. **Create PR** to merge release branch into `main`
5. **After merge**, tag the release
6. **Merge back** to `develop`

### Patch Releases (Hotfixes)

1. **Create hotfix branch** from `main`
   ```bash
   git checkout main
   git pull origin main
   git checkout -b hotfix/0.1.1
   ```

2. **Fix the issue** and update version
3. **Test thoroughly**
4. **Create PR** to merge into `main`
5. **Tag release** after merge
6. **Merge back** to `develop`

## Release Checklist

### Pre-Release (1-2 weeks before)

- [ ] Feature freeze on `develop` branch
- [ ] Create release branch
- [ ] Update version numbers in all `Cargo.toml` files
- [ ] Update `CHANGELOG.md` with all changes
- [ ] Update documentation for new features
- [ ] Run full test suite on multiple platforms
- [ ] Performance testing with large files
- [ ] Security audit of dependencies (`cargo audit`)
- [ ] Update README.md if needed

### Testing Phase

- [ ] Manual testing on all supported platforms:
  - [ ] Ubuntu 22.04+ (x86_64, aarch64)
  - [ ] macOS 12+ (Intel, Apple Silicon)
  - [ ] Windows 10+ (x86_64)
- [ ] Test with various terminal emulators:
  - [ ] Terminal.app (macOS)
  - [ ] iTerm2 (macOS)
  - [ ] Windows Terminal
  - [ ] gnome-terminal (Linux)
  - [ ] Alacritty (cross-platform)
- [ ] Test key features:
  - [ ] Basic editing functionality
  - [ ] Vim keybindings
  - [ ] Buffer management
  - [ ] Window splitting
  - [ ] LSP features
  - [ ] Session management
  - [ ] Unicode/Japanese character support
- [ ] Load testing with large files (>100k lines)
- [ ] Memory leak testing for long sessions

### Release Day

- [ ] Final commit with version bump
- [ ] Create and push Git tag: `git tag v0.2.0`
- [ ] Merge release branch to `main`
- [ ] GitHub Actions will automatically:
  - [ ] Run CI tests
  - [ ] Build release binaries
  - [ ] Create GitHub release
  - [ ] Publish to crates.io
- [ ] Verify release artifacts:
  - [ ] GitHub release created with binaries
  - [ ] Crates.io packages published
  - [ ] Documentation updated
- [ ] Merge `main` back to `develop`
- [ ] Delete release branch
- [ ] Announce release

### Post-Release

- [ ] Monitor for critical issues
- [ ] Update project boards/milestones
- [ ] Plan next release
- [ ] Update documentation website (if applicable)
- [ ] Social media announcement (if applicable)

## Automated Release Process

Our GitHub Actions handle most of the release process:

### CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and PR:
- Formatting check (`cargo fmt`)
- Linting (`cargo clippy`)
- Tests on multiple platforms
- Security audit
- Code coverage

### Release Workflow (`.github/workflows/release.yml`)

Triggered by version tags (`v*`):
- Cross-platform binary builds
- GitHub release creation
- Crates.io publication
- Asset uploads

## Version Bumping

### Automated with Scripts

Create a script to automate version bumping:

```bash
#!/bin/bash
# scripts/bump-version.sh

NEW_VERSION=$1

if [ -z "$NEW_VERSION" ]; then
  echo "Usage: $0 <new_version>"
  exit 1
fi

# Update workspace Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update individual crate Cargo.toml files
find crates -name Cargo.toml -exec sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" {} \;

# Update CHANGELOG.md
sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$NEW_VERSION] - $(date +%Y-%m-%d)/" CHANGELOG.md

echo "Version bumped to $NEW_VERSION"
echo "Please review changes and update CHANGELOG.md"
```

### Manual Steps

1. **Workspace** (`Cargo.toml`):
   ```toml
   [workspace.package]
   version = "0.2.0"
   ```

2. **Each crate** (`crates/*/Cargo.toml`):
   ```toml
   [package]
   version = "0.2.0"
   ```

3. **Dependencies** (if crates depend on each other):
   ```toml
   [dependencies]
   mdcore = { version = "0.2.0", path = "../mdcore" }
   ```

## Critical Path

### For Patch Releases (Hotfixes)

Timeline: Same day
1. Identify and fix critical issue
2. Create hotfix branch
3. Test fix thoroughly
4. Release immediately

### For Minor/Major Releases

Timeline: 2-3 weeks
1. **Week 1**: Feature freeze, create release branch
2. **Week 2**: Testing, bug fixes, documentation
3. **Week 3**: Final testing, release

## Communication

### Channels

- **GitHub Releases** - Primary release announcements
- **CHANGELOG.md** - Detailed change documentation
- **GitHub Issues** - Known issues and milestones
- **README.md** - Current version info

### Release Notes Template

```markdown
## Scriptoris v0.2.0 - "Feature Name"

### 🎉 Highlights
- Major new feature description
- Important improvement description

### ✨ New Features
- Feature 1 (#123)
- Feature 2 (#456)

### 🐛 Bug Fixes
- Fix for issue 1 (#789)
- Fix for issue 2 (#012)

### 📚 Documentation
- Updated installation guide
- New feature documentation

### 🔧 Technical Changes
- Dependency updates
- Performance improvements

### 💥 Breaking Changes
- Change 1 (migration guide)
- Change 2 (migration guide)

### 📦 Installation
[Standard installation instructions]
```

## Emergency Procedures

### Critical Security Issue

1. **Immediate response** (within 24 hours)
2. **Private disclosure** handling
3. **Coordinated patch release**
4. **Security advisory** publication
5. **User notification**

### Critical Bug

1. **Assess severity** and impact
2. **Create hotfix** if needed
3. **Fast-track release** process
4. **Communication** to users

---

This process ensures reliable, predictable releases while maintaining code quality and user trust.