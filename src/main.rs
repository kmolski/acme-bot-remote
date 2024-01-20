// Copyright (C) 2022-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};

use base64::prelude::*;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::stomp::{StompClient, StompUrl};

mod stomp;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessageType {
    Resume,
    Pause,
    Stop,
}

#[derive(Serialize, Deserialize)]
struct Message {
    op: MessageType,
    code: String,
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
        logging::debug_warn!("Could not send command: {e:?}")
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

    let remote_url = StompUrl::new(url.as_str()).unwrap();
    let exchange = format!("/exchange/acme_bot_remote_update/{remote_id}.{access_code}");
    let client: Arc<Mutex<StompClient>> = Arc::new_cyclic(|weak_ref: &Weak<Mutex<StompClient>>| {
        let weak = weak_ref.clone();
        Mutex::new(StompClient::new(
            &remote_url,
            &login,
            &password,
            Some(move |_| {
                if let Some(arc) = weak.upgrade() {
                    let mut client = arc.lock().expect("lock poisoned");
                    if client.connected() && !client.subscribed() {
                        leptos::logging::log!("SUBBING!");
                        if let Err(e) = client.subscribe(
                            move |_| {
                                leptos::logging::log!("Message received!");
                            },
                            &exchange,
                        ) {
                            leptos::logging::debug_warn!("{e:?}");
                        }
                        if let Err(e) = client.publish("", &exchange) {
                            leptos::logging::debug_warn!("{e:?}");
                        }
                        leptos::logging::log!("DONE!");
                    }
                }
            }),
        ))
    });
    let (tracks, _set_tracks) = create_signal(String::new());
    let _arc = client.clone();
    {
        match client.lock() {
            Ok(c) => c.activate(),
            Err(e) => leptos::logging::debug_warn!("{e:?}"),
        }
    }

    view! {
        <div>{move || tracks.get()}</div>
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
