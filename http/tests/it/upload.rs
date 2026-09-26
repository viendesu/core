use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};

use viendesu_http::server::rpc::code;
use viendesu_protocol::types::{entity, file, upload};

use crate::{
    mock::Mock,
    rpc::{raw_id, send, token},
};

const BOUNDARY: &str = "X-VIENDESU-BOUNDARY";

fn upload_id() -> Value {
    json!(upload::Id::from_generic(raw_id(entity::Kind::Upload, 41)).unwrap())
}

fn file_id() -> Value {
    json!(file::Id::from_generic(raw_id(entity::Kind::File, 13)).unwrap())
}

fn multipart(content: &[u8]) -> Vec<u8> {
    [
        format!(
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.png\"\r\n\
             Content-Type: application/octet-stream\r\n\r\n"
        )
        .as_bytes(),
        content,
        format!("\r\n--{BOUNDARY}--\r\n").as_bytes(),
    ]
    .concat()
}

fn finish(path_id: &str) -> axum::http::request::Builder {
    Request::post(format!("/uploads/{path_id}")).header(
        header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={BOUNDARY}"),
    )
}

fn id_str() -> String {
    upload_id().as_str().unwrap().to_owned()
}

#[tokio::test]
async fn finish_streams_the_file_and_answers_json_rpc() {
    let mock = Mock::default();
    mock.reply("uploads.finish", json!({ "ok": { "file": file_id() } }));

    let request = finish(&id_str())
        .header(header::AUTHORIZATION, format!("Bearer {}", token()))
        .body(Body::from(multipart(b"0123456789")))
        .unwrap();
    let reply = send(&mock, request).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.headers[header::CONNECTION], "close");
    assert_eq!(
        reply.json(),
        json!({ "jsonrpc": "2.0", "id": null, "result": { "file": file_id() } })
    );
    assert_eq!(
        mock.calls(),
        [(
            "uploads.finish",
            json!({ "id": upload_id(), "bytes": 10, "aborted": null })
        )]
    );
    assert_eq!(mock.tokens(), [token()]);
}

#[tokio::test]
async fn finish_answers_in_the_accepted_codec() {
    let mock = Mock::default();
    let err = json!({ "underuploading": { "expected": 20, "got": 10 } });
    mock.reply("uploads.finish", json!({ "err": err }));

    let request = finish(&id_str())
        .header(header::ACCEPT, "application/msgpack")
        .body(Body::from(multipart(b"0123456789")))
        .unwrap();
    let reply = send(&mock, request).await.msgpack();

    assert_eq!(reply["id"], Value::Null);
    assert_eq!(reply["error"]["code"], code::DOMAIN);
    assert_eq!(reply["error"]["data"], err);
}

#[tokio::test]
async fn finish_is_not_limited_to_the_rpc_body_size() {
    let mock = Mock::default();
    mock.reply("uploads.finish", json!({ "ok": { "file": file_id() } }));

    let content = vec![7; 3 * 1024 * 1024];
    let request = finish(&id_str())
        .body(Body::from(multipart(&content)))
        .unwrap();
    let reply = send(&mock, request).await;

    assert!(reply.json().get("result").is_some());
    assert_eq!(mock.calls()[0].1["bytes"], content.len());
}

#[tokio::test]
async fn finish_rejects_bad_ids_and_bodies() {
    let mock = Mock::default();
    mock.reply("uploads.finish", json!({ "ok": { "file": file_id() } }));

    let request = finish("nonsense")
        .body(Body::from(multipart(b"x")))
        .unwrap();
    let reply = send(&mock, request).await;
    assert_eq!(reply.headers[header::CONNECTION], "close");
    let reply = reply.json();
    assert_eq!(reply["error"]["code"], code::INVALID_PARAMS);
    assert!(mock.calls().is_empty());

    let request = Request::post(format!("/uploads/{}", id_str()))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .unwrap();
    send(&mock, request).await;
    let (_, args) = &mock.calls()[0];
    assert!(
        args["aborted"].as_str().unwrap().contains("multipart"),
        "{args}"
    );
}

#[tokio::test]
async fn cors_covers_the_upload_route() {
    let mock = Mock::default();
    let request = Request::options(format!("/uploads/{}", id_str()))
        .header(header::ORIGIN, "https://viende.su")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization")
        .body(Body::empty())
        .unwrap();

    let reply = send(&mock, request).await;
    assert!(
        reply
            .headers
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
    );
}

#[tokio::test]
async fn rest_routes_are_gone() {
    let mock = Mock::default();

    for (method, path) in [
        ("GET", "/genres"),
        ("GET", "/users/@nero"),
        ("POST", "/games/search"),
        ("GET", "/openapi.json"),
        ("GET", "/uploads"),
    ] {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let status = send(&mock, request).await.status;
        assert!(
            status == StatusCode::NOT_FOUND || status == StatusCode::METHOD_NOT_ALLOWED,
            "{method} {path}: {status}"
        );
    }
}
