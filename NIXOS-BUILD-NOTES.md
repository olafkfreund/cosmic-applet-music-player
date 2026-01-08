# NixOS Build Notes

## Cargo.lock Duplicate Package Issue

### Problem Summary

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

## NixOS Solution

### Option 1: Post-Fetch Cargo.lock Deduplication (Recommended)

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

### Option 2: Use cargoHash Instead of cargoLock

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

### Option 3: Patch Cargo.lock in postPatch

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

### Option 4: Wait for Upstream Fix

Report the issue to https://github.com/pop-os/libcosmic requesting consistent URL formatting in their dependency specifications.

## Recommended Approach

For immediate progress: **Use Option 2 (`cargoHash`)** - it's the simplest and will work immediately.

For production: **Use Option 1 with proper deduplication** once you verify the build works.

## Testing

After creating your Nix derivation:

```bash
# Build the package
nix-build -E 'with import <nixpkgs> {}; callPackage ./path/to/default.nix {}'

# Or if using flakes
nix build .#cosmic-applet-music-player
```

## Additional Build Dependencies

Ensure your NixOS derivation includes these system dependencies:

```nix
nativeBuildInputs = [
  pkg-config
];

buildInputs = [
  dbus            # For MPRIS D-Bus communication
  openssl         # For HTTPS album artwork fetching
  # These might also be needed depending on your system:
  # xkbcommon
  # wayland
  # libxkbcommon
];
```

## References

- Original issue: https://github.com/olafkfreund/nixos_config/issues/128
- Upstream issue: https://github.com/Ebbo/cosmic-applet-music-player/issues/6
- libcosmic repository: https://github.com/pop-os/libcosmic
