# GitHub Actions Workflows

This directory contains GitHub Actions workflows for automated building, testing, and releasing of the COSMIC Music Player Applet using Nix and Cachix.

## 📋 Workflows Overview

### 1. CI Workflow (`ci.yml`)

**Triggers:**
- Push to `master` or `main` branches
- Pull requests to `master` or `main` branches
- Manual workflow dispatch

**What it does:**
- ✅ Builds the package for x86_64-linux
- ✅ Runs clippy lint checks
- ✅ Verifies code formatting
- ✅ Validates the Nix flake
- ✅ Builds the development shell
- ✅ Caches successful builds to Cachix (master/main only)

**Jobs:**
- `nix-build`: Builds package and runs checks on multiple systems
- `nix-flake-check`: Validates the entire flake configuration
- `build-dev-shell`: Ensures development environment builds correctly
- `summary`: Aggregates results and reports overall status

### 2. Release Workflow (`release.yml`)

**Triggers:**
- Push of version tags (e.g., `v1.0.0`, `v1.2.3`)
- Manual workflow dispatch with tag input

**What it does:**
- ✅ Builds release packages for all supported architectures
- ✅ Runs complete test suite and checks
- ✅ Pushes all builds to Cachix
- ✅ Creates build artifacts
- ✅ Generates GitHub release with installation instructions
- ✅ Provides Nix installation snippets

**Jobs:**
- `build-release`: Builds release versions for all platforms
- `create-github-release`: Creates GitHub release with notes
- `release-summary`: Reports release pipeline status

## 🔧 Setup Requirements

### GitHub Secrets

You must configure the following secret in your GitHub repository:

**`CACHIX_KEY`** - Your Cachix authentication token or signing key
- **Required for:** Pushing built packages to Cachix
- **Where to get it:** [cachix.org](https://app.cachix.org) → Your cache → Settings
- **How to add:** GitHub repo → Settings → Secrets and variables → Actions → New repository secret

### Cachix Cache Configuration

The workflows use a Cachix cache named: `cosmic-applet-music-player`

**Setup your Cachix cache:**

1. **Create cache on Cachix:**
   ```bash
   # Login to Cachix
   cachix authtoken YOUR_TOKEN

   # Create a new cache (if not exists)
   cachix create cosmic-applet-music-player
   ```

2. **Get your auth token:**
   - Visit [https://app.cachix.org](https://app.cachix.org)
   - Go to your cache settings
   - Copy the authentication token

3. **Add to GitHub secrets:**
   - Repository → Settings → Secrets and variables → Actions
   - New repository secret: `CACHIX_KEY`
   - Paste your token

## 🚀 Usage

### Running CI Builds

CI runs automatically on:
- Every push to master/main
- Every pull request

To manually trigger:
1. Go to Actions tab
2. Select "CI" workflow
3. Click "Run workflow"

### Creating a Release

Two methods:

**Method 1: Git Tag (Recommended)**
```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0

# Workflow triggers automatically
```

**Method 2: Manual Dispatch**
1. Go to Actions tab
2. Select "Release" workflow
3. Click "Run workflow"
4. Enter version tag (e.g., `v1.0.0`)

## 📦 Cachix Integration

### How It Works

1. **On Build:**
   - Nix builds the package
   - Successful builds are pushed to Cachix
   - Derivations and outputs are cached

2. **On Subsequent Builds:**
   - Workflow checks Cachix first
   - Downloads pre-built packages if available
   - Builds locally only if cache miss

3. **For Users:**
   - Add the binary cache: `cachix use cosmic-applet-music-player`
   - Install instantly without building: `nix profile install github:olafkfreund/cosmic-applet-music-player`

### Benefits

- ⚡ **Faster CI**: Builds complete in seconds instead of minutes
- 💾 **Lower Resource Usage**: No redundant compilation
- 🌍 **Faster User Installs**: Users download pre-built binaries
- 🔄 **Consistent Builds**: Same binaries across all users

## 🏗️ Architecture Details

### Supported Architecture

The workflows build for:
- **x86_64-linux**: Intel/AMD 64-bit (ubuntu-latest)

### Caching Strategy

**Push to Cachix when:**
- ✅ Push to master/main branches (CI workflow)
- ✅ Release tags (Release workflow)

**Skip Cachix push when:**
- ❌ Pull requests from forks (security)
- ❌ Pull requests to non-main branches
- ❌ Manual workflow runs on feature branches

### Concurrency Control

The CI workflow uses GitHub's concurrency control:
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

This ensures:
- Only one workflow runs per branch at a time
- New pushes cancel in-progress runs
- Saves compute resources and speeds up feedback

## 📊 Monitoring

### Check Workflow Status

1. **GitHub Actions Tab:**
   - View all workflow runs
   - See detailed logs
   - Download artifacts

2. **Cachix Dashboard:**
   - Visit [https://app.cachix.org](https://app.cachix.org)
   - View cache statistics
   - Monitor storage usage
   - Check download counts

3. **CI Badge (Optional):**
   Add to README.md:
   ```markdown
   ![CI](https://github.com/olafkfreund/cosmic-applet-music-player/workflows/CI/badge.svg)
   ```

## 🔍 Troubleshooting

### Common Issues

**1. "Authorization required" error**
- Check that `CACHIX_KEY` secret is set correctly
- Verify the token has write permissions
- Ensure cache name matches: `cosmic-applet-music-player`

**2. "Cannot push to cachix" error**
- Verify you're pushing from master/main branch
- Check `skipPush` condition in workflow
- Confirm Cachix token has push permissions

**3. Build failures**
- Check Nix flake for errors: `nix flake check`
- Review build logs in Actions tab
- Test locally: `nix build .#cosmic-ext-applet-music-player`

### Debug Locally

Test workflows locally before pushing:

```bash
# Install act (GitHub Actions local runner)
nix-shell -p act

# Run CI workflow locally
act push -W .github/workflows/ci.yml

# Run specific job
act push -j nix-build
```

## 📚 References

- [Cachix Documentation](https://docs.cachix.org/)
- [Cachix GitHub Action](https://github.com/cachix/cachix-action)
- [Install Nix Action](https://github.com/cachix/install-nix-action)
- [Nix Flakes](https://nixos.wiki/wiki/Flakes)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

## 🆘 Support

For issues with:
- **Workflows**: Open issue in this repository
- **Cachix**: [Cachix Support](https://github.com/cachix/cachix/issues)
- **Nix**: [NixOS Discourse](https://discourse.nixos.org/)

---

**Last Updated:** 2026-01-16
**Workflow Version:** 1.0.0
