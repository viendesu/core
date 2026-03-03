use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
};
use serde::{Deserialize, Serialize};

use crate::format::{DumpParams, Format};
use crate::requests::{IsResponse, status_code::HasStatusCode};

macro_rules! header {
    ($name:expr => $value:expr) => {
        (
            axum::http::HeaderName::from_static($name),
            axum::http::HeaderValue::from_static($value),
        )
    };
}

#[track_caller]
pub fn err<E: IsResponse + fmt::Display>(format: Format, error: E) -> AxumResponse {
    #[derive(Serialize, Deserialize)]
    struct Failure<E> {
        pub error: E,
        pub description: String,
    }

    impl<T: HasStatusCode> HasStatusCode for Failure<T> {
        fn status_code(&self) -> StatusCode {
            self.error.status_code()
        }
    }

    respond(
        format,
        Failure {
            description: error.to_string(),
            error,
        },
    )
}

#[track_caller]
pub fn ok<O: IsResponse>(format: Format, ok: O) -> AxumResponse {
    #[derive(Serialize, Deserialize)]
    struct Success<T> {
        ok: T,
    }

    impl<T: HasStatusCode> HasStatusCode for Success<T> {
        fn status_code(&self) -> StatusCode {
            self.ok.status_code()
        }
    }

    respond(format, Success { ok })
}

#[track_caller]
pub fn respond<O: IsResponse>(format: Format, res: O) -> AxumResponse {
    let status_code = res.status_code();
    let headers = [
        header!("server" => "Kurisu-desu"),
        header!("content-type" => format.mime_type()),
    ];

    let mut body = Vec::with_capacity(128);
    let dump_params = DumpParams { pretty: false };
    format.dump(dump_params, &res, &mut body);

    (status_code, headers, body).into_response()
}
