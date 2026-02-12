use crate::audio::AudioController;
use anyhow::Result;
use mpris::{PlaybackStatus, Player, PlayerFinder};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub title: String,
    pub artist: String,
    pub status: PlaybackStatus,
    pub volume: f64,
    pub art_url: Option<String>,
    pub bus_name: String,
    pub identity: String,
    pub can_control_volume: bool,
}

#[derive(Debug, Clone)]
pub struct DiscoveredPlayer {
    pub identity: String,
    pub is_active: bool,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            title: "No music playing".to_string(),
            artist: String::new(),
            status: PlaybackStatus::Stopped,
            volume: 0.5,
            art_url: None,
            bus_name: String::new(),
            identity: String::new(),
            can_control_volume: true,
        }
    }
}

#[derive(Clone)]
pub struct MusicController {
    player: Rc<RefCell<Option<Player>>>,
    discovered_players: Rc<RefCell<HashMap<String, DiscoveredPlayer>>>,
    all_players: Rc<RefCell<HashMap<String, Player>>>,
    audio_controller: Option<Arc<AudioController>>,
}

impl MusicController {
    pub fn new() -> Self {
        let audio_controller = Arc::new(AudioController::new());

        Self {
            player: Rc::new(RefCell::new(None)),
            discovered_players: Rc::new(RefCell::new(HashMap::new())),
            all_players: Rc::new(RefCell::new(HashMap::new())),
            audio_controller: Some(audio_controller),
        }
    }

    pub fn discover_all_players(&mut self) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        let mut discovered_borrow = self.discovered_players.borrow_mut();
        let mut all_players_borrow = self.all_players.borrow_mut();
        discovered_borrow.clear();
        all_players_borrow.clear();

        // Try to get all players
        if let Ok(players) = player_finder.find_all() {
            for player in players {
                let identity = player.identity();
                let bus_name = player.bus_name_player_name_part();
                let is_active = player
                    .get_playback_status()
                    .unwrap_or(PlaybackStatus::Stopped)
                    == PlaybackStatus::Playing;

                discovered_borrow.insert(
                    identity.to_string(),
                    DiscoveredPlayer {
                        identity: identity.to_string(),
                        is_active,
                    },
                );

                all_players_borrow.insert(bus_name.to_string(), player);
            }
        }

        Ok(())
    }

    pub fn find_active_player(&mut self) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        // Try to find the first available player
        if let Ok(player) = player_finder.find_active() {
            *self.player.borrow_mut() = Some(player);
        }

        Ok(())
    }

    pub fn find_specific_player(&mut self, player_name: &str) -> Result<()> {
        let player_finder = PlayerFinder::new()?;

        // Try to find all players and pick the one that matches the name
        if let Ok(players) = player_finder.find_all() {
            for player in players {
                let identity = player.identity();
                if identity == player_name {
                    *self.player.borrow_mut() = Some(player);
                    return Ok(());
                }
            }
        }

        // Player not found, clear current player
        *self.player.borrow_mut() = None;

        Ok(())
    }

    pub fn get_discovered_players(&self) -> Vec<DiscoveredPlayer> {
        self.discovered_players.borrow().values().cloned().collect()
    }

    /// Extract player info from an MPRIS player, resolving volume via audio
    /// controller fallback for browsers that don't support MPRIS volume.
    fn extract_player_info(&self, player: &Player, bus_name: String) -> PlayerInfo {
        let metadata = player.get_metadata().unwrap_or_default();
        let status = player
            .get_playback_status()
            .unwrap_or(PlaybackStatus::Stopped);
        let mut volume = player.get_volume().unwrap_or(0.5);

        let title = metadata
            .title()
            .map_or_else(|| "Unknown".to_string(), ToString::to_string);

        let artist = metadata
            .artists()
            .map_or_else(|| "Unknown Artist".to_string(), |a| a.join(", "));

        let art_url = metadata.art_url().map(ToString::to_string);
        let identity = player.identity().to_string();

        // For browsers, get actual volume from PulseAudio/PipeWire
        if let Some(ref audio_ctrl) = self.audio_controller {
            if let Some(sink_input) = audio_ctrl.find_sink_input_by_name(&identity) {
                volume = sink_input.volume;
            }
        }

        PlayerInfo {
            title,
            artist,
            status,
            volume,
            art_url,
            bus_name,
            identity,
            can_control_volume: true,
        }
    }

    pub fn get_player_info(&self) -> PlayerInfo {
        let player_borrow = self.player.borrow();

        let Some(ref player) = *player_borrow else {
            return PlayerInfo::default();
        };

        // Refresh audio sinks before extracting info
        if let Some(ref audio_ctrl) = self.audio_controller {
            if let Err(e) = audio_ctrl.refresh_sink_inputs() {
                eprintln!("Failed to refresh audio sink inputs: {e}");
            }
        }

        let bus_name = player.bus_name_player_name_part().to_string();
        self.extract_player_info(player, bus_name)
    }

    pub fn get_all_players_info(&self) -> Vec<PlayerInfo> {
        let all_players_borrow = self.all_players.borrow();
        let mut players_info: Vec<PlayerInfo> = Vec::new();
        let mut firefox_players: Vec<PlayerInfo> = Vec::new();

        // Refresh audio controller sink inputs once before iterating
        if let Some(ref audio_ctrl) = self.audio_controller {
            if let Err(e) = audio_ctrl.refresh_sink_inputs() {
                eprintln!("Failed to refresh audio sink inputs: {e}");
            }
        }

        for (bus_name, player) in all_players_borrow.iter() {
            let info = self.extract_player_info(player, bus_name.clone());

            // Separate Firefox players for deduplication
            if info.identity.to_lowercase().contains("firefox") {
                firefox_players.push(info);
            } else {
                players_info.push(info);
            }
        }

        // Deduplicate Firefox: keep only the most relevant one (Playing > Paused > Stopped)
        if !firefox_players.is_empty() {
            firefox_players.sort_by_key(|p| match p.status {
                PlaybackStatus::Playing => 0,
                PlaybackStatus::Paused => 1,
                PlaybackStatus::Stopped => 2,
            });

            if let Some(firefox_player) = firefox_players.into_iter().next() {
                players_info.push(firefox_player);
            }
        }

        // Sort players by identity for stable ordering (alphabetical)
        // This prevents players from jumping around when status changes
        players_info.sort_by(|a, b| a.identity.to_lowercase().cmp(&b.identity.to_lowercase()));

        players_info
    }

    /// Set volume on a player, trying MPRIS first, then audio controller fallback.
    fn set_volume_on_player(&self, player: &Player, volume: f64) -> Result<()> {
        // Try MPRIS first
        if player.set_volume(volume).is_ok() {
            return Ok(());
        }

        // If MPRIS fails, try audio controller (for browsers)
        if let Some(ref audio_ctrl) = self.audio_controller {
            let identity = player.identity();
            if let Err(e) = audio_ctrl.refresh_sink_inputs() {
                eprintln!("Failed to refresh audio sink inputs: {e}");
            }
            if let Some(sink_input) = audio_ctrl.find_sink_input_by_name(identity) {
                audio_ctrl.set_sink_input_volume(sink_input.index, volume)?;
            }
        }

        Ok(())
    }

    // --- Single-player controls (operate on self.player) ---

    pub fn play_pause(&self) -> Result<()> {
        let player_borrow = self.player.borrow();
        if let Some(ref player) = *player_borrow {
            player.play_pause()?;
        }
        Ok(())
    }

    pub fn next(&self) -> Result<()> {
        let player_borrow = self.player.borrow();
        if let Some(ref player) = *player_borrow {
            player.next()?;
        }
        Ok(())
    }

    pub fn previous(&self) -> Result<()> {
        let player_borrow = self.player.borrow();
        if let Some(ref player) = *player_borrow {
            player.previous()?;
        }
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> Result<()> {
        let player_borrow = self.player.borrow();
        if let Some(ref player) = *player_borrow {
            self.set_volume_on_player(player, volume)?;
        }
        Ok(())
    }

    // --- Multi-player controls (operate on self.all_players by bus_name) ---

    pub fn play_pause_player(&self, bus_name: &str) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();
        if let Some(player) = all_players_borrow.get(bus_name) {
            player.play_pause()?;
        }
        Ok(())
    }

    pub fn next_player(&self, bus_name: &str) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();
        if let Some(player) = all_players_borrow.get(bus_name) {
            player.next()?;
        }
        Ok(())
    }

    pub fn previous_player(&self, bus_name: &str) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();
        if let Some(player) = all_players_borrow.get(bus_name) {
            player.previous()?;
        }
        Ok(())
    }

    pub fn set_volume_player(&self, bus_name: &str, volume: f64) -> Result<()> {
        let all_players_borrow = self.all_players.borrow();
        if let Some(player) = all_players_borrow.get(bus_name) {
            self.set_volume_on_player(player, volume)?;
        }
        Ok(())
    }
}
