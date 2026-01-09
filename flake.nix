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

        # Common arguments for crane
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;

          # Build from the music-player subdirectory
          pname = "cosmic-ext-applet-music-player";
          version = "1.0.0";

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            dbus
            openssl
            libpulseaudio
            libxkbcommon
            wayland
          ];

          # Required for Wayland support
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

          # Install desktop file and metainfo for COSMIC to discover the applet
          postInstall = ''
            install -Dm644 res/com.github.MusicPlayer.desktop \
              $out/share/applications/com.github.MusicPlayer.desktop

            install -Dm644 res/com.github.MusicPlayer.metainfo.xml \
              $out/share/metainfo/com.github.MusicPlayer.metainfo.xml

            # Install icons if they exist
            for icon in res/icons/hicolor/*/apps/com.github.MusicPlayer.svg; do
              if [ -f "$icon" ]; then
                size=$(echo $icon | grep -oP '\d+x\d+')
                install -Dm644 "$icon" \
                  "$out/share/icons/hicolor/$size/apps/com.github.MusicPlayer.svg"
              fi
            done
          '';

          meta = with pkgs.lib; {
            description = "Music Player applet with MPRIS integration for COSMIC desktop";
            homepage = "https://github.com/olafkfreund/cosmic-applet-music-player";
            license = licenses.gpl3Only;
            maintainers = with maintainers; [ ];
            platforms = platforms.linux;
            mainProgram = "cosmic-ext-applet-music-player";
          };
        });
      in
      {
        packages = {
          cosmic-ext-applet-music-player = cosmic-music-player;
          default = cosmic-music-player;
        };

        # Crane provides a much better development shell
        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
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

        # Checks (clippy, tests, etc.)
        checks = {
          # Run clippy on the workspace
          workspace-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Run tests
          workspace-test = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
            partitions = 1;
            partitionType = "count";
          });

          # Check formatting
          workspace-fmt = craneLib.cargoFmt {
            src = ./.;
            cargoExtraArgs = "--manifest-path music-player/Cargo.toml";
          };
        };
      }
    );
}
