//! Built-in tool sets, generated from the endpoint list.
//!
//! Every endpoint becomes a tool named `{accessor}_{method}`, described by its
//! docs plus the LLM-only context of `add_llm_context!`. Extend with
//! [`Tools::tool`] / [`Tools::merge`].

use crate::registry::Tools;

use viendesu_core::service::{CallStep as _, IsSession, Session};

/// Appends LLM-only doc lines to the descriptions of the listed
/// `accessor::method` tools.
macro_rules! add_llm_context {
    ($($(#[doc = $doc:literal])* $accessor:ident::$method:ident),* $(,)?) => {
        add_llm_context!(@ [$] $([$($doc)*] $accessor $method)*);
    };
    (@ [$d:tt] $([$($doc:literal)*] $accessor:ident $method:ident)*) => {
        macro_rules! llm_context {
            $(($accessor $method) => { &[$($doc),*] };)*
            ($d($d _:tt)*) => { &[] };
        }
    };
}

add_llm_context! {
    /// Requires authentication.
    users::check_auth,
    /// Set `resolve_marks` to also resolve tag, genre and badge names.
    games::get,
    /// For theme or genre requests use `include.genres_any` with genre slugs,
    /// listed in the server instructions or via `genres_list`; never put
    /// themes or genres into the text query. Set `resolve_marks` to also
    /// resolve tag and badge names.
    games::search,
    /// `content` is CommonMark.
    articles::get,
    /// `content` is CommonMark.
    articles::create,
}

/// Endpoints that are no tools: MCP authenticates by the bearer token, and
/// uploads cannot be finished without a byte stream.
const HIDDEN: &[&str] = &[
    "users_begin_auth",
    "users_finish_auth",
    "users_sign_in",
    "users_sign_up",
    "users_confirm_sign_up",
    "uploads_start",
    "uploads_abort",
    "uploads_list_pending",
];

/// Services whose writes are forum posting rather than content management.
const FORUM: &[&str] = &["threads", "messages", "comments"];

/// Coordinates of a generated tool, for picking tool sets.
pub struct Endpoint {
    pub accessor: &'static str,
    pub method: &'static str,
    pub flags: &'static [&'static str],
}

impl Endpoint {
    pub fn is_write(&self) -> bool {
        self.flags.contains(&"write_op")
    }
}

/// Read-only tools over all queryable domains.
pub fn read_only<S: IsSession + 'static>() -> Tools<S> {
    select(|e| !e.is_write())
}

/// Content management tools. Authorization is enforced by the service, so
/// these require an authenticated session with a sufficient role.
pub fn management<S: IsSession + 'static>() -> Tools<S> {
    select(|e| e.is_write() && !FORUM.contains(&e.accessor))
}

/// Forum write tools. Authorization is enforced by the service, so these
/// require an authenticated session.
pub fn forum_posting<S: IsSession + 'static>() -> Tools<S> {
    select(|e| e.is_write() && FORUM.contains(&e.accessor))
}

/// Tools of every exposable endpoint passing `keep`.
pub fn select<S: IsSession + 'static>(mut keep: impl FnMut(&Endpoint) -> bool) -> Tools<S> {
    let mut tools = Tools::new();

    // Streaming endpoints have no JSON form, so they cannot be tools.
    macro_rules! unless_stream {
        ([stream $($rest:ident)*] $($body:tt)*) => {};
        ([$head:ident $($rest:ident)*] $($body:tt)*) => {
            unless_stream!([$($rest)*] $($body)*);
        };
        ([] $($body:tt)*) => { $($body)* };
    }

    macro_rules! register {
        ($(
            $segment:ident {$(
                $accessor:ident : $Trait:ident {$(
                    $(#[$flag:ident])*
                    $method:ident = $endpoint:ident {
                        args: $Args:ty,
                        ok: $Ok:ty,
                        err: $Err:ty,
                        output: $Output:ty,
                        docs: [$($doc:literal),*],
                    }
                )*}
            )*}
        )*) => {$($($(
            unless_stream!([$($flag)*] {
                use viendesu_core::service::$segment::$Trait as _;

                let name = concat!(stringify!($accessor), "_", stringify!($method));
                let endpoint = Endpoint {
                    accessor: stringify!($accessor),
                    method: stringify!($method),
                    flags: &[$(stringify!($flag)),*],
                };
                if !HIDDEN.contains(&name) && keep(&endpoint) {
                    tools = tools.tool(
                        name,
                        describe(&[$($doc),*], llm_context!($accessor $method)),
                        |mut s: Session<S>, args: $Args| async move {
                            s.$accessor().$method().call(args).await
                        },
                    );
                }
            });
        )*)*)*};
    }

    viendesu_core::for_each_endpoint!(register);
    tools
}

/// Joins doc lines, dropping the space `///` leaves after the slashes.
fn describe(doc: &[&str], llm: &[&str]) -> String {
    let separator = (!doc.is_empty() && !llm.is_empty()).then_some("");
    doc.iter()
        .copied()
        .chain(separator)
        .chain(llm.iter().copied())
        .map(|l| l.strip_prefix(' ').unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
}
