// Copyright (C) 2023-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(non_snake_case)]

/// Synchronous wrapper for the stompjs library.
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};
use wasm_bindgen::prelude::*;

/// URL for a STOMP-over-WebSocket secure connection.
pub struct StompUrl(Url);

#[derive(Error, Debug, PartialEq)]
pub enum StompUrlError {
    #[error("Invalid URL: {0}")]
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

    #[wasm_bindgen(constructor)]
    fn new(conf: &JsValue) -> Client;

    #[wasm_bindgen(method, setter, structural)]
    fn set_onConnect(this: &Client, callback: &Closure<dyn FnMut(JsValue)>);

    #[wasm_bindgen(method)]
    fn activate(this: &Client);

    #[wasm_bindgen(method, getter)]
    fn connected(this: &Client) -> bool;

    #[wasm_bindgen(method)]
    fn publish(this: &Client, params: &JsValue);

    #[wasm_bindgen(method)]
    fn subscribe(
        this: &Client,
        destination: &JsValue,
        callback: &Closure<dyn FnMut(JsValue)>,
        headers: &JsValue,
    ) -> Subscription;
}

pub struct StompClient(Client, Option<Subscription>);

#[derive(Error, Debug, PartialEq)]
pub enum StompClientError {
    #[error("Not connected")]
    NotConnected,
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

impl StompClient {
    pub fn new(
        url: &StompUrl,
        login: &str,
        passcode: &str,
        on_connect_callback: Option<impl FnMut(JsValue) + 'static>,
    ) -> Self {
        let conf = StompConfig {
            brokerURL: url.0.to_string(),
            connectHeaders: StompHeaders {
                login: login.to_string(),
                passcode: passcode.to_string(),
            },
        };
        let client = Client::new(&JsValue::from_serde(&conf).expect("from_serde always succeeds"));
        if let Some(on_connect) = on_connect_callback {
            let callback = Closure::new(on_connect);
            client.set_onConnect(&callback);
            callback.forget();
        }
        Self(client, None)
    }

    pub fn activate(&self) {
        self.0.activate();
    }

    pub fn connected(&self) -> bool {
        self.0.connected()
    }

    pub fn subscribed(&self) -> bool {
        self.1.is_some()
    }

    pub fn publish(&self, msg: &str, dest: &str) -> Result<(), StompClientError> {
        if !self.connected() {
            return Err(StompClientError::NotConnected);
        }
        let pub_params = IPublishParams {
            destination: dest.to_string(),
            body: msg.to_string(),
        };
        let args = JsValue::from_serde(&pub_params).expect("from_serde always succeeds");
        self.0.publish(&args);
        Ok(())
    }

    pub fn subscribe(
        &mut self,
        callback: impl FnMut(JsValue) + 'static,
        dest: &str,
    ) -> Result<(), StompClientError> {
        if !self.connected() {
            return Err(StompClientError::NotConnected);
        }
        let callback = Closure::new(callback);
        self.1 = Some(
            self.0
                .subscribe(&JsValue::from_str(dest), &callback, &JsValue::null()),
        );
        callback.forget();
        Ok(())
    }
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
    }

    #[test]
    fn given_fragment_when_new_then_return_error() {
        // given
        let url = "wss://example.com/#fragment";

        // when
        let result = StompUrl::new(url);

        // then
        assert!(result.is_err());
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
