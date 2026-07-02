# viendesu core

## Gotchas

### Patch fields in requests

`#[serde_with::apply(Patch => #[serde(default)])]` must be written **above**
`#[data]`. `#[data]` emits its serde derives above the re-emitted item, so an
`apply` placed below it expands too late to add `#[serde(default)]` to the
fields — omitting a `Patch` field then fails with `missing field` instead of
meaning `Keep`. Guarded by `protocol/tests/patch_defaults.rs`.

TODO: fix in eva itself (make `#[data]` hoist remaining item attributes above
its derives, or reject a late `serde_with::apply`), then drop this note.
