// Copyright (C) 2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;
use std::time::Duration;

pub trait Player {
    fn clear(&self) -> Result<(), impl Error>;

    fn move_to(&self, offset: usize, id: &str) -> Result<(), impl Error>;

    fn pause(&self) -> Result<(), impl Error>;

    fn prev(&self) -> Result<(), impl Error>;

    fn resume(&self) -> Result<(), impl Error>;

    fn set_loop(&self, enabled: bool) -> Result<(), impl Error>;

    fn set_volume(&self, value: u8) -> Result<(), impl Error>;

    fn skip(&self) -> Result<(), impl Error>;
}

pub trait PlayerSnapshot<T: TrackSnapshot>: Default {
    /// Check if queue loop is enabled.
    fn loop_enabled(&self) -> bool;

    /// Get the current volume level, from 0 to 100.
    fn volume(&self) -> u8;

    /// Get the current state of the player.
    fn state(&self) -> MusicPlayerState;

    /// Get the contents of the queue.
    fn queue(&self) -> &[T];
}

/// State set for the music player.
#[derive(Eq, PartialEq)]
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

    /// Get the duration of the track.
    fn duration(&self) -> Duration;

    /// Get the track URL.
    fn webpage_url(&self) -> &str;

    /// Get the track uploader URL as an optional string.
    fn uploader_url(&self) -> Option<&str>;

    /// Get the track thumbnail URL as an optional string.
    fn thumbnail(&self) -> Option<&str>;
}
