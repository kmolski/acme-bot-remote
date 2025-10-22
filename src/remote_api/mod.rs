// Copyright (C) 2024-2025  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;
use std::rc::Rc;

use codee::string::FromToStringCodec;
use leptos::Signal;
use leptos_use::{use_websocket_with_options, UseWebSocketOptions, UseWebSocketReturn};
use serde::Serialize;
use thiserror::Error;
use typify::import_types;

use crate::player::{MusicPlayerState, Player, PlayerSnapshot, TrackSnapshot};

import_types!("src/remote_api/schema.json");

#[derive(Clone)]
pub struct RemotePlayer {
    pub(crate) state: Signal<Option<String>>,
    send: Rc<dyn Fn(&String)>,
    access_code: i64,
}

#[derive(Error, Debug)]
enum RemotePlayerError {
    #[error("serialize error")]
    SerializeError(#[from] serde_json::Error),
}

impl RemotePlayer {
    pub fn new(url: &str, token: &str, access_code: i64) -> Self {
        let options = UseWebSocketOptions::default().protocols(Some(vec![
            "acme-bot".to_string(),
            format!("acme-bot.bearer.{token}"),
        ]));
        let UseWebSocketReturn { message, send, .. } =
            use_websocket_with_options::<String, String, FromToStringCodec>(url, options);
        Self {
            send: Rc::new(send),
            state: message,
            access_code,
        }
    }

    fn publish_json(&self, msg: impl Serialize) -> Result<(), RemotePlayerError> {
        let msg = serde_json::to_string(&msg)?;
        (*self.send)(&msg);
        Ok(())
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

    fn remove(&self, offset: usize, id: &str) -> Result<(), impl Error> {
        let cmd = RemoveCommand {
            op: "remove".to_string(),
            code: self.access_code,
            offset: offset as i64,
            id: id.to_string(),
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
