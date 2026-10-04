use axum::{
    Router,
    body::{Body, Bytes},
    http::{HeaderMap, Request, StatusCode, header},
    routing::get,
};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::util::ServiceExt as _;

use viendesu_http::server::{Config, config, make_router, rpc::code};
use viendesu_protocol::types::{entity, game, session, user};

use crate::mock::{Mock, Service, Types};

pub fn raw_id(kind: entity::Kind, n: u128) -> entity::Id {
    entity::Id::from_parts(1_700_000, n, entity::Metadata::new(kind, 0))
}

fn user_id() -> Value {
    json!(user::Id::from_generic(raw_id(entity::Kind::User, 3)).unwrap())
}

fn game_id() -> Value {
    json!(game::Id::from_generic(raw_id(entity::Kind::Game, 7)).unwrap())
}

pub fn token() -> session::Token {
    session::Token::from_generic(raw_id(entity::Kind::Session, 29)).unwrap()
}

pub fn app(mock: &Mock) -> Router {
    app_with(mock, config::Rpc::default())
}

pub fn app_with(mock: &Mock, rpc: config::Rpc) -> Router {
    let config = Config {
        unencrypted: None,
        ssl: None,
        rpc,
    };
    make_router::<Types>(Service(mock.clone()), &config, |router| {
        router.route("/extra", get(async || "extra"))
    })
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl Reply {
    pub fn content_type(&self) -> &str {
        self.headers[header::CONTENT_TYPE].to_str().unwrap()
    }

    pub fn json(&self) -> Value {
        assert_eq!(self.content_type(), "application/json");
        serde_json::from_slice(&self.body).unwrap()
    }

    pub fn msgpack(&self) -> Value {
        assert_eq!(self.content_type(), "application/msgpack");
        rmp_serde::from_slice(&self.body).unwrap()
    }

    pub fn error_code(&self) -> i64 {
        self.json()["error"]["code"].as_i64().unwrap()
    }
}

pub async fn send(mock: &Mock, request: Request<Body>) -> Reply {
    send_to(app(mock), request).await
}

pub async fn send_to(app: Router, request: Request<Body>) -> Reply {
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        headers,
        body,
    }
}

fn post(content_type: &str) -> axum::http::request::Builder {
    Request::post("/rpc").header(header::CONTENT_TYPE, content_type)
}

fn json_request(body: Value) -> Request<Body> {
    post("application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn msgpack_request(body: Value) -> Request<Body> {
    post("application/msgpack")
        .body(Body::from(rmp_serde::to_vec_named(&body).unwrap()))
        .unwrap()
}

fn call(method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })
}

#[tokio::test]
async fn json_call_without_params() {
    let mock = Mock::default();
    let ok = json!({ "user": user_id(), "role": "admin" });
    mock.reply("users.check_auth", json!({ "ok": ok }));

    let reply = send(
        &mock,
        json_request(json!({ "jsonrpc": "2.0", "id": "a", "method": "users.check_auth" })),
    )
    .await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json(),
        json!({ "jsonrpc": "2.0", "id": "a", "result": ok })
    );
    assert_eq!(mock.calls(), [("users.check_auth", json!({}))]);
}

#[tokio::test]
async fn msgpack_call_decodes_params_and_encodes_result() {
    let mock = Mock::default();
    let ok = json!({ "user": user_id(), "role": "user" });
    mock.reply("users.check_auth", json!({ "ok": ok }));
    mock.reply(
        "games.get",
        json!({ "err": { "not_found": { "game": { "id": game_id() } } } }),
    );

    let reply = send(&mock, msgpack_request(call("users.check_auth", json!({})))).await;
    assert_eq!(
        reply.msgpack(),
        json!({ "jsonrpc": "2.0", "id": 1, "result": ok })
    );

    let params = json!({
        "game": { "fully_qualified": { "author": "@acme", "slug": "my-game" } },
        "resolve_marks": true,
        "comments": true,
    });
    let reply = send(&mock, msgpack_request(call("games.get", params))).await;
    let reply = reply.msgpack();
    assert_eq!(reply["error"]["code"], code::DOMAIN);
    assert_eq!(
        reply["error"]["data"],
        json!({ "not_found": { "game": { "id": game_id() } } })
    );

    let (method, args) = &mock.calls()[1];
    assert_eq!(*method, "games.get");
    assert_eq!(
        *args,
        json!({
            "game": { "fully_qualified": { "author": "@acme", "slug": "my-game" } },
            "resolve_marks": true,
            "latest_articles": false,
            "comments": true,
            "related": false,
        })
    );
}

#[tokio::test]
async fn accept_picks_the_response_codec() {
    let mock = Mock::default();
    mock.reply("marks.list_genres", json!({ "ok": { "genres": ["rpg"] } }));

    let request = post("application/json; charset=utf-8")
        .header(
            header::ACCEPT,
            "application/json;q=0.5, application/msgpack",
        )
        .body(Body::from(
            call("marks.list_genres", json!(null)).to_string(),
        ))
        .unwrap();
    let reply = send(&mock, request).await;

    assert_eq!(reply.msgpack()["result"], json!({ "genres": ["rpg"] }));
}

#[tokio::test]
async fn domain_errors_carry_the_err_value() {
    let mock = Mock::default();
    let err = json!({ "not_found": { "game": { "id": game_id() } } });
    mock.reply("games.rate", json!({ "err": err }));

    let reply = send(
        &mock,
        json_request(call("games.rate", json!({ "id": game_id(), "rating": 70 }))),
    )
    .await;
    let reply = reply.json();

    assert_eq!(reply["id"], 1);
    assert_eq!(reply["error"]["code"], code::DOMAIN);
    assert_eq!(reply["error"]["data"], err);
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("was not found")
    );
}

#[tokio::test]
async fn auxiliary_errors_map_to_codes() {
    for (aux, expected) in [
        (json!("unauthenticated"), code::UNAUTHENTICATED),
        (json!({ "db": "boom" }), code::INTERNAL_ERROR),
        (
            json!({ "invalid_role": { "required_at_least": "admin" } }),
            code::FORBIDDEN,
        ),
    ] {
        let mock = Mock::default();
        mock.reply("boards.delete", json!({ "aux": aux }));

        let reply = send(
            &mock,
            json_request(call("boards.delete", json!({ "board": "@b" }))),
        )
        .await;
        let reply = reply.json();

        assert_eq!(reply["error"]["code"], expected, "{aux}");
        assert_eq!(reply["error"]["data"], aux);
    }
}

#[tokio::test]
async fn invalid_params() {
    let mock = Mock::default();

    for params in [json!({}), json!([1, 2]), json!("x"), json!({ "game": 5 })] {
        let reply = send(&mock, json_request(call("games.get", params.clone()))).await;
        let reply = reply.json();
        assert_eq!(reply["error"]["code"], code::INVALID_PARAMS, "{params}");
        assert_eq!(reply["id"], 1);
    }

    let reply = send(&mock, json_request(call("games.get", json!({})))).await;
    assert!(
        reply.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("game")
    );
    assert!(mock.calls().is_empty());
}

#[tokio::test]
async fn unknown_methods() {
    let mock = Mock::default();

    for method in ["games.nope", "uploads.finish", "sys.healthcheck"] {
        let reply = send(&mock, json_request(call(method, json!({})))).await;
        assert_eq!(
            reply.error_code(),
            i64::from(code::METHOD_NOT_FOUND),
            "{method}"
        );
    }
}

#[tokio::test]
async fn every_serde_endpoint_is_routed() {
    macro_rules! methods {
        ($(
            $segment:ident {$(
                $accessor:ident : $Trait:ident {$(
                    $(#[$flag:ident])*
                    $method:ident = $endpoint:ident {
                        args: $Args:ty,
                        ok: $Ok:ty,
                        err: $Err:ty,
                        output: $Output:ty,
                        docs: [$($doc:literal),*],
                    }
                )*}
            )*}
        )*) => {
            [$($($(
                (
                    concat!(stringify!($segment), ".", stringify!($endpoint)),
                    stringify!($($flag)*).contains("stream"),
                ),
            )*)*)*]
        };
    }

    let mock = Mock::default();
    let methods = viendesu_core::for_each_endpoint!(methods);
    assert_eq!(methods.len(), 57);

    for (method, stream) in methods {
        let reply = send(&mock, json_request(call(method, json!({})))).await;
        let routed = reply.error_code() != i64::from(code::METHOD_NOT_FOUND);
        assert_eq!(routed, !stream, "{method}");
    }
}

#[tokio::test]
async fn malformed_requests() {
    let mock = Mock::default();

    let cases = [
        (json_body("{"), code::PARSE_ERROR),
        (json_body("[]"), code::INVALID_REQUEST),
        (
            json_body(r#"{"jsonrpc":"1.0","id":1,"method":"users.check_auth"}"#),
            code::INVALID_REQUEST,
        ),
        (
            json_body(r#"{"jsonrpc":"2.0","id":1.5,"method":"users.check_auth"}"#),
            code::INVALID_REQUEST,
        ),
        (json_body("7"), code::INVALID_REQUEST),
        (msgpack_body(vec![0xc1]), code::PARSE_ERROR),
        (msgpack_body(vec![0x92, 0x01]), code::PARSE_ERROR),
        (msgpack_body(vec![0x01, 0x02]), code::PARSE_ERROR),
        (msgpack_body(vec![0x01]), code::INVALID_REQUEST),
        (msgpack_body(duplicate_method()), code::INVALID_REQUEST),
        (msgpack_body(integer_key_in_params()), code::INVALID_REQUEST),
        (msgpack_body(vec![0x81, 0xa1]), code::PARSE_ERROR),
        (msgpack_body(vec![0x90]), code::INVALID_REQUEST),
    ];

    for (request, expected) in cases {
        let reply = send(&mock, request).await;
        assert_eq!(reply.status, StatusCode::OK);
        let body: Value = match reply.content_type() {
            "application/json" => reply.json(),
            _ => reply.msgpack(),
        };
        assert_eq!(body["error"]["code"], expected, "{body}");
        assert_eq!(body["id"], Value::Null);
    }
    assert!(mock.calls().is_empty());
}

/// {"jsonrpc": "2.0", "id": 1, "method": "games.nope", "method": "users.check_auth"}
fn duplicate_method() -> Vec<u8> {
    let mut body = vec![0x84];
    for (key, value) in [
        ("jsonrpc", json!("2.0")),
        ("id", json!(1)),
        ("method", json!("games.nope")),
        ("method", json!("users.check_auth")),
    ] {
        body.extend(rmp_serde::to_vec(key).unwrap());
        body.extend(rmp_serde::to_vec(&value).unwrap());
    }
    body
}

/// games.get with params {0: {"id": ...}, "resolve_marks": true}: serde would
/// bind the integer key to the first declared field.
fn integer_key_in_params() -> Vec<u8> {
    let mut body = vec![0x84];
    for (key, value) in [
        ("jsonrpc", json!("2.0")),
        ("id", json!(1)),
        ("method", json!("games.get")),
    ] {
        body.extend(rmp_serde::to_vec(key).unwrap());
        body.extend(rmp_serde::to_vec(&value).unwrap());
    }
    body.extend(rmp_serde::to_vec("params").unwrap());
    body.push(0x82);
    body.push(0x00);
    body.extend(rmp_serde::to_vec(&json!({ "id": game_id() })).unwrap());
    body.extend(rmp_serde::to_vec("resolve_marks").unwrap());
    body.extend(rmp_serde::to_vec(&true).unwrap());
    body
}

fn json_body(body: &'static str) -> Request<Body> {
    post("application/json").body(Body::from(body)).unwrap()
}

fn msgpack_body(body: Vec<u8>) -> Request<Body> {
    post("application/msgpack").body(Body::from(body)).unwrap()
}

#[tokio::test]
async fn notifications_run_without_a_reply() {
    let mock = Mock::default();

    let reply = send(
        &mock,
        json_request(json!({ "jsonrpc": "2.0", "method": "users.check_auth" })),
    )
    .await;

    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(reply.body.is_empty());
    assert_eq!(mock.calls().len(), 1);

    for body in [
        json!({ "jsonrpc": "2.0", "method": "games.nope" }),
        json!({ "jsonrpc": "2.0", "method": "games.get", "params": {} }),
    ] {
        let reply = send(&mock, json_request(body.clone())).await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT, "{body}");
        assert!(reply.body.is_empty());
    }
    assert_eq!(mock.calls().len(), 1);
}

#[tokio::test]
async fn dates_in_params_decode_in_both_codecs() {
    let mock = Mock::default();
    mock.reply("games.create", json!({ "ok": { "id": game_id() } }));

    let author = json!(
        viendesu_protocol::types::author::Id::from_generic(raw_id(entity::Kind::Author, 11))
            .unwrap()
    );
    let params = json!({
        "title": "T",
        "author": author,
        "release_date": { "date": "04.10.2024", "precision": "day" },
    });

    let reply = send(&mock, msgpack_request(call("games.create", params.clone()))).await;
    assert_eq!(reply.msgpack()["result"], json!({ "id": game_id() }));
    let reply = send(&mock, json_request(call("games.create", params))).await;
    assert_eq!(reply.json()["result"], json!({ "id": game_id() }));

    for (_, args) in mock.calls() {
        assert_eq!(
            args["release_date"],
            json!({ "date": "04.10.2024", "precision": "day" })
        );
    }
    assert_eq!(mock.calls().len(), 2);
}

#[tokio::test]
async fn msgpack_envelope_skips_unknown_members() {
    let mock = Mock::default();
    mock.reply(
        "users.check_auth",
        json!({ "ok": { "user": user_id(), "role": "user" } }),
    );

    let junk: Vec<Value> = vec![Value::Null; 500_000];
    let body = json!({
        "jsonrpc": "2.0",
        "junk": junk,
        "id": 7,
        "method": "users.check_auth",
        "params": { "ignored": [1, [2, [3]], { "x": "y" }] },
    });
    let reply = send(&mock, msgpack_request(body)).await;

    assert_eq!(reply.msgpack()["id"], 7);
    assert!(reply.msgpack().get("result").is_some());
}

#[tokio::test]
async fn transport_errors() {
    let mock = Mock::default();

    for content_type in [
        "text/plain",
        "application/x-www-form-urlencoded",
        "multipart/form-data",
    ] {
        let request = post(content_type)
            .body(Body::from(call("users.check_auth", json!({})).to_string()))
            .unwrap();
        let reply = send(&mock, request).await;
        assert_eq!(
            reply.status,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "{content_type}"
        );
        assert_eq!(reply.error_code(), i64::from(code::INVALID_REQUEST));
    }

    let request = Request::post("/rpc")
        .body(Body::from(call("users.check_auth", json!({})).to_string()))
        .unwrap();
    assert_eq!(
        send(&mock, request).await.status,
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );

    let huge = vec![b' '; 3 * 1024 * 1024];
    let request = post("application/json").body(Body::from(huge)).unwrap();
    assert_eq!(
        send(&mock, request).await.status,
        StatusCode::PAYLOAD_TOO_LARGE
    );

    assert!(mock.calls().is_empty());
}

#[tokio::test]
async fn bearer_token_authenticates_the_session() {
    let mock = Mock::default();
    mock.reply(
        "users.check_auth",
        json!({ "ok": { "user": user_id(), "role": "user" } }),
    );

    let request = |authorization: &str| {
        post("application/json")
            .header(header::AUTHORIZATION, authorization)
            .body(Body::from(call("users.check_auth", json!({})).to_string()))
            .unwrap()
    };

    let reply = send(&mock, request(&format!("Bearer {}", token()))).await;
    assert!(reply.json().get("result").is_some());
    assert_eq!(mock.tokens(), [token()]);

    let reply = send(&mock, request("Bearer nonsense")).await;
    assert_eq!(reply.error_code(), i64::from(code::INVALID_REQUEST));

    mock.reject_tokens();
    let reply = send(&mock, request(&format!("Bearer {}", token()))).await;
    assert_eq!(reply.error_code(), i64::from(code::INVALID_SESSION));
    assert_eq!(mock.calls().len(), 1);
}

#[tokio::test]
async fn cors_covers_rpc_and_extra_routes() {
    let mock = Mock::default();

    for path in ["/rpc", "/extra"] {
        let request = Request::options(path)
            .header(header::ORIGIN, "https://viende.su")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(
                header::ACCESS_CONTROL_REQUEST_HEADERS,
                "content-type,authorization",
            )
            .body(Body::empty())
            .unwrap();
        let reply = send(&mock, request).await;
        assert!(
            reply
                .headers
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            "{path}"
        );
    }

    let request = Request::get("/extra")
        .header(header::ORIGIN, "https://viende.su")
        .body(Body::empty())
        .unwrap();
    let reply = send(&mock, request).await;
    assert_eq!(reply.body, "extra");
    assert!(
        reply
            .headers
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
    );
}

#[tokio::test]
async fn batch_answers_in_request_order() {
    let mock = Mock::default();
    let auth = json!({ "user": user_id(), "role": "admin" });
    mock.reply("users.check_auth", json!({ "ok": auth }));
    mock.reply("marks.list_genres", json!({ "ok": { "genres": ["rpg"] } }));

    let batch = json!([
        { "jsonrpc": "2.0", "id": "a", "method": "users.check_auth" },
        { "jsonrpc": "2.0", "method": "users.check_auth" },
        { "jsonrpc": "2.0", "id": 2, "method": "games.nope" },
        7,
        { "jsonrpc": "2.0", "id": 3, "method": "marks.list_genres" },
    ]);
    let reply = send(&mock, authed(json_request(batch))).await;

    assert_eq!(reply.status, StatusCode::OK);
    let reply = reply.json();
    let replies = reply.as_array().unwrap();
    assert_eq!(replies.len(), 4);
    assert_eq!(
        replies[0],
        json!({ "jsonrpc": "2.0", "id": "a", "result": auth })
    );
    assert_eq!(replies[1]["id"], 2);
    assert_eq!(replies[1]["error"]["code"], code::METHOD_NOT_FOUND);
    assert_eq!(replies[2]["id"], Value::Null);
    assert_eq!(replies[2]["error"]["code"], code::INVALID_REQUEST);
    assert_eq!(
        replies[3],
        json!({ "jsonrpc": "2.0", "id": 3, "result": { "genres": ["rpg"] } })
    );
    assert_eq!(mock.calls().len(), 3);
}

#[tokio::test]
async fn msgpack_batch() {
    let mock = Mock::default();
    mock.reply("marks.list_genres", json!({ "ok": { "genres": ["rpg"] } }));

    let batch = json!([
        call("marks.list_genres", json!(null)),
        [call("marks.list_genres", json!(null))],
    ]);
    let reply = send(&mock, authed(msgpack_request(batch))).await;

    let reply = reply.msgpack();
    assert_eq!(reply[0]["result"], json!({ "genres": ["rpg"] }));
    assert_eq!(reply[1]["error"]["code"], code::INVALID_REQUEST);
    assert_eq!(mock.calls().len(), 1);
}

#[tokio::test]
async fn batch_of_notifications_has_no_body() {
    let mock = Mock::default();
    let notification = json!({ "jsonrpc": "2.0", "method": "users.check_auth" });

    let reply = send(
        &mock,
        authed(json_request(json!([notification, notification]))),
    )
    .await;

    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(reply.body.is_empty());
    assert_eq!(mock.calls().len(), 2);
}

#[tokio::test]
async fn batch_limit_is_configurable() {
    let mock = Mock::default();
    mock.reply("marks.list_genres", json!({ "ok": { "genres": [] } }));
    let batch = |n| Value::Array(vec![call("marks.list_genres", json!(null)); n]);

    let reply = send(&mock, authed(json_request(batch(6)))).await;
    assert_eq!(reply.error_code(), i64::from(code::INVALID_REQUEST));
    assert!(mock.calls().is_empty());

    let reply = send(&mock, authed(json_request(batch(5)))).await;
    assert_eq!(reply.json().as_array().unwrap().len(), 5);

    let rpc = config::Rpc {
        max_batch: 2.try_into().unwrap(),
        ..Default::default()
    };
    let reply = send_to(app_with(&mock, rpc), authed(json_request(batch(3)))).await;
    assert_eq!(reply.error_code(), i64::from(code::INVALID_REQUEST));
    assert_eq!(mock.calls().len(), 5);
}

#[tokio::test]
async fn batch_shares_one_session() {
    let mock = Mock::default();
    mock.reply("marks.list_genres", json!({ "ok": { "genres": [] } }));

    let batch = |authorization: &str| {
        let body = json!([
            call("games.nope", json!({})),
            call("marks.list_genres", json!(null)),
            call("marks.list_genres", json!(null)),
        ]);
        post("application/json")
            .header(header::AUTHORIZATION, authorization)
            .body(Body::from(body.to_string()))
            .unwrap()
    };

    let reply = send(&mock, batch(&format!("Bearer {}", token()))).await;
    assert_eq!(reply.json().as_array().unwrap().len(), 3);
    assert_eq!(mock.tokens(), [token()]);
    assert_eq!(mock.calls().len(), 2);

    mock.reject_tokens();
    let rpc = config::Rpc {
        batch_requires_auth: false,
        ..Default::default()
    };
    let reply = send_to(app_with(&mock, rpc), batch(&format!("Bearer {}", token()))).await;
    let reply = reply.json();
    assert_eq!(reply[0]["error"]["code"], code::METHOD_NOT_FOUND);
    for reply in &reply.as_array().unwrap()[1..] {
        assert_eq!(reply["error"]["code"], code::INVALID_SESSION);
    }
    assert_eq!(mock.tokens().len(), 2);
    assert_eq!(mock.calls().len(), 2);
}

#[tokio::test]
async fn batches_require_authentication() {
    let mock = Mock::default();
    mock.reply("marks.list_genres", json!({ "ok": { "genres": [] } }));
    let batch = || json_request(json!([call("marks.list_genres", json!(null))]));

    let reply = send(&mock, batch()).await;
    assert_eq!(reply.status, StatusCode::OK);
    let body = reply.json();
    assert_eq!(body["id"], Value::Null);
    assert_eq!(body["error"]["code"], code::UNAUTHENTICATED);

    let mut request = batch();
    request
        .headers_mut()
        .insert(header::AUTHORIZATION, "Bearer nonsense".parse().unwrap());
    assert_eq!(
        send(&mock, request).await.error_code(),
        i64::from(code::INVALID_REQUEST)
    );

    mock.reject_tokens();
    let reply = send(&mock, authed(batch())).await;
    assert_eq!(reply.error_code(), i64::from(code::INVALID_SESSION));
    assert!(mock.calls().is_empty());

    let rpc = config::Rpc {
        batch_requires_auth: false,
        ..Default::default()
    };
    let reply = send_to(app_with(&mock, rpc), batch()).await;
    assert_eq!(reply.json()[0]["result"], json!({ "genres": [] }));
    assert_eq!(mock.calls().len(), 1);
}

fn authed(mut request: Request<Body>) -> Request<Body> {
    let value = format!("Bearer {}", token()).parse().unwrap();
    request.headers_mut().insert(header::AUTHORIZATION, value);
    request
}
