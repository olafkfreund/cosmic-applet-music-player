use crate::app::{CosmicAppletMusic, Message, PopupTab};
use cosmic::{theme, Element};
use mpris::PlaybackStatus;

/// Size in pixels for album art display (square)
const ALBUM_ART_SIZE: f32 = 80.0;

/// Creates a standardized container for album art with consistent sizing and styling.
///
/// This helper reduces code duplication across the three album art states:
/// - Loaded artwork
/// - Loading state
/// - Fallback icon
///
/// # Arguments
/// * `content` - The widget to display inside the container (image, icon, or loading indicator)
///
/// # Returns
/// A configured Element containing the album art with:
/// - Fixed 80x80 pixel dimensions
/// - Centered content alignment
/// - Card styling theme
fn create_album_art_container<'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    cosmic::widget::container(content)
        .width(cosmic::iced::Length::Fixed(ALBUM_ART_SIZE))
        .height(cosmic::iced::Length::Fixed(ALBUM_ART_SIZE))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .class(cosmic::theme::Container::Card)
        .into()
}

pub fn view_window(app: &CosmicAppletMusic, _id: cosmic::iced::window::Id) -> Element<'_, Message> {
    let cosmic::cosmic_theme::Spacing {
        space_s, space_m, ..
    } = theme::active().cosmic().spacing;

    // Tab bar with proper alignment
    let controls_button = cosmic::widget::button::text(if app.active_tab == PopupTab::Controls {
        "● Controls"
    } else {
        "○ Controls"
    })
    .on_press(Message::SwitchTab(PopupTab::Controls));

    let settings_button = cosmic::widget::button::text(if app.active_tab == PopupTab::Settings {
        "● Settings"
    } else {
        "○ Settings"
    })
    .on_press(Message::SwitchTab(PopupTab::Settings));

    let tabs = cosmic::widget::row()
        .width(cosmic::iced::Length::Fill)
        .push(controls_button)
        .push(
            cosmic::widget::container(cosmic::widget::horizontal_space())
                .width(cosmic::iced::Length::Fill),
        )
        .push(settings_button);

    // Tab content
    let tab_content = match app.active_tab {
        PopupTab::Controls => view_controls_tab(app, space_s.into(), space_m.into()),
        PopupTab::Settings => view_settings_tab(app, space_s.into(), space_m.into()),
    };

    let content = cosmic::widget::column()
        .spacing(space_s)
        .padding(space_m)
        .push(tabs)
        .push(cosmic::widget::divider::horizontal::default())
        .push(tab_content);

    app.core
        .applet
        .popup_container(content)
        .limits(
            cosmic::iced::Limits::NONE
                .min_height(350.)
                .min_width(400.0)
                .max_width(500.0)
                .max_height(600.0),
        )
        .into()
}

/// Renders the Controls tab content for the popup window.
///
/// Displays either:
/// - Single-player mode: One player's info, album art, and controls
/// - Multi-player mode: All active players with compact cards
/// - No player state: Prompt to select a player in Settings
///
/// # Arguments
/// * `app` - Application state containing player info and configuration
/// * `space_s` - Small spacing value from theme
/// * `space_m` - Medium spacing value from theme
///
/// # Returns
/// An Element containing the Controls tab UI
fn view_controls_tab(app: &CosmicAppletMusic, space_s: f32, space_m: f32) -> Element<'_, Message> {
    // Check if "show all players" mode is enabled
    let show_all_players = app
        .config_manager
        .as_ref()
        .map(|config| config.get_show_all_players())
        .unwrap_or(false);

    if show_all_players {
        return view_all_players(app, space_s, space_m);
    }

    // Check if no player is selected (single player mode)
    let no_player_selected = app
        .config_manager
        .as_ref()
        .and_then(|config| config.get_selected_player())
        .is_none();

    if no_player_selected {
        return cosmic::widget::container(
            cosmic::widget::column()
                .spacing(space_s)
                .push(cosmic::widget::icon::from_name("audio-headphones-symbolic").size(48))
                .push(cosmic::widget::text::body("No player selected"))
                .push(cosmic::widget::text::caption(
                    "Go to Settings tab to select a media player",
                ))
                .align_x(cosmic::iced::Alignment::Center),
        )
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fixed(200.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .into();
    }

    // Album cover
    // Note: Using ContentFit::Contain instead of Cover ensures the entire image
    // renders within the container while preserving aspect ratio.
    // This fixes GitHub issue #1 where album art loaded but didn't render.
    let album_cover = if let Some(ref handle) = app.album_art_handle {
        create_album_art_container(
            cosmic::widget::image(handle.clone())
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .content_fit(cosmic::iced::ContentFit::Contain),
        )
    } else if app.player_info.art_url.is_some() {
        // Loading state
        create_album_art_container(
            cosmic::widget::column()
                .push(cosmic::widget::icon::from_name("image-loading-symbolic").size(32))
                .push(cosmic::widget::text::caption("Loading...").size(10))
                .spacing(4)
                .align_x(cosmic::iced::Alignment::Center),
        )
    } else {
        // No art available
        create_album_art_container(
            cosmic::widget::icon::from_name("audio-headphones-symbolic").size(48),
        )
    };

    let song_info = cosmic::widget::column()
        .spacing(space_s)
        .push(cosmic::widget::text::title4(&app.player_info.title))
        .push(cosmic::widget::text::body(&app.player_info.artist));

    let info_row = cosmic::widget::row()
        .spacing(space_m)
        .push(album_cover)
        .push(song_info)
        .align_y(cosmic::iced::Alignment::Center);

    let status_icon = match app.player_info.status {
        PlaybackStatus::Playing => "media-playback-pause-symbolic", // Show pause when playing
        PlaybackStatus::Paused => "media-playback-start-symbolic",  // Show play when paused
        PlaybackStatus::Stopped => "media-playback-start-symbolic", // Show play when stopped
    };

    let controls = cosmic::widget::row()
        .spacing(space_m)
        .push(
            cosmic::widget::button::icon(cosmic::widget::icon::from_name(
                "media-skip-backward-symbolic",
            ))
            .on_press(Message::Previous),
        )
        .push(
            cosmic::widget::button::icon(cosmic::widget::icon::from_name(status_icon))
                .on_press(Message::PlayPause),
        )
        .push(
            cosmic::widget::button::icon(cosmic::widget::icon::from_name(
                "media-skip-forward-symbolic",
            ))
            .on_press(Message::Next),
        )
        .align_y(cosmic::iced::Alignment::Center);

    // Volume control
    let volume_row = cosmic::widget::row()
        .spacing(space_s)
        .push(cosmic::widget::icon::from_name("audio-volume-low-symbolic").size(16))
        .push(
            cosmic::widget::slider(0.0..=1.0, app.player_info.volume, Message::VolumeChanged)
                .step(0.01)
                .width(cosmic::iced::Length::Fill),
        )
        .push(cosmic::widget::icon::from_name("audio-volume-high-symbolic").size(16))
        .align_y(cosmic::iced::Alignment::Center);

    cosmic::widget::column()
        .spacing(space_m)
        .push(info_row)
        .push(cosmic::widget::divider::horizontal::default())
        .push(
            cosmic::widget::container(controls)
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill),
        )
        .push(cosmic::widget::divider::horizontal::default())
        .push(volume_row)
        .into()
}

/// Renders the Settings tab content for the popup window.
///
/// Provides configuration options for:
/// - Multi-player mode toggle (show all players simultaneously)
/// - Hide inactive players option (when multi-player mode is on)
/// - Auto-detect new players toggle
/// - Manual player discovery button
/// - Player selection (single-player mode only)
///
/// # Arguments
/// * `app` - Application state containing configuration and discovered players
/// * `_space_s` - Small spacing value (unused)
/// * `space_m` - Medium spacing value from theme
///
/// # Returns
/// A scrollable Element containing the Settings tab UI
fn view_settings_tab(app: &CosmicAppletMusic, _space_s: f32, space_m: f32) -> Element<'_, Message> {
    // Get discovered players
    let discovered_players = app.music_controller.get_discovered_players();

    let mut settings_content = cosmic::widget::column().spacing(space_m);

    // Multi-player mode section
    settings_content = settings_content.push(cosmic::widget::text::title4("Multi-Player Mode"));

    if let Some(ref config) = app.config_manager {
        let show_all_players = config.get_show_all_players();
        let hide_inactive = config.get_hide_inactive_players();

        let show_all_checkbox =
            cosmic::widget::checkbox("Show all media players", show_all_players)
                .on_toggle(Message::ToggleShowAllPlayers);

        settings_content = settings_content.push(show_all_checkbox);

        // Only show "Hide inactive players" option when "Show all players" is enabled
        if show_all_players {
            let hide_inactive_checkbox =
                cosmic::widget::checkbox("Hide stopped players", hide_inactive)
                    .on_toggle(Message::ToggleHideInactive);

            settings_content = settings_content
                .push(cosmic::widget::text::caption(
                    "Hide players that are stopped (keep Playing and Paused)",
                ))
                .push(hide_inactive_checkbox);
        }
    }

    settings_content = settings_content.push(cosmic::widget::divider::horizontal::default());

    // Auto-detect section
    settings_content = settings_content.push(cosmic::widget::text::title4("Player Discovery"));

    if let Some(ref config) = app.config_manager {
        let auto_detect_enabled = config.get_auto_detect_new_players();

        let auto_detect_checkbox =
            cosmic::widget::checkbox("Auto-detect new players", auto_detect_enabled)
                .on_toggle(Message::ToggleAutoDetect);

        settings_content = settings_content.push(auto_detect_checkbox);
    }

    // Discover Players button
    let discover_button = cosmic::widget::button::text("Discover Players")
        .on_press(Message::DiscoverPlayers)
        .width(cosmic::iced::Length::Fill);

    settings_content = settings_content.push(discover_button);

    settings_content = settings_content.push(cosmic::widget::divider::horizontal::default());

    // Player selection section (only show in single-player mode)
    let show_all_players = app
        .config_manager
        .as_ref()
        .map(|config| config.get_show_all_players())
        .unwrap_or(false);

    if !show_all_players {
        settings_content = settings_content.push(cosmic::widget::text::title4("Player Selection"));

        settings_content = settings_content.push(cosmic::widget::text::caption(
            "Choose which media player to control:",
        ));
    } else {
        settings_content = settings_content.push(cosmic::widget::text::title4("Discovered Players"));
    }

    // Only show player selection in single-player mode
    if !show_all_players {
        // Get currently selected player
        let current_selected = if let Some(ref config) = app.config_manager {
            config.get_selected_player()
        } else {
            None
        };

        let selected_index = current_selected
            .as_ref()
            .and_then(|selected| {
                discovered_players
                    .iter()
                    .position(|p| &p.identity == selected)
            })
            .map(|idx| idx + 1)
            .or(if current_selected.is_none() {
                Some(0)
            } else {
                None
            });

        // "None" option to disable all players
        let none_radio =
            cosmic::widget::radio("None (disable all players)", 0usize, selected_index, |_| {
                Message::SelectPlayer(None)
            });
        settings_content = settings_content.push(none_radio);

        // Add radio buttons for each discovered player
        for (index, player) in discovered_players.iter().enumerate() {
            let status_text = if player.is_active {
                " (♪ currently playing)"
            } else {
                ""
            };
            let radio_text = format!("{}{}", player.identity, status_text);

            let radio = cosmic::widget::radio(
                cosmic::widget::text::body(radio_text),
                index + 1,
                selected_index,
                {
                    let player_name = player.identity.clone();
                    move |_| Message::SelectPlayer(Some(player_name.clone()))
                },
            );
            settings_content = settings_content.push(radio);
        }

        if discovered_players.is_empty() {
            settings_content = settings_content.push(cosmic::widget::text::caption(
                "No players discovered yet. Click 'Discover Players' to search.",
            ));
        }
    } else {
        // In multi-player mode, just list the discovered players
        if discovered_players.is_empty() {
            settings_content = settings_content.push(cosmic::widget::text::caption(
                "No players discovered yet. Click 'Discover Players' to search.",
            ));
        } else {
            for player in discovered_players.iter() {
                let status_text = if player.is_active { " ♪" } else { "" };
                let player_text = format!("{}{}", player.identity, status_text);

                settings_content = settings_content.push(cosmic::widget::text::body(player_text));
            }
        }
    }

    cosmic::widget::scrollable(settings_content).into()
}

/// Renders all active media players in multi-player mode.
///
/// Displays a scrollable list of compact player cards, each showing:
/// - Optional album artwork (48x48)
/// - Player name, track, and artist
/// - Playback controls (previous, play/pause, next)
/// - Volume slider (if supported by player)
///
/// # Arguments
/// * `app` - Application state containing all players info and album arts
/// * `space_s` - Small spacing value from theme
/// * `space_m` - Medium spacing value from theme
///
/// # Returns
/// A scrollable Element containing player cards
fn view_all_players(app: &CosmicAppletMusic, space_s: f32, space_m: f32) -> Element<'_, Message> {
    let hide_inactive = app
        .config_manager
        .as_ref()
        .map(|config| config.get_hide_inactive_players())
        .unwrap_or(false);

    // Filter players based on hide_inactive setting
    // Only hide Stopped players, keep Playing and Paused visible
    let players_to_show: Vec<_> = app
        .all_players_info
        .iter()
        .filter(|player| {
            if hide_inactive {
                player.status != PlaybackStatus::Stopped
            } else {
                true
            }
        })
        .collect();

    if players_to_show.is_empty() {
        return cosmic::widget::container(
            cosmic::widget::column()
                .spacing(space_s)
                .push(cosmic::widget::icon::from_name("audio-headphones-symbolic").size(48))
                .push(cosmic::widget::text::body("No media playing"))
                .push(cosmic::widget::text::caption(
                    "Start playing media in any MPRIS-compatible player",
                ))
                .align_x(cosmic::iced::Alignment::Center),
        )
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fixed(200.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .into();
    }

    let mut players_column = cosmic::widget::column().spacing(space_s);

    for player in players_to_show {
        players_column = players_column.push(view_player_card(app, player, space_s, space_m));
    }

    cosmic::widget::scrollable(players_column)
        .height(cosmic::iced::Length::Fixed(450.0))
        .into()
}

/// Renders a compact player card for multi-player mode.
///
/// Each card displays:
/// - Album artwork (48x48) if available, or loading indicator
/// - Status indicator emoji (▶/⏸/⏹)
/// - Track title and artist (truncated to 25 chars)
/// - Player identity
/// - Playback control buttons
/// - Volume slider (if player supports volume control)
///
/// # Arguments
/// * `app` - Application state for accessing album art cache
/// * `player` - Player information to display
/// * `space_s` - Small spacing value from theme
/// * `_space_m` - Medium spacing value (unused)
///
/// # Returns
/// A styled container Element with all player controls
fn view_player_card<'a>(
    app: &'a CosmicAppletMusic,
    player: &'a crate::music::PlayerInfo,
    space_s: f32,
    _space_m: f32,
) -> Element<'a, Message> {
    // Compact view with optional album art

    // Truncate long titles/artists - use shorter length to ensure controls are always visible
    let max_length = 25;
    let title = if player.title.len() > max_length {
        format!("{}...", &player.title[0..max_length])
    } else {
        player.title.clone()
    };
    let artist = if player.artist.len() > max_length {
        format!("{}...", &player.artist[0..max_length])
    } else {
        player.artist.clone()
    };

    // Compact album art (48x48) - only show if available
    let mut info_row = cosmic::widget::row().spacing(space_s);

    if let Some(handle) = app.player_album_arts.get(&player.bus_name) {
        // Show compact album art
        let compact_art = cosmic::widget::container(
            cosmic::widget::image(handle.clone())
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .content_fit(cosmic::iced::ContentFit::Contain),
        )
        .width(cosmic::iced::Length::Fixed(48.0))
        .height(cosmic::iced::Length::Fixed(48.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .class(cosmic::theme::Container::Card);

        info_row = info_row.push(compact_art);
    } else if player.art_url.is_some() {
        // Show loading placeholder
        let loading_art = cosmic::widget::container(
            cosmic::widget::icon::from_name("image-loading-symbolic").size(24),
        )
        .width(cosmic::iced::Length::Fixed(48.0))
        .height(cosmic::iced::Length::Fixed(48.0))
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .class(cosmic::theme::Container::Card);

        info_row = info_row.push(loading_art);
    }

    // Status indicator emoji
    let status_indicator = match player.status {
        PlaybackStatus::Playing => "▶",
        PlaybackStatus::Paused => "⏸",
        PlaybackStatus::Stopped => "⏹",
    };

    // Compact title column with status and identity
    let title_column = cosmic::widget::column()
        .spacing(2.0)
        .push(
            cosmic::widget::row()
                .spacing(space_s / 2.0)
                .push(cosmic::widget::text::body(status_indicator))
                .push(cosmic::widget::text::body(title).size(12))
        )
        .push(cosmic::widget::text::caption(artist).size(10))
        .push(cosmic::widget::text::caption(&player.identity).size(9))
        .width(cosmic::iced::Length::Shrink);

    info_row = info_row.push(title_column).align_y(cosmic::iced::Alignment::Center);

    let status_icon = match player.status {
        PlaybackStatus::Playing => "media-playback-pause-symbolic",
        PlaybackStatus::Paused => "media-playback-start-symbolic",
        PlaybackStatus::Stopped => "media-playback-start-symbolic",
    };

    let bus_name = player.bus_name.clone();

    // Compact controls - smaller icons
    let controls = cosmic::widget::row()
        .spacing(space_s / 2.0)
        .push(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name("media-skip-backward-symbolic").size(16)
            )
            .padding(4)
            .on_press({
                let bus_name = bus_name.clone();
                Message::PreviousPlayer(bus_name)
            }),
        )
        .push(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name(status_icon).size(16)
            )
            .padding(4)
            .on_press({
                let bus_name = bus_name.clone();
                Message::PlayPausePlayer(bus_name)
            }),
        )
        .push(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name("media-skip-forward-symbolic").size(16)
            )
            .padding(4)
            .on_press({
                let bus_name = bus_name.clone();
                Message::NextPlayer(bus_name)
            }),
        )
        .align_y(cosmic::iced::Alignment::Center);

    // Controls row - info (art + text) on left, buttons on right
    let controls_row = cosmic::widget::row()
        .spacing(space_s)
        .push(info_row)
        .push(cosmic::widget::horizontal_space())
        .push(controls)
        .align_y(cosmic::iced::Alignment::Center)
        .width(cosmic::iced::Length::Fill);

    // Volume control (only show if supported)
    let mut card_content = cosmic::widget::column()
        .spacing(space_s / 2.0)
        .push(controls_row);

    if player.can_control_volume {
        let volume_row = cosmic::widget::row()
            .spacing(space_s / 2.0)
            .push(cosmic::widget::icon::from_name("audio-volume-low-symbolic").size(12))
            .push(
                cosmic::widget::slider(0.0..=1.0, player.volume, {
                    let bus_name = bus_name.clone();
                    move |v| Message::VolumeChangedPlayer(bus_name.clone(), v)
                })
                .step(0.01)
                .width(cosmic::iced::Length::Fill),
            )
            .push(cosmic::widget::icon::from_name("audio-volume-high-symbolic").size(12))
            .align_y(cosmic::iced::Alignment::Center);

        card_content = card_content.push(volume_row);
    }

    cosmic::widget::container(card_content)
        .padding([space_s, space_s * 1.5])
        .class(cosmic::theme::Container::Card)
        .width(cosmic::iced::Length::Fill)
        .into()
}
