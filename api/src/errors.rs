use aws_sdk_dynamodb::{
    error::{
        BatchGetItemError, DeleteItemError, GetItemError, PutItemError, QueryError, ScanError,
        UpdateItemError,
    },
    model::AttributeValue,
    types::SdkError,
};
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use std::num::{ParseFloatError, ParseIntError};

/// This is our project's private error type. It is a very simple wrapper that
/// has both an optional message to log to CloudWatch (if filled) and a message
/// to send to the end user (not optional).
///
/// This error type can be cast into a Response object, so that you can return
/// it to Axum.
///
/// This file also contains quick and dirty (admittedtly repetitive) casts from
/// various other error types that can arise during execution so that those can
/// be converted into ChatError and returned to Axum.
///
/// For the most part, you can ignore this file, the TL;DR is that this is just
/// a mechanism that allows the app to send a uniform error back to Axum.
pub struct ChatError {
    pub debug: Option<String>,
    pub display: String,
}

impl IntoResponse for ChatError {
    fn into_response(self) -> Response {
        let id = fastrand::u64(u64::MIN..u64::MAX);
        let mut id_string = String::new();
        if let Some(debug) = self.debug {
            println!("FATAL ({}) {}", id, debug);
            id_string.push_str(&format!(" ({})", id));
        }
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::boxed(axum::body::Full::from(format!(
                "{}{}",
                self.display, id_string
            ))))
            .unwrap()
    }
}

impl ChatError {
    pub fn new(debug: Option<String>, display: String) -> Self {
        Self { debug, display }
    }
}

impl From<SdkError<GetItemError>> for ChatError {
    fn from(error: SdkError<GetItemError>) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<SdkError<BatchGetItemError>> for ChatError {
    fn from(error: SdkError<BatchGetItemError>) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<SdkError<UpdateItemError>> for ChatError {
    fn from(error: SdkError<UpdateItemError>) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<SdkError<PutItemError>> for ChatError {
    fn from(error: SdkError<PutItemError>) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<SdkError<DeleteItemError>> for ChatError {
    fn from(error: SdkError<DeleteItemError>) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<SdkError<QueryError>> for ChatError {
    fn from(error: SdkError<QueryError>) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<SdkError<ScanError>> for ChatError {
    fn from(error: SdkError<ScanError>) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<ParseIntError> for ChatError {
    fn from(error: ParseIntError) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<ParseFloatError> for ChatError {
    fn from(error: ParseFloatError) -> Self {
        Self::new(
            Some(format!("{:?}", error)),
            "Internal server error".to_string(),
        )
    }
}

impl From<&AttributeValue> for ChatError {
    fn from(error: &AttributeValue) -> Self {
        Self::new(
            Some(format!("Error unwrapping AttributeValue {:?}", error)),
            "Internal server error".to_string(),
        )
    }
}
