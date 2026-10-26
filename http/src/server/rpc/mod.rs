//! JSON-RPC 2.0 endpoint at `POST /rpc`.
//!
//! Every service endpoint is a method named `<module>.<endpoint>` (e.g.
//! `games.get`, `tags.list`) whose `params` are exactly the protocol
//! `Args` of that endpoint. The envelope is JSON-RPC 2.0 in either codec:
//! JSON, or MessagePack carrying a map with the same keys. `Content-Type`
//! selects the request codec, `Accept` the response one (the request codec by
//! default). A request without `id` is a notification and gets no response;
//! a body with nothing to answer gets `204 No Content`. The session token
//! travels in `Authorization: Bearer <token>`.
//!
//! A batch (an array of requests) of at most [`config::Rpc::max_batch`]
//! elements runs sequentially in one session and is answered by an array in
//! request order; an empty or oversized batch is rejected as a whole, as is
//! one without a valid session token under [`config::Rpc::batch_requires_auth`].
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
use viendesu_protocol::errors::Aux;

use crate::{
    format::Format,
    server::{Types, config},
};

use self::{
    envelope::{Id, Message, Params, Request},
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
    max_batch: usize,
    batch_requires_auth: bool,
}

pub fn router<T: Types>(service: T::Service, config: &config::Rpc) -> axum::Router {
    let state = Arc::new(RpcState::<T> {
        service: service.clone(),
        methods: methods::table(),
        max_batch: config.max_batch.get(),
        batch_requires_auth: config.batch_requires_auth,
    });

    axum::Router::new()
        .route("/rpc", routing::post(dispatch::<T>).with_state(state))
        .merge(upload::router::<T>(Arc::new(service)))
}

type SessionResult<T> = Result<Session<SessionOf<<T as Types>::Service>>, SessionFailure>;

/// Why no session could be made; answered to every call that needs one.
enum SessionFailure {
    Service(Aux),
    Header(Aux),
}

impl SessionFailure {
    fn answer(&self, call: &mut Call<'_>) {
        match self {
            Self::Service(aux) => call.fail_aux(aux),
            Self::Header(aux) => {
                if call.id.is_some() {
                    call.fail(code::INVALID_REQUEST, &aux.to_string());
                }
            }
        }
    }
}

/// A session authenticated by the `Authorization` header.
async fn session<T: Types>(service: &T::Service, headers: &HeaderMap) -> SessionResult<T> {
    let mut session = service
        .make_session()
        .await
        .map_err(SessionFailure::Service)?;

    match extract::session_token(headers).map_err(SessionFailure::Header)? {
        None => {}
        Some(token) => session
            .authz()
            .authenticate(token)
            .await
            .map_err(SessionFailure::Service)?,
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

    let message = match envelope::parse(request_format, &body, state.max_batch) {
        Ok(message) => message,
        Err(rejection) => return reply(format, rejection.answer(format)),
    };

    let mut session = None;
    let mut body = Vec::new();
    match message {
        Message::Single(request) => {
            run(&state, &headers, format, request, &mut session, &mut body).await;
        }
        Message::Batch(requests) => {
            if state.batch_requires_auth {
                if let Err(failure) =
                    authenticate::<T>(&state, &headers, format, &mut session).await
                {
                    return reply(format, failure);
                }
            }

            let responds = |request: &Result<Request<'_>, envelope::Rejection>| {
                request
                    .as_ref()
                    .map_or(true, |request| request.id.is_some())
            };
            let count = requests.iter().filter(|request| responds(request)).count();
            if count != 0 {
                match format {
                    Format::Json => body.push(b'['),
                    Format::Msgpack => {
                        let count = u32::try_from(count).expect("batch length fits u32");
                        rmp::encode::write_array_len(&mut body, count)
                            .expect("writing to a Vec cannot fail");
                    }
                }
            }

            let mut remaining = count;
            for request in requests {
                let responds = responds(&request);
                match request {
                    Ok(request) => {
                        run(&state, &headers, format, request, &mut session, &mut body).await;
                    }
                    Err(rejection) => rejection.answer_into(format, &mut body),
                }
                if responds {
                    remaining -= 1;
                    if remaining != 0 && format == Format::Json {
                        body.push(b',');
                    }
                }
            }
            if count != 0 && format == Format::Json {
                body.push(b']');
            }
        }
    }

    if body.is_empty() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        reply(format, body)
    }
}

/// Makes `session` up front from a required session token; the failure is
/// answered with `id: null`.
async fn authenticate<T: Types>(
    state: &RpcState<T>,
    headers: &HeaderMap,
    format: Format,
    session: &mut Option<SessionResult<T>>,
) -> Result<(), Vec<u8>> {
    let mut body = Vec::new();
    let mut call = Call {
        format,
        id: Some(&Id::Null),
        params: Params::Absent,
        body: &mut body,
    };

    match extract::session_token(headers) {
        Ok(Some(_)) => {}
        Ok(None) => {
            call.fail(
                code::UNAUTHENTICATED,
                "batch requests require authentication",
            );
            return Err(body);
        }
        Err(aux) => {
            SessionFailure::Header(aux).answer(&mut call);
            return Err(body);
        }
    }

    match session.insert(self::session::<T>(&state.service, headers).await) {
        Ok(_) => Ok(()),
        Err(failure) => {
            failure.answer(&mut call);
            Err(body)
        }
    }
}

/// Calls the method in `session`, made on first need, appending any answer to `body`.
async fn run<T: Types>(
    state: &RpcState<T>,
    headers: &HeaderMap,
    format: Format,
    request: Request<'_>,
    session: &mut Option<SessionResult<T>>,
    body: &mut Vec<u8>,
) {
    let mut call = Call {
        format,
        id: request.id.as_ref(),
        params: request.params,
        body,
    };

    let Some((&method, &handler)) = state.methods.get_key_value(&*request.method) else {
        if call.id.is_some() {
            call.fail(
                code::METHOD_NOT_FOUND,
                &format!("method {:?} not found", request.method),
            );
        }
        return;
    };

    async {
        let session = match session {
            Some(session) => session,
            None => session.insert(self::session::<T>(&state.service, headers).await),
        };
        match session {
            Ok(session) => handler(session, call).await,
            Err(failure) => failure.answer(&mut call),
        }
    }
    .in_span(Span::enter_with_local_parent(method))
    .await;
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
