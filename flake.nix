{
  description = "COSMIC music player applet with MPRIS integration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        # Common arguments for crane
        commonArgs = {
          # Include res/ directory and justfile for installation
          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type:
              (craneLib.filterCargoSources path type) ||
              (builtins.match ".*res.*" path != null) ||
              (builtins.match ".*justfile$" path != null);
          };

          # Build from the music-player subdirectory
          pname = "cosmic-ext-applet-music-player";
          version = "1.0.0";

          strictDeps = true;

          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          buildInputs = [
            pkgs.dbus
            pkgs.openssl
            pkgs.libpulseaudio
            pkgs.libxkbcommon
            pkgs.wayland
          ];

          # Required for bindgen in Wayland crates
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        # Build dependencies first (for caching)
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          # Use the music-player subdirectory
          cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
        });

        # Build the actual package
        cosmic-music-player = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--manifest-path music-player/Cargo.toml";

          # Add just to nativeBuildInputs for installation
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.just ];

          # Use the justfile install target like official COSMIC applets
          installPhase = ''
            runHook preInstall

            just --set prefix "$out" --set bin-src "target/release/cosmic-ext-applet-music-player" install

            runHook postInstall
          '';

          meta = {
            description = "Music Player applet with MPRIS integration for COSMIC desktop";
            homepage = "https://github.com/olafkfreund/cosmic-applet-music-player";
            license = pkgs.lib.licenses.gpl3Only;
            maintainers = [ ];
            platforms = pkgs.lib.platforms.linux;
            mainProgram = "cosmic-ext-applet-music-player";
          };
        });
      in
      {
        packages = {
          cosmic-ext-applet-music-player = cosmic-music-player;
          default = cosmic-music-player;
        };

        devShells.default = craneLib.devShell {
          packages = [
            pkgs.just
            pkgs.rust-analyzer
            pkgs.rustfmt
            pkgs.clippy
            pkgs.cargo-watch
          ];

          # Inherit build inputs from commonArgs
          inputsFrom = [ cosmic-music-player ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

        # Allow running the app directly with `nix run`
        apps.default = {
          type = "app";
          program = "${cosmic-music-player}/bin/cosmic-ext-applet-music-player";
        };

        # Checks (clippy, fmt)
        checks = {
          # Run clippy on the workspace
          workspace-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Check formatting
          workspace-fmt = craneLib.cargoFmt {
            src = ./.;
            pname = "cosmic-ext-applet-music-player";
            version = "1.0.0";
            cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
          };
        };
      }
    );
}
