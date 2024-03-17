// Copyright (C) 2023-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};

use base64::prelude::*;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::player::{PlayerSnapshot, PubSubClient, TrackSnapshot};
use crate::remote_api::PlayerModel;
use crate::stomp::{StompClient, StompUrl};

mod player;
mod remote_api;
mod stomp;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessageType {
    Resume,
    Pause,
    Stop,
    Clear,
    Move,
    Loop,
}

#[derive(Serialize, Deserialize)]
struct Message {
    op: MessageType,
    code: String,
}

#[derive(Serialize, Deserialize)]
struct MoveMessage {
    op: MessageType,
    code: String,
    offset: usize,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct LoopMessage {
    op: MessageType,
    code: String,
    enabled: bool,
}

fn publish(op: MessageType, access_code: &str, remote_id: &str, client: &Arc<Mutex<StompClient>>) {
    let command = Message {
        op,
        code: access_code.to_string(),
    };
    let msg = serde_json::to_string(&command).unwrap();
    if let Err(e) = client
        .lock()
        .unwrap()
        .publish(&msg, &format!("/exchange/acme_bot_remote/{remote_id}"))
    {
        logging::warn!("Could not send command: {e:?}")
    }
}

fn publish_move(
    offset: usize,
    id: &str,
    access_code: &str,
    remote_id: &str,
    client: &Arc<Mutex<StompClient>>,
) {
    let command = MoveMessage {
        op: MessageType::Move,
        code: access_code.to_string(),
        offset,
        id: id.to_string(),
    };
    let msg = serde_json::to_string(&command).unwrap();
    if let Err(e) = client
        .lock()
        .unwrap()
        .publish(&msg, &format!("/exchange/acme_bot_remote/{remote_id}"))
    {
        logging::warn!("Could not send command: {e:?}")
    }
}

fn publish_loop(
    loop_enabled: bool,
    access_code: &str,
    remote_id: &str,
    client: &Arc<Mutex<StompClient>>,
) {
    let command = LoopMessage {
        op: MessageType::Loop,
        code: access_code.to_string(),
        enabled: loop_enabled,
    };
    let msg = serde_json::to_string(&command).unwrap();
    if let Err(e) = client
        .lock()
        .unwrap()
        .publish(&msg, &format!("/exchange/acme_bot_remote/{remote_id}"))
    {
        logging::warn!("Could not send command: {e:?}")
    }
}

#[component]
fn Player() -> impl IntoView {
    let query_params = use_query_map().get_untracked();
    let access_code = Rc::new(query_params.get("ac").unwrap().to_string());
    let remote_id = Rc::new(query_params.get("rid").unwrap().to_string());
    let rcs_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(query_params.get("rcs").unwrap())
        .unwrap();
    let remote_server = String::from_utf8(rcs_bytes).unwrap();

    let mut url = Url::parse(&remote_server).unwrap();
    let login = url.username().to_string();
    let password = url.password().unwrap().to_string();
    url.set_username("").unwrap();
    url.set_password(None).unwrap();

    let (snapshot, set_snapshot) = create_signal(None::<PlayerModel>);
    let remote_url = StompUrl::new(url.as_str()).unwrap();
    let exchange = format!("/exchange/acme_bot_remote_update/{remote_id}.{access_code}");
    let (tracks, set_tracks) = create_signal(String::new());
    let client: Arc<Mutex<StompClient>> = Arc::new_cyclic(|weak_ref: &Weak<Mutex<StompClient>>| {
        let weak = weak_ref.clone();
        Mutex::new(StompClient::new(
            &remote_url,
            &login,
            &password,
            Some(move |_| {
                if let Some(arc) = weak.upgrade() {
                    let mut client = arc.lock().expect("lock poisoned");
                    if client.connected() {
                        logging::log!("SUBBING!");
                        if let Err(e) = client.subscribe(
                            move |m| {
                                logging::log!("Message received: {}", m);
                                match serde_json::from_str(&m) {
                                    Ok(p) => set_snapshot.set(Some(p)),
                                    Err(e) => logging::error!("Invalid snapshot: {}", e),
                                }
                                set_tracks.set(m);
                            },
                            &exchange,
                        ) {
                            leptos::logging::warn!("{e:?}");
                        }
                        if let Err(e) = client.publish("", &exchange) {
                            leptos::logging::warn!("{e:?}");
                        }
                        logging::log!("DONE!");
                    }
                }
            }),
        ))
    });
    let _arc = client.clone();
    {
        match client.lock() {
            Ok(c) => c.activate(),
            Err(e) => leptos::logging::warn!("{e:?}"),
        }
    }

    view! {
        <div>{move || tracks.get()}</div>
        <div>
            <input type="checkbox" id="loop"
                prop:checked=move || {
                    let snap = snapshot.get();
                    snap.as_ref().map(PlayerSnapshot::loop_enabled).unwrap_or(false)
                }
                on:change={
                    let access_code = access_code.clone();
                    let remote_id = remote_id.clone();
                    let client = client.clone();
                    move |e| { publish_loop(event_target_checked(&e), &access_code, &remote_id, &client); }}/>
            <label for="loop">"Loop"</label>
        </div>
        <button on:click={
            let access_code = access_code.clone();
            let remote_id = remote_id.clone();
            let client = client.clone();
            move |_| { publish(MessageType::Resume, &access_code, &remote_id, &client); }}>
            "Resume"
        </button>
        <button on:click={
            let access_code = access_code.clone();
            let remote_id = remote_id.clone();
            let client = client.clone();
            move |_| { publish(MessageType::Pause, &access_code, &remote_id, &client); }}>
            "Pause"
        </button>
        <button on:click={
            let access_code = access_code.clone();
            let remote_id = remote_id.clone();
            let client = client.clone();
            move |_| { publish(MessageType::Clear, &access_code, &remote_id, &client); }}>
            "Clear"
        </button>
        <button on:click={
            let access_code = access_code.clone();
            let remote_id = remote_id.clone();
            let client = client.clone();
            move |_| {
                let idx = snapshot.get().unwrap().queue().len() - 1;
                publish_move(idx, snapshot.get().unwrap().queue().get(idx).unwrap().id(), &access_code, &remote_id, &client);
            }}>
            "Previous"
        </button>
        <button on:click={
            let access_code = access_code.clone();
            let remote_id = remote_id.clone();
            let client = client.clone();
            move |_| { publish_move(1, snapshot.get().unwrap().queue().get(1).unwrap().id(), &access_code, &remote_id, &client); }}>
            "Next"
        </button>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/*" view=Player/>
            </Routes>
        </Router>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> });
}
