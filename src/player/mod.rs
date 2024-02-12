// Copyright (C) 2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use thiserror::Error;

pub mod snapshot;

//pub struct MusicPlayer<P: PlayerSnapshot, C: PubSubClient> {
//    access_code: String,
//    remote_id: String,
//    snapshot: P,
//    client: C,
//}

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

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PubSubError {
    #[error("Not connected")]
    NotConnected,
}
