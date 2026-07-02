//! Omitted Patch fields must deserialize as Patch::Keep on every
//! update-like request (requires #[serde_with::apply] to precede #[data],
//! see README "Patch fields in requests").

use viendesu_protocol::{requests, types::entity};

fn parses<T: serde::de::DeserializeOwned>(json: &str) {
    if let Err(e) = serde_json::from_str::<T>(json) {
        panic!("{}: {e}", std::any::type_name::<T>());
    }
}

#[test]
fn update_requests_allow_omitted_patch_fields() {
    parses::<requests::games::update::Update>("{}");
    parses::<requests::authors::update::Update>("{}");
    parses::<requests::users::update::Update>("{}");

    let thread = entity::Id::from_parts(
        0,
        1,
        entity::Metadata::new(entity::Kind::Thread, 0),
    );
    parses::<requests::threads::edit::Args>(&format!(r#"{{"thread":"{thread}"}}"#));
}

#[test]
fn explicit_keep_still_parses() {
    parses::<requests::games::update::Update>(
        r#"{"title":"keep","description":"keep","slug":"keep","thumbnail":"keep","genres":"keep","downloads":"keep","badges":"keep","tags":"keep","screenshots":"keep","published":"keep"}"#,
    );
}
