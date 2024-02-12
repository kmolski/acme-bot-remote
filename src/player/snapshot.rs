// Copyright (C) 2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use typify::import_types;

pub trait PlayerSnapshot {
    /// Check if queue loop is enabled.
    fn loop_enabled(&self) -> bool;

    /// Get the current volume level, from 0 to 100.
    fn volume(&self) -> u8;

    /// Get the current state of the player.
    fn state(&self) -> MusicPlayerState;

    /// Get an iterator over the current queue contents.
    fn queue(&self) -> &[impl TrackSnapshot];
}

/// State set for the music player.
pub enum MusicPlayerState {
    Idle,
    Playing,
    Paused,
    Stopped,
    Disconnected,
}

pub trait TrackSnapshot {
    /// Get the unique identifier of the track.
    fn id(&self) -> &str;

    /// Get the track title.
    fn title(&self) -> &str;

    /// Get the uploader of the track.
    fn uploader(&self) -> &str;

    /// Get the duration of the track in seconds.
    fn duration(&self) -> f64;

    /// Get the track URL.
    fn webpage_url(&self) -> &str;

    /// Get the track uploader URL as an optional string.
    fn uploader_url(&self) -> Option<&str>;

    /// Get the track thumbnail URL as an optional string.
    fn thumbnail(&self) -> Option<&str>;
}

import_types!("src/remote_api.json");

impl PlayerSnapshot for PlayerModel {
    fn loop_enabled(&self) -> bool {
        self.loop_
    }

    fn volume(&self) -> u8 {
        self.volume as u8
    }

    fn state(&self) -> MusicPlayerState {
        match self.state {
            PlayerState::Idle => MusicPlayerState::Idle,
            PlayerState::Playing => MusicPlayerState::Playing,
            PlayerState::Paused => MusicPlayerState::Paused,
            PlayerState::Stopped => MusicPlayerState::Stopped,
            PlayerState::Disconnected => MusicPlayerState::Disconnected,
        }
    }

    fn queue(&self) -> &[impl TrackSnapshot] {
        self.queue.as_slice()
    }
}

impl TrackSnapshot for QueueEntry {
    fn id(&self) -> &str {
        self.id.as_deref().unwrap_or("id_unknown")
    }

    fn title(&self) -> &str {
        self.title.as_deref().unwrap_or("unknown")
    }

    fn uploader(&self) -> &str {
        self.uploader.as_deref().unwrap_or("unknown")
    }

    fn duration(&self) -> f64 {
        match self.duration {
            Some(Duration::Variant0(int)) => int as f64,
            Some(Duration::Variant1(float)) => float,
            _ => 0.0,
        }
    }

    fn webpage_url(&self) -> &str {
        self.webpage_url.as_deref().unwrap_or("javascript:void(0)")
    }

    fn uploader_url(&self) -> Option<&str> {
        self.uploader_url.as_deref()
    }

    fn thumbnail(&self) -> Option<&str> {
        self.thumbnail.as_deref()
    }
}
