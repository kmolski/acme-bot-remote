// Copyright (C) 2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use typify::import_types;

pub trait PlayerSnapshot {
    fn loop_enabled(&self) -> bool;

    fn volume(&self) -> u8;

    fn state(&self) -> MusicPlayerState;

    fn queue(&self) -> impl Iterator<Item = &impl TrackSnapshot>;
}

pub enum MusicPlayerState {
    Idle,
    Playing,
    Paused,
    Stopped,
    Disconnected,
}

pub trait TrackSnapshot {
    fn id(&self) -> &str;

    fn title(&self) -> &str;

    fn uploader(&self) -> &str;

    fn duration(&self) -> f64;

    fn webpage_url(&self) -> &str;

    fn uploader_url(&self) -> Option<&str>;

    fn thumbnail(&self) -> Option<&str>;
}

import_types!("src/remote_api.json");

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

    fn queue(&self) -> impl Iterator<Item = &impl TrackSnapshot> {
        self.queue.iter()
    }
}
