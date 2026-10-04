use proc_macro::TokenStream;

use eva_api_builder::{ApiBuilder, MethodBuilder, ServiceBuilder, parse_quote};

trait Access {
    fn read_with(self, name: &str, f: impl FnOnce(MethodBuilder) -> MethodBuilder) -> Self;

    /// A method that changes state, flagged `write_op` for rate limiting.
    fn write_with(self, name: &str, f: impl FnOnce(MethodBuilder) -> MethodBuilder) -> Self;
}

impl Access for ServiceBuilder {
    fn read_with(self, name: &str, f: impl FnOnce(MethodBuilder) -> MethodBuilder) -> Self {
        self.method_with(name, f)
    }

    fn write_with(self, name: &str, f: impl FnOnce(MethodBuilder) -> MethodBuilder) -> Self {
        self.method_with(name, |m| f(m.flag("write_op")))
    }
}

/// Defines the service traits and the endpoint list X-macro named by the input,
/// in the grammar documented by `eva_api_builder`.
#[proc_macro]
pub fn endpoints(args: TokenStream) -> TokenStream {
    ApiBuilder::new("::viendesu_protocol::requests")
        .result_type(|ok, err| parse_quote!(::viendesu_protocol::requests::Response<#ok, #err>))
        .service("users", |svc| {
            svc.doc("Accounts, sessions and profiles.")
                .read_with("get", |m| m.doc("Gets a user; `None` selects the caller."))
                .read_with("check_auth", |m| m.doc("Id and role of the caller."))
                .read_with("search", |m| {
                    m.doc("Searches users by text query, paginated via `start_from`.")
                })
                .write_with("begin_auth", |m| {
                    m.doc(
                        "
                        Begins an auth session for an authored method, e.g. a Telegram bot.

                        The returned id is a secret capability: only the caller may redeem
                        it via `finish_auth`, and only before it expires.
                        ",
                    )
                })
                .write_with("finish_auth", |m| {
                    m.doc("Redeems an auth session begun by the caller for a session token.")
                })
                .write_with("sign_in", |m| {
                    m.doc("Exchanges nickname and password for a session token.")
                })
                .write_with("sign_up", |m| {
                    m.doc("Registers a user; signing in requires `confirm_sign_up` first.")
                })
                .write_with("update", |m| {
                    m.doc(
                        "
                        Patches a user; `None` selects the caller.

                        Updating others requires a moderation role.
                        ",
                    )
                })
                .write_with("confirm_sign_up", |m| {
                    m.doc("Completes a pending sign-up with its completion token.")
                })
        })
        .service("authors", |svc| {
            svc.doc("Game authors: developers and circles.")
                .read_with("get", |m| m.doc("Gets an author."))
                .read_with("search", |m| {
                    m.doc(
                        "
                        Searches authors by text query, optionally owned by a user.

                        Paginated via `start_from`.
                        ",
                    )
                })
                .write_with("create", |m| m.doc("Creates an author."))
                .write_with("update", |m| m.doc("Patches an author."))
        })
        .service("games", |svc| {
            svc.doc("The game catalog.")
                .read_with("get", |m| m.doc("Gets a game."))
                .read_with("search", |m| {
                    m.doc(
                        "
                        Searches the catalog; the text query matches titles only.

                        Paginated via the `after` key of `sort_by`.
                        ",
                    )
                })
                .write_with("create", |m| {
                    m.doc(
                        "
                        Adds a game.

                        `author` must be owned by the caller unless it has a moderation role.
                        ",
                    )
                })
                .write_with("update", |m| m.doc("Patches a game."))
                .write_with("rate", |m| {
                    m.doc("Sets the caller's rating, replacing the previous one.")
                })
        })
        .service("boards", |svc| {
            svc.doc("Forum boards.")
                .read_with("get", |m| m.doc("Gets a board."))
                .write_with("create", |m| {
                    m.doc("Creates a board with its initial message.")
                })
                .write_with("delete", |m| m.doc("Deletes a board."))
                .write_with("edit", |m| m.doc("Patches a board."))
        })
        .service("threads", |svc| {
            svc.doc("Forum threads.")
                .read_with("get", |m| m.doc("Gets a thread."))
                .read_with("search", |m| m.doc("Lists threads, paginated via `after`."))
                .write_with("delete", |m| m.doc("Deletes a thread."))
                .write_with("edit", |m| m.doc("Patches a thread; owner only."))
                .write_with("create", |m| {
                    m.doc("Opens a thread on a board with its initial message.")
                })
        })
        .service("messages", |svc| {
            svc.doc("Forum thread messages.")
                .read_with("get", |m| m.doc("Gets a message."))
                .write_with("post", |m| m.doc("Posts a message to a thread."))
                .write_with("delete", |m| m.doc("Deletes a message."))
                .write_with("edit", |m| m.doc("Replaces a message's text; owner only."))
        })
        .service("blogs", |svc| {
            svc.doc("Blogs: a facet of a user, author or game, keyed by its id.")
                .read_with("get", |m| m.doc("Gets a blog with its resolved owner."))
                .write_with("edit", |m| {
                    m.doc(
                        "
                        Patches blog metadata; owner only.

                        Patching every field to `None` deletes the metadata.
                        ",
                    )
                })
        })
        .service("articles", |svc| {
            svc.doc("Blog articles; content is CommonMark.")
                .read_with("get", |m| m.doc("Gets an article."))
                .read_with("search", |m| {
                    m.doc("Lists articles of a blog, newest first, paginated via `before`.")
                })
                .write_with("create", |m| {
                    m.doc("Publishes an article; requires owning the blog.")
                })
                .write_with("delete", |m| m.doc("Deletes an article; owner only."))
                .write_with("edit", |m| m.doc("Patches an article; owner only."))
        })
        .service("comments", |svc| {
            svc.doc("Game comments, threaded one level deep.")
                .read_with("list", |m| {
                    m.doc(
                        "
                        Lists top-level comments of a game.

                        Paginated via `before`, plus `before_likes` for `Sort::Top`.
                        ",
                    )
                })
                .read_with("replies", |m| {
                    m.doc("Lists replies to a top-level comment, paginated via `after`.")
                })
                .write_with("create", |m| {
                    m.doc("Comments on a game, or replies to a top-level comment via `parent`.")
                })
                .write_with("edit", |m| m.doc("Replaces a comment's text; owner only."))
                .write_with("delete", |m| {
                    m.doc(
                        "
                        Deletes a comment; owner only.

                        A top-level comment with replies is tombstoned instead: its text
                        and author are wiped, the replies stay.
                        ",
                    )
                })
                .write_with("like", |m| m.doc("Likes a comment or retracts the like."))
        })
        .service("tags", |svc| {
            svc.doc("Game tags.")
                .read_with("list", |m| {
                    m.doc("Lists tags matching the query, all without one.")
                })
                .write_with("add", |m| m.doc("Registers a tag."))
        })
        .service("genres", |svc| {
            svc.doc("Game genres.")
                .read_with("list", |m| m.doc("Lists all genres."))
        })
        .service("badges", |svc| {
            svc.doc("Game badges.")
                .read_with("list", |m| {
                    m.doc("Lists badges matching the query, all without one.")
                })
                .write_with("add", |m| m.doc("Registers a badge."))
        })
        .service("tabs", |svc| {
            svc.doc("User profile tabs: named lists of games or authors.")
                .read_with("list", |m| m.doc("Lists tabs of a user."))
                .read_with("list_items", |m| {
                    m.doc("Lists items of a tab, paginated via `start_from`.")
                })
                .write_with("insert", |m| m.doc("Adds an item to a tab."))
                .write_with("delete", |m| m.doc("Removes an item from a tab."))
        })
        .service("uploads", |svc| {
            svc.doc("File uploads: `start` reserves one, `finish` streams the bytes.")
                .read_with("list_pending", |m| {
                    m.doc("Lists the caller's unfinished uploads.")
                })
                .write_with("start", |m| {
                    m.doc("Reserves an upload against the per-class byte quotas.")
                })
                .write_with("abort", |m| m.doc("Cancels an unfinished upload."))
                // `Args` carries the byte stream, so it has no serde form.
                .write_with("finish", |m| {
                    m.flag("stream").doc(
                        "
                        Streams the file contents of an upload.

                        The upload is consumed whatever the outcome; a failed one is
                        started anew.
                        ",
                    )
                })
        })
        .export(args, true)
        .into()
}
