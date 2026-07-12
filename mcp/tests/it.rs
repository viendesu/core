use std::marker::PhantomData;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::util::ServiceExt;

use viendesu_core::service::{
    AuxFut, CallStep, RespFut, Session, SessionMaker, authors::Authors, authz::Authentication,
    boards::Boards, games::Games, marks::{Badges, Genres, Tags}, messages::Messages, tabs::Tabs,
    threads::Threads, uploads::Uploads, users::Users,
};
use viendesu_protocol::requests;

// == Mock service ==

struct Fail<O, E>(PhantomData<fn() -> (O, E)>);

fn fail<O, E>() -> Fail<O, E> {
    Fail(PhantomData)
}

impl<I: Send, O: Send, E: Send> CallStep<I> for Fail<O, E> {
    type Ok = O;
    type Err = E;

    fn call(&mut self, _: I) -> impl RespFut<O, E> {
        async { unimplemented!("stub endpoint") }
    }
}

macro_rules! stub {
    ($($method:ident => $($seg:ident)::+),* $(,)?) => {$(
        fn $method(
            &mut self,
        ) -> impl CallStep<
            $($seg)::+::Args,
            Ok = $($seg)::+::Ok,
            Err = $($seg)::+::Err,
        > {
            fail()
        }
    )*};
}

struct Mock;

impl Users for Mock {
    stub! {
        get => requests::users::get,
        check_auth => requests::users::check_auth,
        search => requests::users::search,
        begin_auth => requests::users::begin_auth,
        finish_auth => requests::users::finish_auth,
        sign_in => requests::users::sign_in,
        sign_up => requests::users::sign_up,
        update => requests::users::update,
        confirm_sign_up => requests::users::confirm_sign_up,
    }
}

impl Authors for Mock {
    stub! {
        get => requests::authors::get,
        search => requests::authors::search,
        create => requests::authors::create,
        update => requests::authors::update,
    }
}

impl Games for Mock {
    stub! {
        get => requests::games::get,
        search => requests::games::search,
        create => requests::games::create,
        update => requests::games::update,
    }
}

impl Boards for Mock {
    stub! {
        get => requests::boards::get,
        create => requests::boards::create,
        delete => requests::boards::delete,
        edit => requests::boards::edit,
    }
}

impl Threads for Mock {
    stub! {
        get => requests::threads::get,
        search => requests::threads::search,
        delete => requests::threads::delete,
        edit => requests::threads::edit,
        create => requests::threads::create,
    }
}

impl Messages for Mock {
    stub! {
        get => requests::messages::get,
        post => requests::messages::post,
        delete => requests::messages::delete,
        edit => requests::messages::edit,
    }
}

impl Genres for Mock {
    stub!(list => requests::marks::list_genres);
}

impl Badges for Mock {
    stub! {
        list => requests::marks::list_badges,
        add => requests::marks::add_badge,
    }
}

struct ListTags;

impl CallStep<requests::marks::list_tags::Args> for ListTags {
    type Ok = requests::marks::list_tags::Ok;
    type Err = requests::marks::list_tags::Err;

    fn call(&mut self, _: requests::marks::list_tags::Args) -> impl RespFut<Self::Ok, Self::Err> {
        async { Ok(requests::marks::list_tags::Ok { tags: vec![] }) }
    }
}

impl Tags for Mock {
    fn list(
        &mut self,
    ) -> impl CallStep<
        requests::marks::list_tags::Args,
        Ok = requests::marks::list_tags::Ok,
        Err = requests::marks::list_tags::Err,
    > {
        ListTags
    }

    stub!(add => requests::marks::add_tag);
}

impl Tabs for Mock {
    stub! {
        list => requests::tabs::list,
        insert => requests::tabs::insert,
        delete => requests::tabs::delete,
        list_items => requests::tabs::list_items,
    }
}

impl Uploads for Mock {
    stub! {
        list_pending => requests::uploads::list_pending,
        start => requests::uploads::start,
        abort => requests::uploads::abort,
        finish => requests::uploads::finish,
    }
}

impl Authentication for Mock {
    fn authenticate(
        &mut self,
        _: viendesu_protocol::types::session::Token,
    ) -> impl AuxFut<()> {
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
        rpc("tools/call", json!({ "name": "list_tags", "arguments": {} })),
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
