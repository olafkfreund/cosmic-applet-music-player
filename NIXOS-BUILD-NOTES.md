# NixOS Build Notes

## ✅ STATUS: RESOLVED

**The Cargo.lock duplicate package issue has been successfully resolved using Crane!**

- **Solution**: Switched from `rustPlatform.buildRustPackage` to **Crane**
- **Build Status**: ✅ Working (produces ~19MB binary)
- **Date Resolved**: 2026-01-09
- **Method**: Crane fetches each git dependency separately instead of vendoring into a single directory

See [flake.nix](flake.nix) for the working implementation.

---

## Historical Context: Cargo.lock Duplicate Package Issue

> **Note**: This section documents the original problem for historical reference. The issue is now resolved using Crane.

### Problem Summary (Historical)

The `Cargo.lock` file contains duplicate entries for libcosmic packages due to inconsistent git URL formatting:
- `git+https://github.com/pop-os/libcosmic.git?rev=f6039597#...` (with `?rev=`)
- `git+https://github.com/pop-os/libcosmic#...` (without `?rev=` or `.git`)

Both URLs point to the **same commit** (`f6039597b72d3eefe2ee1d6528a04077982db238`), but Cargo treats them as different sources.

### Root Cause

This is an **upstream issue** in libcosmic's dependency chain:
1. We specify `libcosmic` with `rev = "f6039597"` in `Cargo.toml`
2. Cargo resolves this as `git+https://github.com/pop-os/libcosmic.git?rev=f6039597`
3. `libcosmic` internally depends on `cosmic-panel-config`
4. `cosmic-panel-config` depends on `cosmic-config` (and other packages) WITHOUT a rev parameter
5. Cargo resolves these as `git+https://github.com/pop-os/libcosmic#<hash>`

When Cargo regenerates the lockfile, it cannot unify these URLs because they come from different dependency resolution paths.

### Affected Packages

- `cosmic-config` (2 entries)
- `cosmic-config-derive` (2 entries)
- `iced_core` (2 entries)
- `iced_futures` (2 entries)

### Impact on NixOS Builds

`buildRustPackage` uses `cargo vendor` to create a local copy of dependencies. The vendoring process may fail with:

```
FileExistsError: [Errno 17] File exists: '.../cosmic-config-0.1.0'
```

This happens because Cargo tries to create directories for both entries, but the package name and version are identical.

## The Solution: Using Crane

### ✅ Implemented Working Solution

The project now uses **Crane** instead of `rustPlatform.buildRustPackage`. This completely solves the duplicate package issue.

**Key Insight**: Crane's architecture handles git dependencies fundamentally differently:
- `rustPlatform`: Vendors all dependencies into a single shared directory → fails on duplicate package names
- `Crane`: Fetches each git source as a separate Nix derivation → avoids the duplicate directory issue entirely

**Implementation** (see [flake.nix](flake.nix)):

```nix
{
  description = "COSMIC music player applet with MPRIS integration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          pname = "cosmic-ext-applet-music-player";
          version = "1.0.0";

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            dbus openssl libpulseaudio libxkbcommon wayland
          ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        # Build dependencies first (for caching)
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
        });

        # Build the actual package
        cosmic-music-player = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
        });
      in {
        packages.default = cosmic-music-player;
        devShells.default = craneLib.devShell {
          packages = with pkgs; [ rust-analyzer rustfmt clippy cargo-watch ];
          inputsFrom = [ cosmic-music-player ];
        };
      }
    );
}
```

**Build Results**:
- ✅ Build succeeds without errors
- ✅ Produces ~19MB stripped binary
- ✅ Includes all required dependencies
- ✅ Works with `nix build`, `nix run`, and `nix develop`

### Why Crane Works

Crane's `buildDepsOnly` and `buildPackage` functions:
1. Fetch each git dependency as a separate Nix store derivation
2. Create proper symlinks between dependencies
3. Never attempt to place multiple versions in the same directory
4. Handle the Cargo.lock duplicates transparently

This architectural difference makes Crane the ideal solution for projects with git dependency URL inconsistencies.

---

## Alternative Approaches (Historical Reference)

> **Note**: The following approaches were explored but are no longer necessary since Crane solves the issue. They are documented here for reference.

### Option A: Post-Fetch Cargo.lock Deduplication (Not Recommended)

Modify your Nix derivation to deduplicate `Cargo.lock` after fetching:

```nix
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  dbus,
  openssl,
  ...
}:

rustPlatform.buildRustPackage rec {
  pname = "cosmic-ext-applet-music-player";
  version = "1.0.0";

  src = fetchFromGitHub {
    owner = "olafkfreund";  # or "Ebbo" for upstream
    repo = "cosmic-applet-music-player";
    rev = "v${version}";  # or specific commit
    hash = "sha256-AAAA...";  # Use lib.fakeHash initially
  };

  # Deduplicate Cargo.lock after fetching
  postFetch = ''
    cd $out
    # Normalize all libcosmic URLs to use the shorter format without .git and ?rev=
    sed -i 's|git+https://github.com/pop-os/libcosmic.git?rev=f6039597#f6039597b72d3eefe2ee1d6528a04077982db238|git+https://github.com/pop-os/libcosmic#f6039597b72d3eefe2ee1d6528a04077982db238|g' Cargo.lock

    # Remove duplicate package entries
    # This requires a more sophisticated approach - see below
  '';

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    dbus
    openssl
  ];

  # Build in release mode
  buildType = "release";

  meta = with lib; {
    description = "Music Player applet with MPRIS integration for COSMIC desktop";
    homepage = "https://github.com/Ebbo/cosmic-applet-music-player";
    license = licenses.gpl3Only;
    maintainers = with maintainers; [ ];  # Add your name
    platforms = platforms.linux;
  };
}
```

### Option B: Use cargoHash Instead of cargoLock

Let Nix handle the vendoring without using the Cargo.lock:

```nix
rustPlatform.buildRustPackage rec {
  # ... same as above ...

  cargoHash = "sha256-AAAA...";  # Use lib.fakeHash initially, then fill in real hash

  # Remove the cargoLock attribute entirely
}
```

This approach:
- ✅ Simpler - no manual Cargo.lock manipulation
- ✅ Nix handles deduplication during vendoring
- ❌ Less reproducible - dependency versions may drift
- ❌ Requires rebuilding when updating

### Option C: Patch Cargo.lock in postPatch

```nix
rustPlatform.buildRustPackage rec {
  # ... same as above ...

  postPatch = ''
    # Normalize libcosmic URLs to prevent duplicates
    sed -i 's|git+https://github.com/pop-os/libcosmic.git?rev=f6039597|git+https://github.com/pop-os/libcosmic|g' Cargo.lock
    sed -i 's|#f6039597b72d3eefe2ee1d6528a04077982db238||g' Cargo.lock
  '';

  cargoHash = "sha256-AAAA...";  # Use lib.fakeHash initially
}
```

### Option D: Wait for Upstream Fix

Report the issue to https://github.com/pop-os/libcosmic requesting consistent URL formatting in their dependency specifications.

## Recommended Approach

**✅ Current Recommendation: Use Crane** (already implemented in this repository)

The project's [flake.nix](flake.nix) uses Crane and works perfectly. If you're:
- **Using this project**: Just use `nix build` or include the flake in your NixOS config
- **Building a similar project**: Consider switching to Crane if you encounter the same issue
- **Must use rustPlatform**: Try Option B (`cargoHash`) as the simplest workaround

The historical Options A-D are documented above but are no longer necessary for this project.

## Testing the Build

### ✅ This Project (Using Crane)

The flake build works out of the box:

```bash
# Build the package
nix build github:olafkfreund/cosmic-applet-music-player

# Or locally
git clone https://github.com/olafkfreund/cosmic-applet-music-player
cd cosmic-applet-music-player
nix build

# Run directly
nix run github:olafkfreund/cosmic-applet-music-player

# Development environment
nix develop
```

**Expected output**:
```
building 'cosmic-ext-applet-music-player-1.0.0'
...
[lots of build output]
...
cosmic-ext-applet-music-player> Finished release [optimized] target(s)
```

**Result**: A working ~19MB binary at `./result/bin/cosmic-ext-applet-music-player`

### Testing Alternative Approaches

If testing the historical rustPlatform approaches:

```bash
# Build with a custom derivation
nix-build -E 'with import <nixpkgs> {}; callPackage ./path/to/default.nix {}'
```

## Build Dependencies

### ✅ Crane Configuration (Current)

The Crane build automatically includes all necessary dependencies:

```nix
nativeBuildInputs = with pkgs; [
  pkg-config
];

buildInputs = with pkgs; [
  dbus              # For MPRIS D-Bus communication
  openssl           # For HTTPS album artwork fetching
  libpulseaudio     # For PulseAudio/PipeWire integration
  libxkbcommon      # For keyboard input
  wayland           # For Wayland support
];

LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";  # For bindgen
```

All of this is already configured in [flake.nix](flake.nix).

### For Custom rustPlatform Derivations

If building a custom derivation with rustPlatform, ensure you include all the above dependencies.

## Summary

**Problem**: Cargo.lock contained duplicate git dependencies with different URL formats, causing `rustPlatform.buildRustPackage` to fail during vendoring.

**Solution**: Switched to **Crane**, which handles git dependencies by fetching each as a separate Nix derivation instead of vendoring into a single directory.

**Status**: ✅ Fully resolved and working

**Build Time**: ~2-5 minutes (depending on system, with full rebuild)

**Output**: 19MB stripped binary at `./result/bin/cosmic-ext-applet-music-player`

## References

- **Successful build**: [flake.nix](flake.nix)
- **Original issue**: https://github.com/olafkfreund/nixos_config/issues/128
- **Upstream issue**: https://github.com/Ebbo/cosmic-applet-music-player/issues/6
- **Crane documentation**: https://crane.dev/
- **libcosmic repository**: https://github.com/pop-os/libcosmic

## For Other Projects

If you're encountering similar Cargo.lock duplicate issues in other Rust projects:

1. **First choice**: Switch to Crane (best long-term solution)
2. **Quick fix**: Use `cargoHash` instead of `cargoLock` with rustPlatform
3. **Report upstream**: Help the upstream project normalize their git URL formats

Crane has proven to be the most robust solution for projects with complex git dependency chains.
