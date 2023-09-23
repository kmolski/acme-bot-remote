use std::rc::Rc;

use base64::engine;
use base64::prelude::*;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::stomp::{StompClient, StompUrl};

mod remote_api;
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

#[component]
fn Player() -> impl IntoView {
    let query_params = use_query_map().get_untracked();
    let access_code = Rc::new(query_params.get("ac").unwrap().to_string());
    let remote_id = Rc::new(query_params.get("rid").unwrap().to_string());
    let rcs_bytes = engine::general_purpose::URL_SAFE_NO_PAD
        .decode(query_params.get("rcs").unwrap())
        .unwrap();
    let remote_server = String::from_utf8(rcs_bytes).unwrap();

    let mut url = Url::parse(&remote_server).unwrap();
    let login = url.username().to_string();
    let password = url.password().unwrap().to_string();
    url.set_username("").unwrap();
    url.set_password(None).unwrap();

    let remote_url = StompUrl::new(url.as_str()).unwrap();
    let client = Rc::new(StompClient::new(&remote_url, &login, &password));
    client.activate();

    fn publish(op: MessageType, access_code: &str, remote_id: &str, client: &StompClient) {
        let command = Message {
            op,
            code: access_code.to_string(),
        };
        let msg = serde_json::to_string(&command).unwrap();
        client
            .publish(&msg, &format!("/exchange/acme_bot_remote/{}", remote_id))
            .expect("TODO: panic message");
    }

    view! {
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
            move |_| { publish(MessageType::Stop, &access_code, &remote_id, &client); }}>
            "Stop"
        </button>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Player/>
            </Routes>
        </Router>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> });
}
