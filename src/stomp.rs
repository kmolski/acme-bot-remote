#![allow(non_snake_case)]

use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub const RABBITMQ_URI: &str = env!("RABBITMQ_URI");
pub const RABBITMQ_USER: &str = env!("RABBITMQ_USER");
pub const RABBITMQ_PASS: &str = env!("RABBITMQ_PASS");

#[wasm_bindgen(module = "@stomp/stompjs")]
extern "C" {
    type Client;

    #[wasm_bindgen(constructor)]
    fn new(conf: &JsValue) -> Client;

    #[wasm_bindgen(method)]
    fn activate(this: &Client);

    #[wasm_bindgen(method)]
    fn publish(this: &Client, params: &JsValue);
}

#[derive(Serialize, Deserialize)]
struct StompHeaders {
    login: String,
    passcode: String,
}

#[derive(Serialize, Deserialize)]
struct StompConfig {
    brokerURL: String,
    connectHeaders: StompHeaders,
}

#[derive(Serialize, Deserialize)]
struct IPublishParams {
    destination: String,
    body: String,
}

pub struct StompClient(Client);

impl StompClient {
    pub fn new(url: &str, login: &str, passcode: &str) -> Self {
        let conf = StompConfig {
            brokerURL: url.to_string(),
            connectHeaders: StompHeaders {
                login: login.to_string(),
                passcode: passcode.to_string(),
            },
        };
        Self(Client::new(&JsValue::from_serde(&conf).unwrap()))
    }

    pub fn activate(&self) {
        self.0.activate();
    }

    pub fn publish<T: ?Sized + Serialize>(&self, msg: &T, dest: &str) {
        let params = IPublishParams {
            destination: dest.to_string(),
            body: serde_json::to_string(&msg).unwrap(),
        };
        self.0.publish(&JsValue::from_serde(&params).unwrap());
    }
}
