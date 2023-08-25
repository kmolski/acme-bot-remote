mod stomp;

use crate::stomp::{StompClient, RABBITMQ_PASS, RABBITMQ_URI, RABBITMQ_USER};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use yew::prelude::*;

const RABBITMQ_DEST: &str = "/exchange/acme_bot_remote";

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

#[function_component(App)]
pub fn app() -> Html {
    let client = Arc::new(StompClient::new(RABBITMQ_URI, RABBITMQ_USER, RABBITMQ_PASS));
    client.activate();
    let on_resume = {
        let client = client.clone();
        move |_| {
            client.publish(
                &Message {
                    op: MessageType::Resume,
                    code: "100000".to_string(),
                },
                RABBITMQ_DEST,
            )
        }
    };
    let on_pause = {
        let client = client.clone();
        move |_| {
            client.publish(
                &Message {
                    op: MessageType::Pause,
                    code: "100000".to_string(),
                },
                RABBITMQ_DEST,
            )
        }
    };
    let on_stop = {
        let client = client.clone();
        move |_| {
            client.publish(
                &Message {
                    op: MessageType::Stop,
                    code: "100000".to_string(),
                },
                RABBITMQ_DEST,
            )
        }
    };

    html! {
        <div>
            <label for="accessCode">{ "Access code:" }</label>
            <input id="accessCode" type="number" minlength="1" required=true/>

            <button onclick={on_resume}>{ "Resume" }</button>
            <button onclick={on_pause}>{ "Pause" }</button>
            <button onclick={on_stop}>{ "Stop" }</button>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
