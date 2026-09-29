use viendesu_protocol::types::{
    game::DownloadLink,
    http_url::{HttpUrl, ParseError},
};

#[test]
fn accepts_web_schemes() {
    for s in [
        "http://example.com/a",
        "https://example.com/a",
        "HTTPS://example.com",
    ] {
        assert!(s.parse::<HttpUrl>().is_ok(), "{s}");
    }
}

#[test]
fn rejects_other_schemes() {
    for s in [
        "javascript:alert(1)",
        "JavaScript:alert(1)",
        "data:text/html,<script>alert(1)</script>",
        "vbscript:msgbox(1)",
        "ftp://example.com/a",
        "file:///etc/passwd",
    ] {
        assert_eq!(s.parse::<HttpUrl>(), Err(ParseError::Scheme), "{s}");
    }
}

#[test]
fn rejects_malformed() {
    assert_eq!("not a url".parse::<HttpUrl>(), Err(ParseError::Malformed));
}

#[test]
fn external_link_does_not_deserialize_with_other_schemes() {
    let ok: DownloadLink = serde_json::from_str(r#"{"external":"https://example.com/a"}"#).unwrap();
    assert_eq!(
        ok,
        DownloadLink::External("https://example.com/a".parse().unwrap())
    );

    let err =
        serde_json::from_str::<DownloadLink>(r#"{"external":"javascript:alert(1)"}"#).unwrap_err();
    assert!(err.to_string().contains("only http and https"), "{err}");
}
