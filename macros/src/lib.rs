use proc_macro::TokenStream;

use eva_api_builder::{ApiBuilder, parse_quote};

/// Defines the service endpoint list as an X-macro named by the input,
/// in the grammar documented by `eva_api_builder`.
#[proc_macro]
pub fn endpoints(args: TokenStream) -> TokenStream {
    ApiBuilder::new("::viendesu_protocol::requests")
        .result_type(|ok, err| parse_quote!(::viendesu_protocol::requests::Response<#ok, #err>))
        .segment("users", |s| {
            s.service("users", "Users", |s| {
                s.methods([
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
        })
        .segment("authors", |s| {
            s.service("authors", "Authors", |s| {
                s.methods(["get", "search", "create", "update"])
            })
        })
        .segment("games", |s| {
            s.service("games", "Games", |s| {
                s.methods(["get", "search", "create", "update", "rate"])
            })
        })
        .segment("boards", |s| {
            s.service("boards", "Boards", |s| {
                s.methods(["get", "create", "delete", "edit"])
            })
        })
        .segment("threads", |s| {
            s.service("threads", "Threads", |s| {
                s.methods(["get", "search", "delete", "edit", "create"])
            })
        })
        .segment("messages", |s| {
            s.service("messages", "Messages", |s| {
                s.methods(["get", "post", "delete", "edit"])
            })
        })
        .segment("blogs", |s| {
            s.service("blogs", "Blogs", |s| s.methods(["get", "edit"]))
        })
        .segment("articles", |s| {
            s.service("articles", "Articles", |s| {
                s.methods(["get", "search", "create", "delete", "edit"])
            })
        })
        .segment("comments", |s| {
            s.service("comments", "Comments", |s| {
                s.methods(["list", "replies", "create", "edit", "delete", "like"])
            })
        })
        .segment("marks", |s| {
            s.service("tags", "Tags", |s| {
                s.method_with("list", |m| m.endpoint("list_tags"))
                    .method_with("add", |m| m.endpoint("add_tag"))
            })
            .service("genres", "Genres", |s| {
                s.method_with("list", |m| m.endpoint("list_genres"))
            })
            .service("badges", "Badges", |s| {
                s.method_with("list", |m| m.endpoint("list_badges"))
                    .method_with("add", |m| m.endpoint("add_badge"))
            })
        })
        .segment("tabs", |s| {
            s.service("tabs", "Tabs", |s| {
                s.methods(["list", "insert", "delete", "list_items"])
            })
        })
        .segment("uploads", |s| {
            s.service("uploads", "Uploads", |s| {
                // `Args` carries the byte stream, so it has no serde form.
                s.methods(["list_pending", "start", "abort"])
                    .method_with("finish", |m| m.flag("stream"))
            })
        })
        .export(args, true)
        .into()
}
