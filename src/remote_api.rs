// Copyright (C) 2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use typify::import_types;

use crate::player::{MusicPlayerState, PlayerSnapshot, TrackSnapshot};

import_types!("src/remote_api.json");

impl PlayerSnapshot<QueueEntry> for PlayerModel {
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

    fn queue(&self) -> &[QueueEntry] {
        self.queue.as_slice()
    }
}

impl Default for PlayerModel {
    fn default() -> Self {
        PlayerModel {
            loop_: true,
            volume: 100,
            position: 0,
            state: PlayerState::Idle,
            queue: vec![],
            current: None,
        }
    }
}

impl TrackSnapshot for QueueEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn uploader(&self) -> &str {
        &self.uploader
    }

    fn duration(&self) -> std::time::Duration {
        match self.duration {
            Duration::Variant0(int) => std::time::Duration::from_secs(int as u64),
            Duration::Variant1(float) => std::time::Duration::from_secs_f64(float),
        }
    }

    fn webpage_url(&self) -> &str {
        &self.webpage_url
    }

    fn uploader_url(&self) -> Option<&str> {
        self.uploader_url.as_deref()
    }

    fn thumbnail(&self) -> Option<&str> {
        self.thumbnail.as_deref()
    }
}
