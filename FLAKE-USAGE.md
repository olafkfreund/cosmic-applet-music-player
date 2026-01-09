# Flake Usage Guide

## Overview

This repository includes a `flake.nix` using **Crane** for easy NixOS integration and local development.

## ✅ What Works

1. **✅ FULL BUILDS WORKING!** - Crane successfully handles the Cargo.lock duplicate issue
2. **✅ Local development shell** - Full Rust development environment
3. **✅ Package definition** - Complete working derivation
4. **✅ Direct execution** - Run the applet with `nix run`
5. **✅ Integration ready** - Reference from your nixos_config flake

## 🎉 SUCCESS: Cargo.lock Duplicate Issue SOLVED

**Previous issue:** rustPlatform failed with duplicate git dependencies.

**Solution:** Switched to **Crane**, which handles git dependencies by fetching each source separately rather than vendoring into a single directory.

**Result:** ✅ Builds successfully, producing a working 19MB binary!

## Building and Installation

### Quick Build

```bash
# Build the package
nix build github:olafkfreund/cosmic-applet-music-player

# Or locally
nix build .#cosmic-ext-applet-music-player

# Run directly
nix run github:olafkfreund/cosmic-applet-music-player
```

### Integration with nixos_config

#### Option 1: Direct Flake Reference (Recommended)

Add to your nixos_config flake:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cosmic-music-player = {
      url = "github:olafkfreund/cosmic-applet-music-player";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, cosmic-music-player, ... }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      modules = [
        {
          environment.systemPackages = [
            cosmic-music-player.packages.x86_64-linux.default
          ];
        }
      ];
    };
  };
}
```

## Local Development (Works Now!)

The development shell works perfectly because it doesn't need to vendor dependencies:

```bash
# Enter development environment
nix develop

# Now you have:
# - Rust toolchain
# - rust-analyzer for IDE support
# - clippy for linting
# - cargo-watch for auto-rebuilding
# - All system dependencies (dbus, wayland, etc.)

# Build in development mode
cd music-player
cargo build

# Run with logging
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## What the Flake Provides

### Packages

- `packages.cosmic-ext-applet-music-player` - The main package
- `packages.default` - Alias to the main package

### Development Shell

```bash
nix develop
```

Includes:
- Rust toolchain
- rust-analyzer (LSP)
- rustfmt
- clippy
- cargo-watch
- All build dependencies

### Apps

```bash
# Run directly (once building works)
nix run github:olafkfreund/cosmic-applet-music-player
```

## Integration with nixos_config

For now, the best approach is:

1. **Use the flake for development** - `nix develop` works great
2. **Create a custom derivation** in your nixos_config using `cargoHash`
3. **Reference this repo** via `fetchFromGitHub`

Example `flake.nix` snippet for your nixos_config:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [
            (pkgs.callPackage ./pkgs/cosmic-applets/music-player {})
          ];
        })
      ];
    };
  };
}
```

## Future: When Upstream is Fixed

Once pop-os/libcosmic fixes the dependency URL consistency, or NixOS improves duplicate handling:

1. The flake will build successfully
2. You can reference it directly in your nixos_config
3. No custom derivation needed

## Development Workflow

### Quick Start

```bash
# Clone and enter dev shell
git clone https://github.com/olafkfreund/cosmic-applet-music-player
cd cosmic-applet-music-player
nix develop

# Start developing
cd music-player
cargo watch -x run
```

### IDE Setup

With `nix develop` active:

```bash
# VS Code
code .

# Or configure your editor to use rust-analyzer from the Nix shell
```

### Running on COSMIC

1. Build the binary: `cargo build --release`
2. Copy to a location in `$PATH`
3. Add to COSMIC panel via Settings → Panel → Configure

## Documentation

- **Build issues:** See `NIXOS-BUILD-NOTES.md`
- **Architecture:** See `CLAUDE.md`
- **GitHub issue:** https://github.com/olafkfreund/nixos_config/issues/128

## Summary

✅ **Flake created and working for development**
⚠️ **Building blocked by upstream Cargo.lock duplicate issue**
📝 **Workarounds documented for nixos_config integration**
🚀 **Ready for development and testing**

The flake is production-ready for development workflows. For deployment, use the custom derivation approach in your nixos_config until the upstream issue is resolved.
