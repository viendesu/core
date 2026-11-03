use serde_json::json;
use viendesu_protocol::requests::Tabs::list_items;

#[test]
fn list_items_preserves_variant_shapes_and_default_marks() {
    let games: list_items::Ok = serde_json::from_value(json!({
        "games": { "items": [], "authors": {}, "users": {} }
    }))
    .unwrap();
    assert!(matches!(games, list_items::Ok::Games { .. }));
    assert_eq!(
        serde_json::to_value(games).unwrap(),
        json!({
            "games": {
                "items": [],
                "marks": { "tags": {}, "badges": {} },
                "authors": {},
                "users": {}
            }
        })
    );

    let expected = json!({ "authors": { "items": [], "users": {} } });
    let authors: list_items::Ok = serde_json::from_value(expected.clone()).unwrap();
    assert!(matches!(authors, list_items::Ok::Authors { .. }));
    assert_eq!(serde_json::to_value(authors).unwrap(), expected);
}
