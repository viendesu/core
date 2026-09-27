use std::marker::PhantomData;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::util::ServiceExt;

use viendesu_core::service::{AuxFut, CallStep, Session, SessionMaker, authz::Authentication};
use viendesu_protocol::requests::{
    Response,
    marks::{list_genres, list_tags},
};

// == Mock service ==

struct Fail<O, E>(PhantomData<fn() -> (O, E)>);

fn fail<O, E>() -> Fail<O, E> {
    Fail(PhantomData)
}

impl<I: Send, O: Send, E: Send> CallStep<I> for Fail<O, E> {
    type Output = Response<O, E>;

    async fn call(&mut self, _: I) -> Self::Output {
        unimplemented!("stub endpoint")
    }
}

struct ListGenres;

impl CallStep<list_genres::Args> for ListGenres {
    type Output = Response<list_genres::Ok, list_genres::Err>;

    async fn call(&mut self, _: list_genres::Args) -> Self::Output {
        Ok(list_genres::Ok {
            genres: ["romance".parse().unwrap(), "horror".parse().unwrap()]
                .try_into()
                .unwrap(),
        })
    }
}

struct ListTags;

impl CallStep<list_tags::Args> for ListTags {
    type Output = Response<list_tags::Ok, list_tags::Err>;

    async fn call(&mut self, _: list_tags::Args) -> Self::Output {
        Ok(list_tags::Ok { tags: vec![] })
    }
}

macro_rules! reply {
    (marks list_genres) => {
        ListGenres
    };
    (marks list_tags) => {
        ListTags
    };
    ($segment:ident $endpoint:ident) => {
        fail()
    };
}

macro_rules! mock {
    ($(
        $segment:ident {$(
            $accessor:ident : $Trait:ident {$(
                $(#[$flag:ident])*
                $method:ident = $endpoint:ident {
                    args: $Args:ty,
                    ok: $Ok:ty,
                    err: $Err:ty,
                    output: $Output:ty,
                }
            )*}
        )*}
    )*) => {$($(
        impl viendesu_core::service::$segment::$Trait for Mock {$(
            fn $method(&mut self) -> impl CallStep<$Args, Output = $Output> {
                reply!($segment $endpoint)
            }
        )*}
    )*)*};
}

struct Mock;

viendesu_core::for_each_endpoint!(mock);

impl Authentication for Mock {
    fn authenticate(&mut self, _: viendesu_protocol::types::session::Token) -> impl AuxFut<()> {
        async { Ok(()) }
    }

    fn clear(&mut self) {}
}

struct Service;

impl SessionMaker for Service {
    type Session = Mock;

    fn make_session(&self) -> impl AuxFut<Session<Mock>> {
        async { Ok(Session::new(Mock)) }
    }
}

// == Harness ==

fn router() -> Router {
    viendesu_mcp::router(
        Service,
        viendesu_mcp::catalog::read_only()
            .merge(viendesu_mcp::catalog::forum_posting())
            .merge(viendesu_mcp::catalog::management()),
    )
}

async fn post(router: Router, body: Value) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::post("/")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };

    (status, value)
}

fn rpc(method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })
}

// == Tests ==

#[tokio::test]
async fn initialize() {
    let (status, resp) = post(
        router(),
        rpc("initialize", json!({ "protocolVersion": "2025-06-18" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let result = &resp["result"];
    assert_eq!(result["protocolVersion"], "2025-06-18");
    assert_eq!(result["serverInfo"]["name"], "viendesu");
    assert!(result["capabilities"]["tools"].is_object());
    assert!(
        result["instructions"]
            .as_str()
            .unwrap()
            .ends_with("Genre slugs: romance, horror.")
    );
}

#[tokio::test]
async fn initialize_downgrades_unknown_version() {
    let (_, resp) = post(
        router(),
        rpc("initialize", json!({ "protocolVersion": "1998-05-14" })),
    )
    .await;

    assert_eq!(resp["result"]["protocolVersion"], "2025-06-18");
}

#[tokio::test]
async fn tools_list() {
    let (status, resp) = post(router(), rpc("tools/list", json!({}))).await;

    assert_eq!(status, StatusCode::OK);
    let tools = resp["result"]["tools"].as_array().unwrap();
    let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"search_games"));
    assert!(names.contains(&"post_message"));
    assert!(names.contains(&"update_game"));
    assert!(names.contains(&"delete_board"));

    for tool in tools {
        assert_eq!(tool["inputSchema"]["type"], "object", "{}", tool["name"]);
        assert!(tool["description"].is_string());
    }
}

#[tokio::test]
async fn tools_call() {
    let (status, resp) = post(
        router(),
        rpc(
            "tools/call",
            json!({ "name": "list_tags", "arguments": {} }),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let result = &resp["result"];
    assert_eq!(result["structuredContent"], json!({ "tags": [] }));
    assert!(result["isError"].is_null());
    assert_eq!(result["content"][0]["type"], "text");
}

#[tokio::test]
async fn tools_call_unknown_tool() {
    let (_, resp) = post(router(), rpc("tools/call", json!({ "name": "nope" }))).await;
    assert_eq!(resp["error"]["code"], -32602);
}

#[tokio::test]
async fn tools_call_invalid_args() {
    let (_, resp) = post(
        router(),
        rpc(
            "tools/call",
            json!({ "name": "get_game", "arguments": { "unexpected": true } }),
        ),
    )
    .await;

    assert_eq!(resp["error"]["code"], -32602);
}

#[tokio::test]
async fn notification_is_accepted() {
    let (status, resp) = post(
        router(),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    )
    .await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(resp, Value::Null);
}

#[tokio::test]
async fn unknown_method() {
    let (_, resp) = post(router(), rpc("resources/list", json!({}))).await;
    assert_eq!(resp["error"]["code"], -32601);
}

#[tokio::test]
async fn get_is_method_not_allowed() {
    let response = router()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.headers()["allow"], "POST");
}
