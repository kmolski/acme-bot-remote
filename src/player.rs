// Copyright (C) 2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Duration;
use thiserror::Error;

// pub struct MusicPlayer<P: PlayerSnapshot, C: PubSubClient> {
//     access_code: String,
//     remote_id: String,
//     snapshot: P,
//     client: C,
// }

pub trait PlayerSnapshot<T: TrackSnapshot> {
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

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PubSubError {
    #[error("Not connected")]
    NotConnected,
}

pub trait PubSubClient {
    /// Start connecting to the message broker.
    fn activate(&self);

    /// Check if the client is connected to the message broker.
    fn connected(&self) -> bool;

    /// Check if the client is subscribed to a destination.
    fn subscribed(&self) -> bool;

    /// Publish a message to the given destination.
    ///
    /// # Arguments
    ///
    /// * `msg`: &str - message content
    /// * `dest`: &str - destination queue
    ///
    /// returns: Result<(), PubSubError>
    ///
    /// # Errors
    ///
    /// * `PubSubError::NotConnected` - client is not connected to the message broker
    fn publish(&self, msg: &str, dest: &str) -> Result<(), PubSubError>;

    /// Subscribe to the given destination.
    ///
    /// # Arguments
    ///
    /// * `callback`: C - callback invoked when a message is received
    /// * `dest`: &str - destination queue
    ///
    /// returns: Result<(), PubSubError>
    ///
    /// # Errors
    ///
    /// * `PubSubError::NotConnected` - client is not connected to the message broker
    fn subscribe<C>(&mut self, callback: C, dest: &str) -> Result<(), PubSubError>
    where
        C: Fn(String) + 'static;
}
