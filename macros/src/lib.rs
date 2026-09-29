use proc_macro::TokenStream;

use eva_api_builder::{ApiBuilder, ServiceBuilder, parse_quote};

trait Access {
    fn read<'a>(self, names: impl IntoIterator<Item = &'a str>) -> Self;

    /// Methods that change state, flagged `write_op` for rate limiting.
    fn write<'a>(self, names: impl IntoIterator<Item = &'a str>) -> Self;
}

impl Access for ServiceBuilder {
    fn read<'a>(self, names: impl IntoIterator<Item = &'a str>) -> Self {
        self.methods(names)
    }

    fn write<'a>(self, names: impl IntoIterator<Item = &'a str>) -> Self {
        names.into_iter().fold(self, |svc, name| {
            svc.method_with(name, |m| m.flag("write_op"))
        })
    }
}

/// Defines the service traits and the endpoint list X-macro named by the input,
/// in the grammar documented by `eva_api_builder`.
#[proc_macro]
pub fn endpoints(args: TokenStream) -> TokenStream {
    ApiBuilder::new("::viendesu_protocol::requests")
        .result_type(|ok, err| parse_quote!(::viendesu_protocol::requests::Response<#ok, #err>))
        .service("users", |svc| {
            svc.read(["get", "check_auth", "search"]).write([
                "begin_auth",
                "finish_auth",
                "sign_in",
                "sign_up",
                "update",
                "confirm_sign_up",
            ])
        })
        .service("authors", |svc| {
            svc.read(["get", "search"]).write(["create", "update"])
        })
        .service("games", |svc| {
            svc.read(["get", "search"])
                .write(["create", "update", "rate"])
        })
        .service("boards", |svc| {
            svc.read(["get"]).write(["create", "delete", "edit"])
        })
        .service("threads", |svc| {
            svc.read(["get", "search"])
                .write(["delete", "edit", "create"])
        })
        .service("messages", |svc| {
            svc.read(["get"]).write(["post", "delete", "edit"])
        })
        .service("blogs", |svc| svc.read(["get"]).write(["edit"]))
        .service("articles", |svc| {
            svc.read(["get", "search"])
                .write(["create", "delete", "edit"])
        })
        .service("comments", |svc| {
            svc.read(["list", "replies"])
                .write(["create", "edit", "delete", "like"])
        })
        .segment("marks", |seg| {
            seg.service("tags", |svc| {
                svc.method_with("list", |m| m.endpoint("list_tags"))
                    .method_with("add", |m| m.endpoint("add_tag").flag("write_op"))
            })
            .service("genres", |svc| {
                svc.method_with("list", |m| m.endpoint("list_genres"))
            })
            .service("badges", |svc| {
                svc.method_with("list", |m| m.endpoint("list_badges"))
                    .method_with("add", |m| m.endpoint("add_badge").flag("write_op"))
            })
        })
        .service("tabs", |svc| {
            svc.read(["list", "list_items"]).write(["insert", "delete"])
        })
        .service("uploads", |svc| {
            // `Args` carries the byte stream, so it has no serde form.
            svc.read(["list_pending"])
                .write(["start", "abort"])
                .method_with("finish", |m| m.flag("stream").flag("write_op"))
        })
        .export(args, true)
        .into()
}
