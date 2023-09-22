#![allow(non_snake_case)]

use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};
use wasm_bindgen::prelude::*;

pub struct StompUrl(Url);

#[derive(Error, Debug, PartialEq)]
pub enum StompUrlError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] ParseError),

    #[error("URL must use wss scheme")]
    InvalidScheme,

    #[error("URL cannot have a fragment")]
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
    pub fn new(url: &StompUrl, login: &str, passcode: &str) -> Self {
        let conf = StompConfig {
            brokerURL: url.0.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_invalid_syntax_when_new_then_return_error() {
        let result = StompUrl::new("foobarbaz");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            StompUrlError::InvalidUrl(ParseError::RelativeUrlWithoutBase)
        )
    }

    #[test]
    fn given_invalid_scheme_when_new_then_return_error() {
        let result = StompUrl::new("http://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn given_fragment_when_new_then_return_error() {
        let result = StompUrl::new("wss://example.com/#fragment");
        assert!(result.is_err());
    }

    #[test]
    fn given_valid_url_when_new_then_return_ok() {
        let result = StompUrl::new("wss://example.com").unwrap();
        assert_eq!(result.0.as_str(), "wss://example.com/");
    }
}
