{
  description = "COSMIC music player applet with MPRIS integration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Use a custom vendor script that handles duplicates
        cargoVendorDir = pkgs.stdenv.mkDerivation {
          name = "cosmic-applet-music-player-vendor";
          src = ./.;

          nativeBuildInputs = [ pkgs.cargo ];

          buildPhase = ''
            cd music-player
            export CARGO_HOME=$TMPDIR/cargo
            mkdir -p $out

            # Vendor dependencies
            cargo vendor --versioned-dirs $out 2>&1 | tee vendor.log || true

            # Check if vendoring succeeded
            if [ -d "$out/cosmic-config-0.1.0" ]; then
              echo "Vendoring completed (with expected duplicate warnings)"
            fi
          '';

          installPhase = "true";  # Already installed to $out
        };
      in
      {
        packages = {
          cosmic-ext-applet-music-player = pkgs.rustPlatform.buildRustPackage {
            pname = "cosmic-ext-applet-music-player";
            version = "1.0.0";

            src = ./.;

            # Build from the music-player subdirectory
            buildAndTestSubdir = "music-player";

            # Simple cargoHash approach - let Nix handle the vendoring
            cargoHash = "sha256-PYvv5DaxQLAEGy4ztRZQSjrJ5y5rRbhuvnLsrEt9yLg=";

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

            meta = with pkgs.lib; {
              description = "Music Player applet with MPRIS integration for COSMIC desktop";
              homepage = "https://github.com/Ebbo/cosmic-applet-music-player";
              license = licenses.gpl3Only;
              maintainers = with maintainers; [ ];
              platforms = platforms.linux;
              mainProgram = "cosmic-ext-applet-music-player";
            };
          };

          default = self.packages.${system}.cosmic-ext-applet-music-player;
        };

        # Development shell with all build dependencies
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.cosmic-ext-applet-music-player ];

          packages = with pkgs; [
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

        # Allow running the app directly with `nix run`
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/cosmic-ext-applet-music-player";
        };
      }
    );
}
