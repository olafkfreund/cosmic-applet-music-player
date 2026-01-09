# Music Player Applet for the COSMIC™ Desktop

A modern music player applet for the COSMIC™ desktop with MPRIS integration, providing seamless control of your music directly from the panel.

## Screenshots

### Main Controls Tab
<img src="Music_Player_Applet_Controls.png" alt="Controls Tab">

The main controls interface provides:
- Album artwork display
- Song title and artist information
- Media control buttons (previous, play/pause, next)
- Volume control slider

### Settings Tab
<img src="Music_Player_Applet_Settings.png" alt="Settings Tab">

The settings interface allows you to:
- Discover available media players
- Select which player to control
- Enable/disable auto-detection of new players

## Features

### 🎵 **Music Control**
- **Play/Pause**: Toggle playback with a single click
- **Track Navigation**: Skip to previous/next tracks
- **Real-time Status**: Shows current playback state (playing/paused/stopped)

### 🎨 **Visual Display**
- **Song Information**: Displays current song title and artist
- **Album Artwork**: Shows album covers from MPRIS-compatible players
- **Responsive UI**: Clean, modern interface that fits seamlessly in COSMIC

### 🔊 **Volume Control**
- **Precision Slider**: Fine-grained volume control (1% increments)
- **Visual Indicators**: Volume icons for easy reference

### ⌨️ **Convenient Controls**
- **Mouse Scroll**: Scroll up/down over the applet icon for next/previous track
- **Middle Click**: Middle-click the applet icon to play/pause
- **Panel Integration**: Compact icon in the panel, detailed controls in popup
- **Tabbed Interface**: Switch between Controls and Settings tabs in the popup

### 🔌 **MPRIS Compatibility**
Works with any MPRIS-compatible music player, including:
- Spotify
- VLC Media Player
- Rhythmbox
- Clementine
- MPD clients
- And many more!

### ⚙️ **Player Management**
- **Auto-Discovery**: Automatically finds available media players
- **Player Selection**: Choose which specific player to control via Settings tab
- **Smart Detection**: Shows which players are currently active/playing

## Installation

### Arch Linux (AUR)

The applet is available as an **AUR package**:

```bash
paru -S cosmic-applet-music-player-git
```

or with yay:

```bash
yay -S cosmic-applet-music-player-git
```

This will build and install the latest development version directly from Git.

### NixOS

**⚠️ Note**: This package currently has a known issue with Cargo.lock duplicate git dependencies that affects NixOS builds. Workarounds are documented below.

#### Option 1: Custom Derivation (Recommended)

Create a derivation in your NixOS configuration:

```nix
# In your configuration.nix or as a separate package file
{ lib, rustPlatform, fetchFromGitHub, pkg-config, dbus, openssl, libpulseaudio, libxkbcommon, wayland }:

rustPlatform.buildRustPackage rec {
  pname = "cosmic-ext-applet-music-player";
  version = "1.0.0";

  src = fetchFromGitHub {
    owner = "olafkfreund";  # or "Ebbo" for upstream
    repo = "cosmic-applet-music-player";
    rev = "master";  # or specific commit/tag
    hash = "";  # Use lib.fakeHash initially, then update with real hash
  };

  sourceRoot = "${src.name}/music-player";

  cargoHash = "";  # Use lib.fakeHash initially, then update with real hash

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [
    dbus
    openssl
    libpulseaudio
    libxkbcommon
    wayland
  ];

  meta = with lib; {
    description = "Music Player applet with MPRIS integration for COSMIC desktop";
    homepage = "https://github.com/Ebbo/cosmic-applet-music-player";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    mainProgram = "cosmic-ext-applet-music-player";
  };
}
```

Then add to your `configuration.nix`:

```nix
{
  environment.systemPackages = [
    (pkgs.callPackage ./path/to/music-player.nix {})
  ];
}
```

**To get the correct hashes:**

1. Set both `hash` and `cargoHash` to empty strings `""`
2. Run `nixos-rebuild build` (or `nix-build`)
3. Copy the correct hashes from the error messages
4. Update the derivation with the real hashes

#### Option 2: Using the Flake (Development)

This repository includes a `flake.nix` for development environments:

```bash
# Enter development shell with all dependencies
nix develop github:olafkfreund/cosmic-applet-music-player

# Now you can build and test
cd music-player
cargo build
cargo run
```

The development shell includes:
- Rust toolchain
- rust-analyzer (LSP)
- clippy and rustfmt
- cargo-watch
- All system dependencies

#### Option 3: Flake as Input (Future)

Once the Cargo.lock duplicate issue is resolved, you can use the flake directly:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cosmic-music-player.url = "github:olafkfreund/cosmic-applet-music-player";
  };

  outputs = { self, nixpkgs, cosmic-music-player, ... }: {
    nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      modules = [{
        environment.systemPackages = [
          cosmic-music-player.packages.x86_64-linux.default
        ];
      }];
    };
  };
}
```

#### Known Issues

This package has duplicate entries in `Cargo.lock` due to inconsistent git URL formatting in the libcosmic dependency chain. This is an upstream issue that prevents direct flake builds but can be worked around using `cargoHash` as shown in Option 1.

**For detailed information:**
- See [NIXOS-BUILD-NOTES.md](NIXOS-BUILD-NOTES.md) for technical details
- See [FLAKE-USAGE.md](FLAKE-USAGE.md) for development workflows
- Track issue: [nixos_config#128](https://github.com/olafkfreund/nixos_config/issues/128)

#### NixOS Development Workflow

```bash
# Clone and enter dev environment
git clone https://github.com/olafkfreund/cosmic-applet-music-player
cd cosmic-applet-music-player
nix develop

# Build and test
cd music-player
cargo watch -x run  # Auto-rebuild on file changes
```

### Build from Source (Other Distributions)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Ebbo/cosmic-applet-music-player.git
   cd cosmic-applet-music-player
   ```

2. **Install Just build tool** (if not already installed):
   ```bash
   cargo install just
   ```

3. **Build the applet**:
   ```bash
   just build-release
   ```

4. **Install system-wide**:
   ```bash
   sudo just install
   ```

### Prerequisites

- Rust 1.80+
- COSMIC™ desktop environment
- Just build tool (`cargo install just`)
- Git (for cloning)
- System development packages (see Building Requirements below)

## Development

For development and testing:

```bash
# Build debug version
just build-debug

# Run with debug logging
just run

# Format code and run
just dev

# Run clippy linting
just check

# Clean build artifacts
just clean
```

## Usage

### Adding the Applet to COSMIC™ Panel

After installation, you need to add the Music Player applet to your COSMIC™ panel:

1. **Open COSMIC™ Settings**
2. **Navigate to Desktop → Panel → Configure panel applets**
3. **Find "Music Player" in the available applets list**
4. **Click to add it to your panel**

The applet will now appear as a music icon in your COSMIC™ panel.

### Using the Applet

1. **Basic Control**: Click the music icon to open the control popup
2. **Controls Tab**:
   - View album artwork and song information
   - Use media control buttons (previous, play/pause, next)
   - Adjust volume with the precision slider
3. **Settings Tab**:
   - Click "Discover Players" to find available media players
   - Select which player to control from the radio button list
   - Enable/disable auto-detection of new players
4. **Quick Actions**:
   - Scroll up/down over the icon for track navigation
   - Middle-click for play/pause

## Configuration

### Player Selection

The applet provides flexible player management through the Settings tab:

1. **Auto-Discovery**: Click "Discover Players" to scan for available MPRIS-compatible players
2. **Player Selection**: Use the radio buttons to choose which player to control:
   - **None**: Disables all player control
   - **Specific Player**: Select a discovered player from the list
   - **Active players** are marked with ♪ symbol
3. **Auto-Detection**: Enable to automatically detect new players when they start

### Configuration Files

The applet stores its configuration in:
- `~/.config/cosmic/com.github.MusicPlayer/`

No manual configuration editing is typically required.

## Supported Players

Any application that implements the MPRIS D-Bus interface is supported, including:

- **Streaming Services**: Spotify, YouTube Music (browser), etc.
- **Media Players**: VLC, MPV, Clementine, Rhythmbox
- **Music Daemons**: MPD with compatible clients
- **Browser Players**: Firefox, Chrome with media playing

## Technical Details

- **Framework**: Built with libcosmic (COSMIC™'s UI toolkit)
- **Language**: Rust
- **Integration**: MPRIS D-Bus interface
- **Performance**: Lightweight, updates every 500ms
- **Memory**: Minimal footprint, efficient image caching

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## Troubleshooting

### Applet not appearing in panel
- Ensure the applet is properly installed: `which cosmic-ext-applet-music-player` should return a path
- Try restarting COSMIC™ or logging out/in
- Check COSMIC™ Settings → Desktop → Panel → Configure panel applets

### No players showing in Settings
- Click "Discover Players" to scan for available players
- Ensure your music player is running and supports MPRIS
- Try restarting your music player

### Applet not controlling music
- Check that the correct player is selected in Settings tab
- Ensure the selected player is currently running
- Some players may need to be playing music before they appear

### No album artwork
- Album artwork depends on the music player providing image URLs
- Some players may not provide artwork through MPRIS
- Local files may not have embedded artwork

### Mouse shortcuts not working
- Ensure you're hovering directly over the applet icon
- Check that no other application is intercepting mouse events

### Settings not saving
- Check file permissions in `~/.config/cosmic/`
- Ensure the directory exists and is writable

## Building Requirements

The following system packages are required for building:

- `libdbus-1-dev` (for MPRIS D-Bus communication via mpris crate)
- `pkg-config` (for dependency detection)
- `libssl-dev` (for HTTPS requests via reqwest crate for album artwork)
- `build-essential` or equivalent (C compiler for native dependencies)

### Ubuntu/Debian:
```bash
sudo apt install libdbus-1-dev pkg-config libssl-dev build-essential
```

### Fedora/RHEL:
```bash
sudo dnf install dbus-devel pkgconfig openssl-devel gcc
```

### Arch Linux:
```bash
sudo pacman -S dbus pkg-config openssl base-devel
```

### openSUSE:
```bash
sudo zypper install dbus-1-devel pkg-config libopenssl-devel gcc
```
