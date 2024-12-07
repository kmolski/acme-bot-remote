// Copyright (C) 2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;
use std::sync::{Arc, Mutex, Weak};

use leptos::logging;
use serde::Serialize;
use thiserror::Error;
use typify::import_types;

pub use stomp::StompUrl;

use crate::player::{MusicPlayerState, Player, PlayerSnapshot, TrackSnapshot};

mod stomp;

import_types!("src/remote_api/schema.json");

#[derive(Clone)]
pub struct RemotePlayer {
    client: Arc<Mutex<stomp::StompClient>>,
    access_code: i64,
    destination: String,
}

#[derive(Error, Debug)]
enum RemotePlayerError {
    #[error("serialize error")]
    SerializeError(#[from] serde_json::Error),

    #[error("publish error")]
    PublishError(#[from] stomp::StompClientError),
}

impl RemotePlayer {
    pub fn new<M>(
        url: StompUrl,
        login: &str,
        password: &str,
        remote_id: &str,
        access_code: i64,
        on_message: M,
    ) -> Self
    where
        M: Fn(&str) + 'static,
    {
        let exchange = format!("/exchange/acme_bot_remote_update/{remote_id}.{access_code}");
        let client = Arc::new_cyclic(move |weak: &Weak<Mutex<stomp::StompClient>>| {
            let weak_ref = weak.clone();
            let mut client =
                stomp::StompClient::new(&url, login, password, on_message, move |_| {
                    if let Some(arc) = weak_ref.upgrade() {
                        let mut client = arc.lock().expect("lock poisoned");
                        if client.connected() {
                            if let Err(e) = client.subscribe(&exchange) {
                                logging::warn!("stomp subscribe error: {e:?}");
                            }
                            if let Err(e) = client.publish("", &exchange) {
                                logging::warn!("stomp publish error: {e:?}");
                            }
                        }
                    }
                });
            client.connect();
            Mutex::new(client)
        });

        Self {
            client,
            access_code,
            destination: format!("/exchange/acme_bot_remote/{remote_id}"),
        }
    }

    fn publish_json(&self, msg: impl Serialize) -> Result<(), RemotePlayerError> {
        let msg = serde_json::to_string(&msg)?;
        let mut client = self.client.lock().expect("lock poisoned");
        Ok(client.publish(&msg, &self.destination)?)
    }
}

impl Player for RemotePlayer {
    fn clear(&self) -> Result<(), impl Error> {
        let cmd = ClearCommand {
            op: "clear".to_string(),
            code: self.access_code,
        };
        self.publish_json(cmd)
    }

    fn move_to(&self, offset: usize, id: &str) -> Result<(), impl Error> {
        let cmd = MoveCommand {
            op: "move".to_string(),
            code: self.access_code,
            offset: offset as i64,
            id: id.to_string(),
        };
        self.publish_json(cmd)
    }

    fn pause(&self) -> Result<(), impl Error> {
        let cmd = PauseCommand {
            op: "pause".to_string(),
            code: self.access_code,
        };
        self.publish_json(cmd)
    }

    fn prev(&self) -> Result<(), impl Error> {
        let cmd = PrevCommand {
            op: "prev".to_string(),
            code: self.access_code,
        };
        self.publish_json(cmd)
    }

    fn resume(&self) -> Result<(), impl Error> {
        let cmd = ResumeCommand {
            op: "resume".to_string(),
            code: self.access_code,
        };
        self.publish_json(cmd)
    }

    fn set_loop(&self, enabled: bool) -> Result<(), impl Error> {
        let cmd = LoopCommand {
            op: "loop".to_string(),
            code: self.access_code,
            enabled,
        };
        self.publish_json(cmd)
    }

    fn set_volume(&self, value: u8) -> Result<(), impl Error> {
        let cmd = VolumeCommand {
            op: "volume".to_string(),
            code: self.access_code,
            value: value as i64,
        };
        self.publish_json(cmd)
    }

    fn skip(&self) -> Result<(), impl Error> {
        let cmd = SkipCommand {
            op: "skip".to_string(),
            code: self.access_code,
        };
        self.publish_json(cmd)
    }
}

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
