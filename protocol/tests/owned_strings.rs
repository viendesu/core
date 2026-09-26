//! Tree deserializers (`serde_json::from_value`, as MCP uses) hand out owned
//! strings; every type must accept them, not only borrowed ones.

use serde_json::json;

use viendesu_protocol::requests::games;

#[test]
fn release_date_from_value() {
    let args = json!({
        "title": "T",
        "author": "0bkc----------1k4",
        "release_date": { "date": "04.10.2024", "precision": "day" },
    });

    let args: games::create::Args = serde_json::from_value(args).unwrap();
    assert!(args.release_date.is_some());
}
