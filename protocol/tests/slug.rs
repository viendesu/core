use viendesu_protocol::types::slug::{LowerSlug, ParseError};

type S = LowerSlug<7>;

#[test]
fn accepts_up_to_max_len() {
    assert_eq!("a".repeat(8).parse::<S>().unwrap().as_str(), "a".repeat(8));
}

#[test]
fn rejects_overlong_instead_of_panicking() {
    for n in [9, 10, 300] {
        assert_eq!("a".repeat(n).parse::<S>(), Err(ParseError::Length));
    }
}

#[test]
fn rejects_empty() {
    assert_eq!("".parse::<S>(), Err(ParseError::Length));
}
