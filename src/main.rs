// Copyright (C) 2023-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};

use base64::prelude::*;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::player::{MusicPlayerState, PlayerSnapshot, PubSubClient, TrackSnapshot};
use crate::remote_api::PlayerModel;
use crate::stomp::{StompClient, StompUrl};

mod player;
mod remote_api;
mod stomp;

const ICON_FRAME_SMALL: &str = "8 8 22 22";
const ICON_FRAME_LARGE: &str = "0 0 38 38";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessageType {
    Resume,
    Pause,
    Stop,
    Clear,
    Move,
    Loop,
    Volume,
    Remove,
    Skip,
    Prev,
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

#[derive(Serialize, Deserialize)]
struct VolumeMessage {
    op: MessageType,
    code: String,
    value: u8,
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
    op: MessageType,
    access_code: &str,
    remote_id: &str,
    client: &Arc<Mutex<StompClient>>,
) {
    let command = MoveMessage {
        op,
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

fn publish_volume(
    volume: u8,
    access_code: &str,
    remote_id: &str,
    client: &Arc<Mutex<StompClient>>,
) {
    let command = VolumeMessage {
        op: MessageType::Volume,
        code: access_code.to_string(),
        value: volume,
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

use std::time::Duration;

fn format_duration(duration: &Duration) -> String {
    let mut formatted = String::new();
    let mut sec = duration.as_secs();
    let mut min = sec / 60;
    let hrs = min / 60;
    min %= 60;
    sec %= 60;
    if hrs > 0 {
        formatted.push_str(&format!("{hrs}:{min:02}:"));
    } else {
        formatted.push_str(&format!("{min}:"));
    }
    formatted.push_str(&format!("{sec:02}"));
    formatted
}

#[component]
fn DeleteIcon(frame: &'static str) -> impl IntoView {
    view! {
        <svg class="svg-icon" aria-hidden="true" viewBox={ frame }>
            <path d="M 14.75 13 C 14.75 11.62 16.65 10.5 19 10.5 C 21.35 10.5 23.25 11.62 23.25 13" fill="none" stroke="#000000" stroke-width="1" stroke-miterlimit="10"/>
            <path d="M 12.5 28 L 14 14.75 L 24 14.75 L 25.5 28 Z" fill="#000000" transform="rotate(-180,19,21.38)"/>
            <rect x="12" y="12.5" width="14" height="1" rx="0.06" ry="0.06" fill="#000000" stroke="#000000"/>
        </svg>
    }
}

#[component]
fn PreviousIcon(frame: &'static str) -> impl IntoView {
    view! {
        <svg class="svg-icon" aria-hidden="true" viewBox={ frame }>
            <rect x="13" y="9" width="4" height="20" fill="#000000"/>
            <path d="M 25 9 L 13 19 L 25 29 Z" fill="#000000"/>
        </svg>
    }
}

#[component]
fn NextIcon(frame: &'static str) -> impl IntoView {
    view! {
        <svg class="svg-icon" aria-hidden="true" viewBox={ frame }>
            <rect x="21" y="9" width="4" height="20" fill="#000000"/>
            <path d="M 13 9 L 25 19 L 13 29 Z" fill="#000000"/>
        </svg>
    }
}

#[component]
fn PlayIcon(frame: &'static str) -> impl IntoView {
    view! {
        <svg class="svg-icon" aria-hidden="true" viewBox={ frame }>
            <path d="M 13 9 L 28 19 L 13 29 Z" fill="#000000"/>
        </svg>
    }
}

#[component]
fn PauseIcon(frame: &'static str) -> impl IntoView {
    view! {
        <svg class="svg-icon" aria-hidden="true" viewBox={ frame }>
            <rect x="13" y="9" width="4" height="20" fill="#000000"/>
            <rect x="21" y="9" width="4" height="20" fill="#000000"/>
        </svg>
    }
}

#[component]
fn LoopIcon(frame: &'static str) -> impl IntoView {
    view! {
        <svg class="svg-icon" aria-hidden="true" viewBox={ frame }>
            <path d="M 20.15 26.5 L 17.15 28 L 17.15 25 Z" fill="#000000" stroke="#000000" stroke-width="3" stroke-miterlimit="10"/>
            <path d="M 26.5 26.5 L 26.5 11.5 L 11.5 11.5 L 11.5 26.5 L 17.15 26.5" fill="none" stroke="#000000" stroke-width="3"/>
        </svg>
    }
}

#[component]
fn VolumeIcon(value: Signal<u8>) -> impl IntoView {
    view! {
        <svg width="1.3em" height="1.3em" aria-hidden="true" viewBox="0 0 22 20">
            <path d="M -3 16 L 2 4 L 9 4 L 14 16 Z" fill="#000000" stroke="none" transform="rotate(-90,5.5,10)"/>
            <path d="M 13 9.2 L 16.2 9.2 L 16.2 6 L 17.8 6 L 17.8 9.2 L 21 9.2 L 21 10.8 L 17.8 10.8 L 17.8 14 L 16.2 14 L 16.2 10.8 L 13 10.8 Z" transform="rotate(-45,17,10)"
                fill=move || { if value.get() == 0 { "#000000" } else { "none" }}/>
            <path d="M 6.68 15.26 C 5 14.04 4 12.08 4 10 C 4 7.92 5 5.96 6.68 4.74" fill="none" stroke-width="2" stroke-miterlimit="10" transform="rotate(-180,10.5,10)"
                stroke=move || { if value.get() > 0 { "#000000" } else { "none" }}/>
            <path d="M 5.71 17.69 C 3.38 15.9 2 13.04 2 10 C 2 6.96 3.38 4.1 5.71 2.31" fill="none" stroke-width="2" stroke-miterlimit="10" transform="rotate(-180,11,10)"
                stroke=move || { if value.get() > 50 { "#000000" } else { "none" }}/>
        </svg>
    }
}

#[component]
fn TrackCard<T: TrackSnapshot + Clone + 'static>(track: MaybeSignal<T>) -> impl IntoView {
    view! {
        <div style="display: flex; white-space: nowrap">
            <img src={ track.get().thumbnail().map(|s| s.to_string()) } style="height: 2.5rem; width: 2.5rem; border-radius: 0.25rem; object-fit: cover; margin-right: 0.5rem"/>
            <div style="display: flex; flex-direction: column; padding-right: 2rem">
                <a href={ track.get().webpage_url().to_string() } target="_blank" style="font-weight: 600">{ track.get().title().to_string() }</a>
                <a href={ track.get().uploader_url().map(|s| s.to_string()) } target="_blank">{ track.get().uploader().to_string() }</a>
            </div>
        </div>
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

    let (snapshot, set_snapshot) = create_signal::<PlayerModel>(Default::default());
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
                    if client.connected() {
                        logging::log!("SUBBING!");
                        if let Err(e) = client.subscribe(
                            move |m| {
                                logging::log!("Message received: {}", m);
                                match serde_json::from_str(&m) {
                                    Ok(p) => set_snapshot.set(p),
                                    Err(e) => logging::error!("Invalid snapshot: {}", e),
                                }
                            },
                            &exchange,
                        ) {
                            leptos::logging::warn!("{e:?}");
                        }
                        if let Err(e) = client.publish("", &exchange) {
                            leptos::logging::warn!("{e:?}");
                        }
                        logging::log!("connected: {}", client.subscribed());
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
    let s = snapshot;
    view! {
        <div class="container">
            <header class="header">
                <span>Next up</span>
            </header>
            <main class="track-list">
                <ol>
                    <For each=move || snapshot.get().queue().to_vec()
                         key=move |entry| entry.id().to_string()
                         children={
                            let access_code = access_code.clone();
                            let remote_id = remote_id.clone();
                            let client = client.clone();
                            move |entry| {
                                let access_code = access_code.clone();
                                let remote_id = remote_id.clone();
                                let client = client.clone();
                                view! {
                                    <li>
                                        <div class="track" style="width: 100%; align-items: center; border-radius: 0.25rem; padding: 0.25rem; justify-content: space-between; contain: inline-size">
                                            <div style="display: inline-flex; mask-image: linear-gradient(0.75turn, transparent, #fff4e0 2rem); overflow: hidden">
                                                <TrackCard track=entry.clone().into()/>
                                            </div>
                                            <div style="display: inline-flex">
                                                <span style="margin-right: 0.5rem">{ format_duration(&entry.duration()) }</span>
                                                <button class="btn-inline" on:click={
                                                        let access_code = access_code.clone();
                                                        let remote_id = remote_id.clone();
                                                        let client = client.clone();
                                                        let entry = entry.clone();
                                                        move |_| {
                                                        let idx = snapshot.get().queue().iter().position(|e| e.id() == entry.id()).unwrap();
                                                        publish_move(idx, entry.id(), MessageType::Move, &access_code, &remote_id, &client); }}>
                                                    <PlayIcon frame=ICON_FRAME_SMALL/>
                                                    <span class="screenreader-only">Play</span>
                                                </button>
                                                <button class="btn-inline" on:click={
                                                        let access_code = access_code.clone();
                                                        let remote_id = remote_id.clone();
                                                        let client = client.clone();
                                                        let entry = entry.clone();
                                                        move |_| {
                                                        let idx = snapshot.get().queue().iter().position(|e| e.id() == entry.id()).unwrap();
                                                        publish_move(idx, entry.id(), MessageType::Remove, &access_code, &remote_id, &client); }}>
                                                    <DeleteIcon frame=ICON_FRAME_SMALL/>
                                                    <span class="screenreader-only">Remove</span>
                                                </button>
                                            </div>
                                        </div>
                                    </li>
                                }
                         }}
                    />
                </ol>
            </main>
            <footer class="footer">
                <div class="track" style="mask-image: linear-gradient(0.75turn, transparent, #fff4e0 2rem); contain: inline-size; overflow: hidden">
                    {move || {
                        if s.get().current.is_some() {
                            Some(
                                view! {
                                    <TrackCard track=MaybeSignal::derive(move || { let snap = s.get(); snap.current.unwrap().clone() })/>
                                }
                            )
                        } else {
                            None
                        }
                    }}
                </div>
                <div class="controls">
                    <button class="btn-round" on:click={
                        let access_code = access_code.clone();
                        let remote_id = remote_id.clone();
                        let client = client.clone();
                        move |_| { publish(MessageType::Clear, &access_code, &remote_id, &client); }}>
                        <DeleteIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Clear queue</span>
                    </button>
                    <button class="btn-round" on:click={
                        let access_code = access_code.clone();
                        let remote_id = remote_id.clone();
                        let client = client.clone();
                        move |_| {
                            publish(MessageType::Prev, &access_code, &remote_id, &client);
                        }}>
                        <PreviousIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Previous track</span>
                    </button>
                    <button class="btn-round" on:click={
                        let access_code = access_code.clone();
                        let remote_id = remote_id.clone();
                        let client = client.clone();
                        move |_| {
                            if s.get().state() == MusicPlayerState::Playing {
                                publish(MessageType::Pause, &access_code, &remote_id, &client);
                            } else {
                                publish(MessageType::Resume, &access_code, &remote_id, &client);
                            }
                        }}>
                        <Show when=move || { s.get().state() == MusicPlayerState::Playing }
                              fallback=|| view! { <PlayIcon frame=ICON_FRAME_LARGE/> <span class="screenreader-only">Resume</span> }>
                            <PauseIcon frame=ICON_FRAME_LARGE/>
                            <span class="screenreader-only">Pause</span>
                        </Show>
                    </button>
                    <button class="btn-round" on:click={
                        let access_code = access_code.clone();
                        let remote_id = remote_id.clone();
                        let client = client.clone();
                        move |_| {
                            publish(MessageType::Skip, &access_code, &remote_id, &client);
                        }}>
                        <NextIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Next track</span>
                    </button>
                    <label class="btn-round">
                        <input type="checkbox"
                            prop:checked=move || { snapshot.get().loop_enabled() }
                            on:change={
                                let access_code = access_code.clone();
                                let remote_id = remote_id.clone();
                                let client = client.clone();
                                move |e| { publish_loop(event_target_checked(&e), &access_code, &remote_id, &client); }}/>
                        <LoopIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Loop</span>
                    </label>
                </div>
                <label class="volume-widget">
                    <VolumeIcon value=Signal::derive(move || s.get().volume())/>
                    <span class="screenreader-only">Volume</span>
                    <input type="range" id="volume" min="0" max="100" step="1"
                        prop:value=move || { snapshot.get().volume() }
                        on:change={
                            let access_code = access_code.clone();
                            let remote_id = remote_id.clone();
                            let client = client.clone();
                            move |e| { publish_volume(event_target_value(&e).parse().unwrap(), &access_code, &remote_id, &client); }}/>
                </label>
            </footer>
        </div>
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
