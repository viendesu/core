use axum::{
    Router,
    body::{Body, Bytes},
    http::{HeaderMap, Request, StatusCode, header},
    routing::get,
};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::util::ServiceExt as _;

use viendesu_http::server::{make_router, rpc::code};
use viendesu_protocol::types::{entity, game, session, user};

use crate::mock::{Mock, Service, Types};

fn raw_id(kind: entity::Kind, n: u128) -> entity::Id {
    entity::Id::from_parts(1_700_000, n, entity::Metadata::new(kind, 0))
}

fn user_id() -> Value {
    json!(user::Id::from_generic(raw_id(entity::Kind::User, 3)).unwrap())
}

fn game_id() -> Value {
    json!(game::Id::from_generic(raw_id(entity::Kind::Game, 7)).unwrap())
}

fn token() -> session::Token {
    session::Token::from_generic(raw_id(entity::Kind::Session, 29)).unwrap()
}

fn app(mock: &Mock) -> Router {
    make_router::<Types>(Service(mock.clone()), |router| {
        router.route("/extra", get(async || "extra"))
    })
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl Reply {
    fn content_type(&self) -> &str {
        self.headers[header::CONTENT_TYPE].to_str().unwrap()
    }

    fn json(&self) -> Value {
        assert_eq!(self.content_type(), "application/json");
        serde_json::from_slice(&self.body).unwrap()
    }

    fn msgpack(&self) -> Value {
        assert_eq!(self.content_type(), "application/msgpack");
        rmp_serde::from_slice(&self.body).unwrap()
    }

    fn error_code(&self) -> i64 {
        self.json()["error"]["code"].as_i64().unwrap()
    }
}

async fn send(mock: &Mock, request: Request<Body>) -> Reply {
    let response = app(mock).oneshot(request).await.unwrap();
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
            $module:ident {$(
                $accessor:ident : $Trait:ident {
                    $( $(#[$attr:ident])* $method:ident $(=> $endpoint:ident)? ),* $(,)?
                }
            )*}
        )*) => {
            [$($($(
                (
                    concat!(stringify!($module), ".", method!($method $($endpoint)?)),
                    !stringify!($($attr)*).is_empty(),
                ),
            )*)*)*]
        };
    }
    macro_rules! method {
        ($method:ident) => {
            stringify!($method)
        };
        ($method:ident $endpoint:ident) => {
            stringify!($endpoint)
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
            json_body(r#"[{"jsonrpc":"2.0","id":1,"method":"users.check_auth"}]"#),
            code::INVALID_REQUEST,
        ),
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
        (
            msgpack_body(rmp_serde::to_vec(&json!([call("users.check_auth", json!({}))])).unwrap()),
            code::INVALID_REQUEST,
        ),
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
