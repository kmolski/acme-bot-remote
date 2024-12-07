// Copyright (C) 2023-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(non_snake_case)]

use gloo_utils::format::JsValueSerdeExt;
use js_sys::Object;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};
use wasm_bindgen::prelude::*;

/// Synchronous wrapper for the stompjs.Client class.
///
/// See https://stomp-js.github.io/api-docs/latest/classes/Client.html for details.
pub struct StompClient {
    client: Client,
    subscription: Option<Subscription>,
    subscription_callback: MessageConsumer,
    #[allow(unused)]
    on_connect_callback: EventConsumer,
}

type MessageConsumer = Closure<dyn FnMut(IMessage)>;
type EventConsumer = Closure<dyn FnMut(JsValue)>;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum StompClientError {
    #[error("not connected")]
    NotConnected,
}

impl StompClient {
    pub fn new<M, C>(
        url: &StompUrl,
        login: &str,
        passcode: &str,
        on_message: M,
        on_connect: C,
    ) -> Self
    where
        M: Fn(&str) + 'static,
        C: Fn(JsValue) + 'static,
    {
        let conf = StompConfig {
            brokerURL: url.0.to_string(),
            connectHeaders: StompHeaders {
                login: login.to_string(),
                passcode: passcode.to_string(),
            },
        };
        let client = Client::new(&JsValue::from_serde(&conf).expect("from_serde always succeeds"));
        let on_connect = Closure::new(on_connect);
        client.set_onConnect(&on_connect);

        Self {
            client,
            subscription: None,
            subscription_callback: Closure::new(move |msg: IMessage| on_message(&msg.body())),
            on_connect_callback: on_connect,
        }
    }

    pub fn connect(&mut self) {
        self.client.activate();
    }

    pub fn connected(&self) -> bool {
        self.client.connected()
    }

    pub fn subscribed(&self) -> bool {
        self.subscription.is_some()
    }

    pub fn publish(&mut self, msg: &str, dest: &str) -> Result<(), StompClientError> {
        if !self.connected() {
            return Err(StompClientError::NotConnected);
        }
        let pub_params = IPublishParams {
            destination: dest.to_string(),
            body: msg.to_string(),
        };
        let args = JsValue::from_serde(&pub_params).expect("from_serde always succeeds");
        self.client.publish(&args);
        Ok(())
    }

    pub fn subscribe(&mut self, dest: &str) -> Result<(), StompClientError> {
        if !self.connected() {
            return Err(StompClientError::NotConnected);
        }
        self.subscription = Some(self.client.subscribe(
            &JsValue::from_str(dest),
            &self.subscription_callback,
            &JsValue::null(),
        ));
        Ok(())
    }
}

impl Drop for StompClient {
    fn drop(&mut self) {
        if self.connected() {
            self.client.deactivate(&JsValue::from(Object::new()));
        }
    }
}

/// URL for a secure STOMP-over-WebSocket connection.
pub struct StompUrl(Url);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum StompUrlError {
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] ParseError),

    #[error("URL must use the WSS scheme")]
    InvalidScheme,

    #[error("URL cannot contain a fragment")]
    HasFragment,
}

impl StompUrl {
    pub fn new(url: &str) -> Result<Self, StompUrlError> {
        let url = Url::parse(url)?;
        if url.scheme() != "wss" {
            Err(StompUrlError::InvalidScheme)
        } else if url.fragment().is_some() {
            Err(StompUrlError::HasFragment)
        } else {
            Ok(Self(url))
        }
    }
}

#[wasm_bindgen(module = "@stomp/stompjs")]
extern "C" {
    type Client;
    type Subscription;
    type IMessage;

    #[wasm_bindgen(constructor)]
    fn new(conf: &JsValue) -> Client;

    #[wasm_bindgen(method, setter, structural)]
    fn set_onConnect(this: &Client, callback: &EventConsumer);

    #[wasm_bindgen(method)]
    fn activate(this: &Client);

    #[wasm_bindgen(method)]
    fn deactivate(this: &Client, options: &JsValue);

    #[wasm_bindgen(method, getter)]
    fn connected(this: &Client) -> bool;

    #[wasm_bindgen(method)]
    fn publish(this: &Client, params: &JsValue);

    #[wasm_bindgen(method)]
    fn subscribe(
        this: &Client,
        destination: &JsValue,
        callback: &MessageConsumer,
        headers: &JsValue,
    ) -> Subscription;

    #[wasm_bindgen(method, getter)]
    fn body(this: &IMessage) -> String;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_invalid_url_returns_error() {
        let url = "foobarbaz";

        let result = StompUrl::new(url);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            StompUrlError::InvalidUrl(ParseError::RelativeUrlWithoutBase)
        )
    }

    #[test]
    fn new_from_invalid_scheme_returns_error() {
        let url = "http://example.com";

        let result = StompUrl::new(url);

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), StompUrlError::InvalidScheme)
    }

    #[test]
    fn new_from_fragment_url_returns_error() {
        let url = "wss://example.com/#fragment";

        let result = StompUrl::new(url);

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), StompUrlError::HasFragment)
    }

    #[test]
    fn new_from_valid_url_returns_ok() {
        let url = "wss://example.com";

        let result = StompUrl::new(url).unwrap();

        assert_eq!(result.0.as_str(), "wss://example.com/");
    }
}
