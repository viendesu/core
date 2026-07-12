use std::{
    fmt::Display,
    sync::{Arc, OnceLock},
};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing,
};

use serde::Deserialize;
use serde_json::{Value, json};

use viendesu_core::service::{
    CallStep as _, SessionMaker, SessionOf, authz::Authentication as _, marks::Genres as _,
};
use viendesu_protocol::{requests::marks, types::session};

use crate::registry::{CallOutcome, Tools};

/// Newest first.
const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26"];

/// Base instructions; [`initialize`] appends the genre slugs to them.
const INSTRUCTIONS: &str = "VienDesu! is a Russian visual novel catalog and forum. \
    To find games by theme or genre, call search_games with genre slugs in \
    include.genres_any. Its text query matches game titles only — never put \
    themes or genres there. \
    Authenticated requests (Authorization: Bearer <session token>) may also \
    act on behalf of the user, subject to their role.";

struct McpState<S: SessionMaker> {
    service: S,
    tools: Tools<SessionOf<S>>,
    /// Instructions with the genre slugs embedded, built on first
    /// successful fetch in [`initialize`].
    instructions: OnceLock<String>,
}

/// MCP endpoint at `/` over the streamable HTTP transport (stateless:
/// no sessions, no SSE stream). Nest it wherever appropriate, e.g.
/// `router.nest("/mcp", viendesu_mcp::router(service, tools))`.
pub fn router<S>(service: S, tools: Tools<SessionOf<S>>) -> Router
where
    S: SessionMaker + Send + Sync + 'static,
    SessionOf<S>: 'static,
{
    let state = Arc::new(McpState {
        service,
        tools,
        instructions: OnceLock::new(),
    });
    Router::new().route(
        "/",
        routing::post(handle::<S>)
            .fallback(async || {
                (
                    StatusCode::METHOD_NOT_ALLOWED,
                    [(header::ALLOW, "POST")],
                    (),
                )
            })
            .with_state(state),
    )
}

#[derive(Deserialize)]
struct Message {
    id: Option<Value>,
    method: Option<String>,
    #[serde(default)]
    params: Value,
}

async fn handle<S>(
    State(state): State<Arc<McpState<S>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response
where
    S: SessionMaker + Send + Sync + 'static,
    SessionOf<S>: 'static,
{
    let msg: Message = match serde_json::from_slice(&body) {
        Ok(m) => m,
        Err(e) => return rpc_err(Value::Null, -32700, format_args!("parse error: {e}")),
    };

    // Client responses and notifications expect no reply.
    let (Some(method), Some(id)) = (msg.method, msg.id) else {
        return StatusCode::ACCEPTED.into_response();
    };

    match method.as_str() {
        "initialize" => initialize(&state, id, &msg.params).await,
        "ping" => rpc_ok(id, json!({})),
        "tools/list" => rpc_ok(id, json!({ "tools": state.tools.list() })),
        "tools/call" => call(&state, &headers, id, msg.params).await,
        _ => rpc_err(
            id,
            -32601,
            format_args!("method {method:?} is not supported"),
        ),
    }
}

async fn initialize<S>(state: &McpState<S>, id: Value, params: &Value) -> Response
where
    S: SessionMaker + Send + Sync + 'static,
{
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let version = requested
        .filter(|v| SUPPORTED_VERSIONS.contains(v))
        .unwrap_or(SUPPORTED_VERSIONS[0]);

    // Genres are effectively static, so they are embedded into the
    // instructions once: clients get the valid slugs upfront instead of
    // discovering them via list_genres. On fetch failure fall back to
    // the base text and retry on the next initialize.
    let instructions = match state.instructions.get() {
        Some(cached) => cached.as_str(),
        None => match genre_slugs(&state.service).await {
            Some(slugs) => state
                .instructions
                .get_or_init(|| format!("{INSTRUCTIONS} Genre slugs: {slugs}.")),
            None => INSTRUCTIONS,
        },
    };

    rpc_ok(
        id,
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "viendesu",
                "title": "VienDesu!",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "instructions": instructions,
        }),
    )
}

async fn genre_slugs<S: SessionMaker>(service: &S) -> Option<String> {
    let mut session = service.make_session().await.ok()?;
    let marks::list_genres::Ok { genres } = session
        .genres()
        .list()
        .call(marks::list_genres::Args {})
        .await
        .ok()?;

    if genres.is_empty() {
        return None;
    }

    let mut list = String::new();
    for genre in &genres {
        if !list.is_empty() {
            list.push_str(", ");
        }
        list.push_str(genre.as_str());
    }
    Some(list)
}

async fn call<S>(state: &McpState<S>, headers: &HeaderMap, id: Value, params: Value) -> Response
where
    S: SessionMaker + Send + Sync + 'static,
    SessionOf<S>: 'static,
{
    #[derive(Deserialize)]
    struct Params {
        name: String,
        #[serde(default)]
        arguments: Value,
    }

    let Params {
        name,
        mut arguments,
    } = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return rpc_err(id, -32602, format_args!("invalid params: {e}")),
    };
    if arguments.is_null() {
        arguments = json!({});
    }

    let mut session = match state.service.make_session().await {
        Ok(s) => s,
        Err(e) => return rpc_err(id, -32603, format_args!("failed to make session: {e}")),
    };

    match bearer_token(headers) {
        Ok(None) => {}
        Ok(Some(token)) => {
            if let Err(e) = session.authz().authenticate(token).await {
                return rpc_ok(
                    id,
                    tool_error_text(format_args!("authentication failed: {e}")),
                );
            }
        }
        Err(e) => return rpc_ok(id, tool_error_text(e)),
    }

    let Some(outcome) = state.tools.call(&name, session, arguments) else {
        return rpc_err(id, -32602, format_args!("unknown tool {name:?}"));
    };

    match outcome.await {
        CallOutcome::Ok(value) => {
            let mut result = json!({
                "content": [{ "type": "text", "text": pretty(&value) }],
            });
            if value.is_object() {
                result["structuredContent"] = value;
            }
            rpc_ok(id, result)
        }
        CallOutcome::ToolError(value) => rpc_ok(
            id,
            json!({
                "content": [{ "type": "text", "text": pretty(&value) }],
                "isError": true,
            }),
        ),
        CallOutcome::InvalidArgs(msg) => rpc_err(id, -32602, msg),
    }
}

fn bearer_token(headers: &HeaderMap) -> Result<Option<session::Token>, String> {
    let Some(value) = headers.get(header::AUTHORIZATION) else {
        return Ok(None);
    };
    let value = value
        .to_str()
        .map_err(|e| format!("failed to decode Authorization header: {e}"))?;
    let Some((scheme, rest)) = value.split_once(' ') else {
        return Err("invalid Authorization header format, expected `<scheme> <token>`".into());
    };
    if scheme != "Bearer" {
        return Err(format!("scheme {scheme:?} is not supported"));
    }

    rest.parse()
        .map(Some)
        .map_err(|e| format!("invalid session token: {e}"))
}

fn tool_error_text(message: impl Display) -> Value {
    json!({
        "content": [{ "type": "text", "text": message.to_string() }],
        "isError": true,
    })
}

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).expect("Value serialization is infallible")
}

fn rpc_ok(id: Value, result: Value) -> Response {
    json_response(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn rpc_err(id: Value, code: i64, message: impl Display) -> Response {
    json_response(json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message.to_string() },
    }))
}

fn json_response(value: Value) -> Response {
    (
        [(header::CONTENT_TYPE, "application/json")],
        value.to_string(),
    )
        .into_response()
}
