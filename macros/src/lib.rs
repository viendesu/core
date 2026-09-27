use proc_macro::TokenStream;

use eva_api_builder::{ApiBuilder, parse_quote};

/// Defines the service traits and the endpoint list X-macro named by the input,
/// in the grammar documented by `eva_api_builder`.
#[proc_macro]
pub fn endpoints(args: TokenStream) -> TokenStream {
    ApiBuilder::new("::viendesu_protocol::requests")
        .result_type(|ok, err| parse_quote!(::viendesu_protocol::requests::Response<#ok, #err>))
        .service("users", |svc| {
            svc.methods([
                "get",
                "check_auth",
                "search",
                "begin_auth",
                "finish_auth",
                "sign_in",
                "sign_up",
                "update",
                "confirm_sign_up",
            ])
        })
        .service("authors", |svc| {
            svc.methods(["get", "search", "create", "update"])
        })
        .service("games", |svc| {
            svc.methods(["get", "search", "create", "update", "rate"])
        })
        .service("boards", |svc| {
            svc.methods(["get", "create", "delete", "edit"])
        })
        .service("threads", |svc| {
            svc.methods(["get", "search", "delete", "edit", "create"])
        })
        .service("messages", |svc| {
            svc.methods(["get", "post", "delete", "edit"])
        })
        .service("blogs", |svc| svc.methods(["get", "edit"]))
        .service("articles", |svc| {
            svc.methods(["get", "search", "create", "delete", "edit"])
        })
        .service("comments", |svc| {
            svc.methods(["list", "replies", "create", "edit", "delete", "like"])
        })
        .segment("marks", |seg| {
            seg.service("tags", |svc| {
                svc.method_with("list", |m| m.endpoint("list_tags"))
                    .method_with("add", |m| m.endpoint("add_tag"))
            })
            .service("genres", |svc| {
                svc.method_with("list", |m| m.endpoint("list_genres"))
            })
            .service("badges", |svc| {
                svc.method_with("list", |m| m.endpoint("list_badges"))
                    .method_with("add", |m| m.endpoint("add_badge"))
            })
        })
        .service("tabs", |svc| {
            svc.methods(["list", "insert", "delete", "list_items"])
        })
        .service("uploads", |svc| {
            // `Args` carries the byte stream, so it has no serde form.
            svc.methods(["list_pending", "start", "abort"])
                .method_with("finish", |m| m.flag("stream"))
        })
        .export(args, true)
        .into()
}
