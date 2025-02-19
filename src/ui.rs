// Copyright (C) 2023-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Duration;

use base64::prelude::*;
use leptos::*;
use leptos_router::use_query_map;
use url::Url;

use crate::player::{MusicPlayerState, Player, PlayerSnapshot, TrackSnapshot};
use crate::remote_api::{PlayerModel, RemotePlayer, StompUrl};

const ICON_FRAME_SMALL: &str = "8 8 22 22";
const ICON_FRAME_LARGE: &str = "0 0 38 38";

const COPYRIGHT_INFO: &str = "\
acme-bot-remote
Copyright (C) 2023-2024  Krzysztof Molski

This program is free software: you can redistribute it and/or modify it under the terms
of the GNU Affero General Public License as published by the Free Software Foundation,
either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE.
See the GNU Affero General Public License for more details.
";

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
fn InfoIcon(frame: &'static str) -> impl IntoView {
    view! {
        <svg class="svg-icon" aria-hidden="true" viewBox={ frame }>
            <rect x="17" y="9" width="4" height="4" fill="#bfb7a8"/>
            <rect x="17" y="15" width="4" height="14" fill="#bfb7a8"/>
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
fn TrackCard(track: MaybeSignal<impl TrackSnapshot + 'static>) -> impl IntoView {
    view! {
        <div class="track-card">
            <img src={let t = track.clone(); move || t.get().thumbnail().map(|s| s.to_string())} class="track-thumbnail"/>
            <div class="track-card-labels">
                <a href={let t = track.clone(); move || t.get().webpage_url().to_string()} target="_blank" class="track-title">{let t = track.clone(); move || t.get().title().to_string()}</a>
                <a href={let t = track.clone(); move || t.get().uploader_url().map(|s| s.to_string())} target="_blank">{let t = track.clone(); move || t.get().uploader().to_string()}</a>
            </div>
        </div>
    }
}

#[component]
pub fn Player() -> impl IntoView {
    let query_params = use_query_map().get_untracked();
    let access_code = query_params.get("ac").unwrap().parse::<i64>().unwrap();
    let remote_id = query_params.get("rid").unwrap().to_string();
    let rcs_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(query_params.get("rcs").unwrap())
        .unwrap();
    let remote_server = String::from_utf8(rcs_bytes).unwrap();

    let mut url = Url::parse(&remote_server).unwrap();
    let login = url.username().to_string();
    let password = url.password().unwrap().to_string();
    url.set_username("").unwrap();
    url.set_password(None).unwrap();
    url.set_fragment(None);

    let (snapshot, set_snapshot) = create_signal::<PlayerModel>(Default::default());
    let remote_url = StompUrl::new(url.as_str()).unwrap();
    let client = RemotePlayer::new(
        remote_url,
        &login,
        &password,
        &remote_id,
        access_code,
        move |m| {
            logging::log!("Message received: {}", m);
            match serde_json::from_str(m) {
                Ok(p) => set_snapshot.set(p),
                Err(e) => logging::error!("Invalid snapshot: {}", e),
            }
        },
    );
    let client2 = client.clone();
    view! {
        <div class="container">
            <header class="header">
                <span>Next up</span>
                <button class="btn-inline" popovertarget="copyright-dialog">
                    <InfoIcon frame=ICON_FRAME_SMALL/>
                    <span class="screenreader-only">Show copyright info</span>
                </button>
            </header>
            <main class="track-list">
                <ol>
                    <For each=move || snapshot.get().queue().to_vec()
                         key=move |entry| entry.id().to_string()
                         let: entry>
                        <li>
                            <div class="track">
                                <TrackCard track=MaybeSignal::Static(entry.clone())/>
                                <div class="track-controls">
                                    <span class="track-duration">{ format_duration(&entry.duration()) }</span>
                                    <button class="btn-inline" on:click={
                                            let entry = entry.clone();
                                            let client = client2.clone();
                                            move |_| {
                                            let idx = snapshot.get().queue().iter().position(|e| e.id() == entry.id()).unwrap();
                                            client.move_to(idx, entry.id()).unwrap(); }}>
                                        <PlayIcon frame=ICON_FRAME_SMALL/>
                                        <span class="screenreader-only">Play</span>
                                    </button>
                                    <button class="btn-inline" on:click={
                                            let entry = entry.clone();
                                            let client = client2.clone();
                                            move |_| {
                                            let idx = snapshot.get().queue().iter().position(|e| e.id() == entry.id()).unwrap();
                                            client.remove(idx, entry.id()).unwrap(); }}>
                                        <DeleteIcon frame=ICON_FRAME_SMALL/>
                                        <span class="screenreader-only">Remove</span>
                                    </button>
                                </div>
                            </div>
                        </li>
                    </For>
                </ol>
            </main>
            <footer class="footer">
                <div class="track">
                    <Show when=move || { snapshot.get().current.is_some() }>
                        <TrackCard track=MaybeSignal::derive(move || { snapshot.get().current.unwrap() })/>
                    </Show>
                </div>
                <div class="controls">
                    <button class="btn-round" on:click={
                        let client = client.clone();
                        move |_| { client.clear().unwrap(); }}>
                        <DeleteIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Clear queue</span>
                    </button>
                    <button class="btn-round" on:click={
                        let client = client.clone();
                        move |_| { client.prev().unwrap(); }}>
                        <PreviousIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Previous track</span>
                    </button>
                    <button class="btn-round" on:click={
                        let client = client.clone();
                        move |_| {
                            if snapshot.get().state() == MusicPlayerState::Playing {
                                client.pause().unwrap();
                            } else {
                                client.resume().unwrap();
                            }
                        }}>
                        <Show when=move || { snapshot.get().state() == MusicPlayerState::Playing }
                              fallback=|| view! { <PlayIcon frame=ICON_FRAME_LARGE/> <span class="screenreader-only">Resume</span> }>
                            <PauseIcon frame=ICON_FRAME_LARGE/>
                            <span class="screenreader-only">Pause</span>
                        </Show>
                    </button>
                    <button class="btn-round" on:click={
                        let client = client.clone();
                        move |_| { client.skip().unwrap(); }}>
                        <NextIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Next track</span>
                    </button>
                    <label class="btn-round">
                        <input type="checkbox"
                            prop:checked=move || { snapshot.get().loop_enabled() }
                            on:change={
                                let client = client.clone();
                                move |e| { client.set_loop(event_target_checked(&e)).unwrap(); }}/>
                        <LoopIcon frame=ICON_FRAME_LARGE/>
                        <span class="screenreader-only">Loop</span>
                    </label>
                </div>
                <label class="volume-widget">
                    <VolumeIcon value=Signal::derive(move || snapshot.get().volume())/>
                    <span class="screenreader-only">Volume</span>
                    <input type="range" id="volume" min="0" max="100" step="1"
                        prop:value=move || { snapshot.get().volume() }
                        on:change={
                            let client = client.clone();
                            move |e| { client.set_volume(event_target_value(&e).parse().unwrap()).unwrap(); }}/>
                </label>
            </footer>
            <dialog id="copyright-dialog" class="copyright-dialog" popover>
                <pre>{ COPYRIGHT_INFO }</pre>
                <p><a href="https://github.com/kmolski/acme-bot-remote" target="_blank">Show source code</a></p>
                <p><a href="./license_info.html" target="_blank">Show OSS licenses</a></p>
                <button popovertarget="copyright-dialog">Close</button>
            </dialog>
        </div>
    }
}
