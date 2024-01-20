// Copyright (C) 2023-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(non_snake_case)]

use gloo_utils::format::JsValueSerdeExt;
use js_sys::Object;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};
use wasm_bindgen::prelude::*;

use crate::player::{PubSubClient, PubSubError};

/// URL for a STOMP-over-WebSocket secure connection.
pub struct StompUrl(Url);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum StompUrlError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] ParseError),

    #[error("URL must use the WSS scheme")]
    InvalidScheme,

    #[error("URL cannot contain a fragment")]
    HasFragment,
}

impl StompUrl {
    /// Parse a STOMP-over-WebSocket URL from a string.
    ///
    /// # Arguments
    ///
    /// * `url`: &str - STOMP-over-WebSocket URL to parse
    ///
    /// returns: Result<StompUrl, StompUrlError>
    ///
    /// # Errors
    ///
    /// * `StompUrlError::InvalidUrl` - invalid URL syntax
    /// * `StompUrlError::InvalidScheme` - invalid URL scheme (must be WSS)
    /// * `StompUrlError::HasFragment` - invalid URL fragment (must be empty)
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

type EventConsumer = Closure<dyn FnMut(JsValue)>;
type MessageConsumer = Closure<dyn FnMut(IMessage)>;

/// Synchronous wrapper for the stompjs.Client class.
///
/// See https://stomp-js.github.io/api-docs/latest/classes/Client.html for details.
pub struct StompClient {
    client: Client,
    subscription: Option<Subscription>,
    subscription_callback: Option<MessageConsumer>,
    #[allow(unused)]
    on_connect_callback: Option<EventConsumer>,
}

impl StompClient {
    /// Create a new STOMP-over-WebSocket client.
    ///
    /// # Arguments
    ///
    /// * `url`: &StompUrl - URL of the message broker
    /// * `login`: &str - user identifier used for authentication
    /// * `passcode`: &str - password used for authentication
    /// * `on_connect`: Option<C> - callback invoked on a successful connection
    ///
    /// returns: StompClient
    pub fn new<C>(url: &StompUrl, login: &str, passcode: &str, on_connect: Option<C>) -> Self
    where
        C: FnMut(JsValue) + 'static,
    {
        let conf = StompConfig {
            brokerURL: url.0.to_string(),
            connectHeaders: StompHeaders {
                login: login.to_string(),
                passcode: passcode.to_string(),
            },
        };
        let on_connect_callback = on_connect.map(Closure::new);
        let client = Client::new(&JsValue::from_serde(&conf).expect("from_serde always succeeds"));
        if let Some(ref callback) = on_connect_callback {
            client.set_onConnect(callback);
        }

        Self {
            client,
            subscription: None,
            subscription_callback: None,
            on_connect_callback,
        }
    }
}

impl PubSubClient for StompClient {
    /// Start connecting to the message broker.
    fn activate(&self) {
        self.client.activate();
    }

    /// Check if the client is connected to the message broker.
    fn connected(&self) -> bool {
        self.client.connected()
    }

    /// Check if the client is subscribed to a STOMP destination.
    fn subscribed(&self) -> bool {
        self.subscription.is_some()
    }

    /// Publish a message to the given STOMP destination.
    ///
    /// # Arguments
    ///
    /// * `msg`: &str - message content
    /// * `dest`: &str - STOMP destination
    ///
    /// returns: Result<(), PubSubError>
    ///
    /// # Errors
    ///
    /// * `PubSubError::NotConnected` - client is not connected to the message broker
    fn publish(&self, msg: &str, dest: &str) -> Result<(), PubSubError> {
        if !self.connected() {
            return Err(PubSubError::NotConnected);
        }
        let pub_params = IPublishParams {
            destination: dest.to_string(),
            body: msg.to_string(),
        };
        let args = JsValue::from_serde(&pub_params).expect("from_serde always succeeds");
        self.client.publish(&args);
        Ok(())
    }

    /// Subscribe to a STOMP destination.
    ///
    /// # Arguments
    ///
    /// * `callback`: C - callback invoked when a message is received
    /// * `dest`: &str - STOMP destination
    ///
    /// returns: Result<(), PubSubError>
    ///
    /// # Errors
    ///
    /// * `PubSubError::NotConnected` - client is not connected to the message broker
    fn subscribe<C>(&mut self, callback: C, dest: &str) -> Result<(), PubSubError>
    where
        C: Fn(String) + 'static,
    {
        if !self.connected() {
            return Err(PubSubError::NotConnected);
        }
        self.subscription_callback = Some(Closure::new(move |msg: IMessage| callback(msg.body())));
        self.subscription = Some(self.client.subscribe(
            &JsValue::from_str(dest),
            self.subscription_callback.as_ref().unwrap(),
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
    fn given_invalid_syntax_when_new_then_return_error() {
        // given
        let url = "foobarbaz";

        // when
        let result = StompUrl::new(url);

        // then
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            StompUrlError::InvalidUrl(ParseError::RelativeUrlWithoutBase)
        )
    }

    #[test]
    fn given_invalid_scheme_when_new_then_return_error() {
        // given
        let url = "http://example.com";

        // when
        let result = StompUrl::new(url);

        // then
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), StompUrlError::InvalidScheme)
    }

    #[test]
    fn given_fragment_when_new_then_return_error() {
        // given
        let url = "wss://example.com/#fragment";

        // when
        let result = StompUrl::new(url);

        // then
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), StompUrlError::HasFragment)
    }

    #[test]
    fn given_valid_url_when_new_then_return_ok() {
        // given
        let url = "wss://example.com";

        // when
        let result = StompUrl::new(url).unwrap();

        // then
        assert_eq!(result.0.as_str(), "wss://example.com/");
    }
}
