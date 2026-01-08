# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a COSMIC desktop applet for music player control via MPRIS. It's written in Rust using the libcosmic UI framework and provides both single-player and multi-player control modes with PulseAudio/PipeWire fallback for volume control.

**Binary Name**: `cosmic-ext-applet-music-player`
**App ID**: `com.github.MusicPlayer`
**Minimum Rust**: 1.80

## Build Commands

All commands use `just` from the project root:

```bash
# Build release version (default)
just build-release

# Build debug version
just build-debug

# Run with debug logging
just run

# Format and run (development workflow)
just dev

# Run clippy linting
just check

# Run clippy with JSON output
just check-json

# Clean build artifacts
just clean

# Install system-wide (requires sudo)
sudo just install

# Uninstall
sudo just uninstall
```

**Note**: All `just` commands automatically `cd` into `music-player/` before executing cargo commands. The actual working crate is `music-player/`, not the workspace root.

## Architecture

### Module Structure

- **main.rs**: Entry point, runs `CosmicAppletMusic` via cosmic applet runtime
- **app.rs**: Core application state and message handling
  - **app/view.rs**: Main applet icon view
  - **app/view_window.rs**: Popup window UI (Controls and Settings tabs)
  - **app/subscription.rs**: Event subscriptions (player updates, mouse events)
- **music.rs**: MPRIS player interaction via `mpris` crate
- **audio.rs**: PulseAudio/PipeWire volume control via `pactl` command
- **config.rs**: Persistent config via `cosmic-config`

### Key Design Patterns

**State Management**:
- Single `CosmicAppletMusic` struct holds all state
- Message-based updates via `update()` method
- Uses `Rc<RefCell<>>` for shared mutable player state
- Config persists to `~/.config/cosmic/com.github.MusicPlayer/`

**Player Modes**:
- **Single-player mode** (default): Controls one selected player via `selected_player` config
- **Multi-player mode**: Shows all discovered players simultaneously when `show_all_players` is enabled

**Volume Control Strategy**:
- Primary: MPRIS `set_volume()` for compliant players
- Fallback: `pactl` commands for browsers (Firefox, Chrome) that don't support MPRIS volume control
- `AudioController` parses `pactl list sink-inputs` output to match players by application name

**Firefox Deduplication**:
- Firefox creates multiple MPRIS instances (one per tab)
- `get_all_players_info()` deduplicates by prioritizing: Playing > Paused > Stopped

**Album Art Handling**:
- Async HTTP fetch via `reqwest` when `art_url` changes
- Cached in `album_art_handle` (single-player) or `player_album_arts` HashMap (multi-player)
- Images converted to `cosmic::iced::widget::image::Handle::from_bytes()`

### Configuration Schema

```rust
pub struct AppConfig {
    pub enabled_players: HashSet<String>,
    pub auto_detect_new_players: bool,
    pub selected_player: Option<String>,
    pub show_all_players: bool,
    pub hide_inactive_players: bool,
}
```

### Message Flow Examples

**Play/Pause**:
1. User clicks button → `Message::PlayPause`
2. `handle_play_pause()` calls `music_controller.play_pause()`
3. Immediately toggles UI status for responsiveness
4. Dispatches `Message::FindPlayer` to sync actual state

**Volume Change**:
1. User moves slider → `Message::VolumeChanged(f64)`
2. Try MPRIS `set_volume()` first
3. On failure, use `AudioController` to set via `pactl set-sink-input-volume`

**Album Art Loading**:
1. `UpdatePlayerInfo` detects URL change
2. `LoadAlbumArt(url)` spawns async task
3. HTTP fetch via reqwest
4. `AlbumArtLoaded(handle)` stores result

## Testing Strategy

This project currently has no automated tests. When adding tests:
- Use `cargo test` in the `music-player/` directory
- Mock MPRIS interactions for player control tests
- Mock HTTP requests for album art tests
- Test configuration persistence

## Common Development Tasks

### Adding a New Player Control Action

1. Add message variant to `Message` enum in `app.rs`
2. Add handler method `handle_*()` in impl block
3. Add corresponding method to `MusicController` in `music.rs`
4. Wire up in `update()` match statement
5. Add UI element in `view.rs` or `view_window.rs`

### Modifying the Popup UI

- Single-player UI: `app/view_window.rs` → `view_window()`
- Controls tab vs Settings tab logic in `match self.active_tab`
- Use libcosmic widgets: `widget::button`, `widget::slider`, `widget::image`

### Adding Configuration Options

1. Add field to `AppConfig` struct in `config.rs`
2. Add getter/setter methods to `ConfigManager`
3. Update `Default` impl
4. Increment `CONFIG_VERSION` if schema changes
5. Add UI toggle/selection in Settings tab

## Dependencies

**COSMIC Framework**:
- `libcosmic` (pinned to rev `52b802a`): UI toolkit, applet runtime
- `cosmic-config`: Configuration persistence

**MPRIS Integration**:
- `mpris` (2.0.1): D-Bus player discovery and control

**Audio Fallback**:
- `libpulse-binding`: PulseAudio bindings (used indirectly via pactl)
- Actual implementation uses `Command::new("pactl")` for compatibility

**Async Runtime**:
- `tokio` with "full" features for async tasks
- `futures` for stream combinators

**Utilities**:
- `reqwest`: HTTP client for album art
- `image`: Image processing
- `anyhow`: Error handling
- `serde` + `toml`: Config serialization

## Workspace Configuration

Root `Cargo.toml` defines:
- `[workspace]` with member `music-player`
- `[profile.release]` with `lto = "fat"`
- Workspace lints: `clippy::todo = "warn"`, `clippy::unwrap_used = "warn"`

Project enforces:
- No `todo!()` macros in production code
- Prefer `?` operator or explicit error handling over `.unwrap()`

## Clippy Configuration

`clippy.toml` may contain additional linting rules. Run `just check` to verify compliance with pedantic lints enabled.

## Internationalization

`music-player/i18n.toml` exists but is not currently utilized. String literals are hardcoded in English.

## Installation Paths

When installed system-wide:
- Binary: `/usr/bin/cosmic-ext-applet-music-player`
- Desktop file: `/usr/share/applications/com.github.MusicPlayer.desktop`
- Metainfo: `/usr/share/metainfo/com.github.MusicPlayer.metainfo.xml`
- Icons: `/usr/share/icons/hicolor/{size}/apps/com.github.MusicPlayer.svg`
