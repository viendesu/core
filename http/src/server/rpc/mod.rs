//! JSON-RPC 2.0 endpoint at `POST /rpc`.
//!
//! Every service endpoint is a method named `<module>.<endpoint>` (e.g.
//! `games.get`, `marks.list_tags`) whose `params` are exactly the protocol
//! `Args` of that endpoint. The envelope is JSON-RPC 2.0 in either codec:
//! JSON, or MessagePack carrying a map with the same keys. `Content-Type`
//! selects the request codec, `Accept` the response one (the request codec by
//! default). Batches are not supported; a request without `id` is a
//! notification and gets `204 No Content`. The session token travels in
//! `Authorization: Bearer <token>`.
//!
//! Every dispatched call answers `200 OK`; 4xx statuses mean the body never
//! reached the dispatcher (unsupported media type, body too large).
//!
//! `uploads.finish` streams bytes, so it is `POST /uploads/{id}` with a
//! multipart body instead (see [`upload`]); it answers the same response
//! object with `id: null`.

use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::{State, rejection::BytesRejection},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing,
};
use fastrace::{Span, future::FutureExt as _};

use viendesu_core::service::{Session, SessionMaker as _, SessionOf, authz::Authentication as _};

use crate::{format::Format, server::Types};

use self::{
    envelope::Id,
    methods::{Call, Handler},
};

mod envelope;
mod extract;
mod methods;
mod upload;

/// Error codes of the `error.code` member.
pub mod code {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    /// `data` is the auxiliary error (db, object store, mail, internal).
    pub const INTERNAL_ERROR: i32 = -32603;

    /// The endpoint's own error: `data` is its `Err` value.
    pub const DOMAIN: i32 = -32010;
    pub const UNAUTHENTICATED: i32 = -32011;
    /// The caller's role is too low; `data` is the auxiliary error.
    pub const FORBIDDEN: i32 = -32012;
    /// The session token is unknown or expired.
    pub const INVALID_SESSION: i32 = -32013;
    pub const CAPTCHA: i32 = -32014;
}

struct RpcState<T: Types> {
    service: T::Service,
    methods: HashMap<&'static str, Handler<SessionOf<T::Service>>>,
}

pub fn router<T: Types>(service: T::Service) -> axum::Router {
    let state = Arc::new(RpcState::<T> {
        service: service.clone(),
        methods: methods::table(),
    });

    axum::Router::new()
        .route("/rpc", routing::post(dispatch::<T>).with_state(state))
        .merge(upload::router::<T>(Arc::new(service)))
}

/// A session authenticated by the `Authorization` header, or the encoded
/// failure to answer instead.
async fn session<T: Types>(
    service: &T::Service,
    headers: &HeaderMap,
    call: &Call<'_>,
) -> Result<Session<SessionOf<T::Service>>, Vec<u8>> {
    let mut session = service
        .make_session()
        .await
        .map_err(|aux| call.fail_aux(&aux))?;

    match extract::session_token(headers) {
        Ok(None) => {}
        Ok(Some(token)) => session
            .authz()
            .authenticate(token)
            .await
            .map_err(|aux| call.fail_aux(&aux))?,
        Err(aux) => return Err(call.fail(code::INVALID_REQUEST, &aux.to_string())),
    }

    Ok(session)
}

async fn dispatch<T: Types>(
    State(state): State<Arc<RpcState<T>>>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let Ok(request_format) = extract::request_format(&headers) else {
        return transport_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Content-Type must be application/json or application/msgpack",
        );
    };
    let format = extract::response_format(&headers, request_format);

    let body = match body {
        Ok(body) => body,
        Err(rejection) => return transport_error(rejection.status(), &rejection.body_text()),
    };

    let request = match envelope::parse(request_format, &body) {
        Ok(request) => request,
        Err(rejection) => {
            return reply(
                format,
                envelope::failure_plain(format, &Id::Null, rejection.code, &rejection.message),
            );
        }
    };
    let id = request.id.clone().unwrap_or(Id::Null);

    let Some((&method, &handler)) = state.methods.get_key_value(&*request.method) else {
        if request.id.is_none() {
            return StatusCode::NO_CONTENT.into_response();
        }
        let message = format!("method {:?} not found", request.method);
        return reply(
            format,
            envelope::failure_plain(format, &id, code::METHOD_NOT_FOUND, &message),
        );
    };

    let call = Call {
        format,
        id: &id,
        params: request.params,
    };
    let response = async {
        match session::<T>(&state.service, &headers, &call).await {
            Ok(mut session) => handler(&mut session, call).await,
            Err(failure) => failure,
        }
    }
    .in_span(Span::enter_with_local_parent(method))
    .await;

    if request.id.is_none() {
        return StatusCode::NO_CONTENT.into_response();
    }

    reply(format, response)
}

fn reply(format: Format, body: Vec<u8>) -> Response {
    (
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static(format.mime_type()),
            ),
            (
                HeaderName::from_static("server"),
                HeaderValue::from_static("Kurisu-desu"),
            ),
        ],
        body,
    )
        .into_response()
}

fn transport_error(status: StatusCode, message: &str) -> Response {
    let body = envelope::failure_plain(Format::Json, &Id::Null, code::INVALID_REQUEST, message);
    (status, reply(Format::Json, body)).into_response()
}
