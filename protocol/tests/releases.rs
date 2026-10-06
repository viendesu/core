use serde_json::json;
use viendesu_protocol::types::game::{MAX_RELEASES, Platforms, Release, Releases};

#[test]
fn omitted_platforms_are_false() {
    let p: Platforms = serde_json::from_value(json!({"android": true})).unwrap();
    assert_eq!(
        p,
        Platforms {
            android: true,
            ..Default::default()
        }
    );
}

#[test]
fn minimal_release_round_trips() {
    let r: Release = serde_json::from_value(json!({"name": "1.0"})).unwrap();
    assert!(r.links.is_empty() && r.downloads.is_empty());
    assert_eq!(
        serde_json::to_value(&r).unwrap(),
        json!({"name": "1.0", "links": [], "downloads": []})
    );
}

#[test]
fn link_label_is_optional() {
    let r: Release = serde_json::from_value(json!({
        "name": "1.0",
        "links": [{"url": "https://example.com/tl"}],
    }))
    .unwrap();
    assert_eq!(r.links[0].label, None);
    assert_eq!(
        serde_json::to_value(&r.links[0]).unwrap(),
        json!({"url": "https://example.com/tl"})
    );
}

#[test]
fn link_rejects_other_schemes() {
    let err = serde_json::from_value::<Release>(json!({
        "name": "1.0",
        "links": [{"url": "javascript:alert(1)"}],
    }))
    .unwrap_err();
    assert!(err.to_string().contains("only http and https"), "{err}");
}

#[test]
fn releases_are_capped() {
    let many = vec![json!({"name": "r"}); MAX_RELEASES + 1];
    serde_json::from_value::<Releases>(json!(many)).unwrap_err();
}
