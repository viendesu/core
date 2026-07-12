//! Built-in tool sets over the service domains.
//!
//! Every tool is a thin adapter from MCP arguments to the corresponding
//! protocol request; extend with [`Tools::tool`] / [`Tools::merge`].

use crate::registry::Tools;

use viendesu_core::service::{
    CallStep as _, IsSession, Session,
    articles::Articles as _,
    authors::Authors as _,
    blogs::Blogs as _,
    boards::Boards as _,
    games::Games as _,
    marks::{Badges as _, Genres as _, Tags as _},
    messages::Messages as _,
    tabs::Tabs as _,
    threads::Threads as _,
    users::Users as _,
};
use viendesu_protocol::requests::{
    articles, authors, blogs, boards, games, marks, messages, tabs, threads, users,
};

/// Read-only tools over all queryable domains.
pub fn read_only<S: IsSession + 'static>() -> Tools<S> {
    Tools::new()
        .tool(
            "whoami",
            "Get the id and role of the currently authenticated user. Requires authentication.",
            |mut s: Session<S>, args: users::check_auth::Args| async move {
                s.users().check_auth().call(args).await
            },
        )
        .tool(
            "get_user",
            "Get a user by selector. Omit `user` to get the currently authenticated one.",
            |mut s: Session<S>, args: users::get::Args| async move {
                s.users().get().call(args).await
            },
        )
        .tool(
            "search_users",
            "Search users by text query, paginated via `start_from` + `limit`.",
            |mut s: Session<S>, args: users::search::Args| async move {
                s.users().search().call(args).await
            },
        )
        .tool(
            "get_author",
            "Get a game author (developer/circle) by selector.",
            |mut s: Session<S>, args: authors::get::Args| async move {
                s.authors().get().call(args).await
            },
        )
        .tool(
            "search_authors",
            "Search game authors by text query, optionally restricted to authors owned by a user.",
            |mut s: Session<S>, args: authors::search::Args| async move {
                s.authors().search().call(args).await
            },
        )
        .tool(
            "get_game",
            "Get a game by selector (id or author + slug). \
             Set `resolve_marks` to also resolve tag/genre/badge names, \
             `latest_articles` to also fetch the latest articles of the \
             game's blog.",
            |mut s: Session<S>, args: games::get::Args| async move {
                s.games().get().call(args).await
            },
        )
        .tool(
            "search_games",
            "Search the games catalog. The text query matches game titles only; \
             for theme/genre requests use include.genres_any with genre slugs \
             (listed in the server instructions, or via list_genres). Also \
             supports an author filter, sorting and pagination.",
            |mut s: Session<S>, args: games::search::Args| async move {
                s.games().search().call(args).await
            },
        )
        .tool(
            "get_board",
            "Get a forum board by selector.",
            |mut s: Session<S>, args: boards::get::Args| async move {
                s.boards().get().call(args).await
            },
        )
        .tool(
            "search_threads",
            "List forum threads, paginated via `after` + `limit`.",
            |mut s: Session<S>, args: threads::search::Args| async move {
                s.threads().search().call(args).await
            },
        )
        .tool(
            "get_thread",
            "Get a forum thread by selector.",
            |mut s: Session<S>, args: threads::get::Args| async move {
                s.threads().get().call(args).await
            },
        )
        .tool(
            "get_message",
            "Get a forum message by selector.",
            |mut s: Session<S>, args: messages::get::Args| async move {
                s.messages().get().call(args).await
            },
        )
        .tool(
            "get_blog",
            "Get a blog by id. A blog's id is the id of the entity that authors \
             it (user, author or game); the response resolves that owner.",
            |mut s: Session<S>, args: blogs::get::Args| async move {
                s.blogs().get().call(args).await
            },
        )
        .tool(
            "get_article",
            "Get a blog article by id. The content is CommonMark.",
            |mut s: Session<S>, args: articles::get::Args| async move {
                s.articles().get().call(args).await
            },
        )
        .tool(
            "list_articles",
            "List articles of a blog, newest first, paginated via `before` + `limit`.",
            |mut s: Session<S>, args: articles::search::Args| async move {
                s.articles().search().call(args).await
            },
        )
        .tool(
            "list_tags",
            "List known game tags, optionally filtered by text query.",
            |mut s: Session<S>, args: marks::list_tags::Args| async move {
                s.tags().list().call(args).await
            },
        )
        .tool(
            "list_badges",
            "List known game badges, optionally filtered by text query.",
            |mut s: Session<S>, args: marks::list_badges::Args| async move {
                s.badges().list().call(args).await
            },
        )
        .tool(
            "list_genres",
            "List all game genres.",
            |mut s: Session<S>, args: marks::list_genres::Args| async move {
                s.genres().list().call(args).await
            },
        )
        .tool(
            "list_tabs",
            "List profile tabs of a user.",
            |mut s: Session<S>, args: tabs::list::Args| async move {
                s.tabs().list().call(args).await
            },
        )
        .tool(
            "list_tab_items",
            "List items (games or authors) of a user's profile tab.",
            |mut s: Session<S>, args: tabs::list_items::Args| async move {
                s.tabs().list_items().call(args).await
            },
        )
}

/// Content management tools: create/update/delete over games, authors,
/// boards and users. Authorization is enforced by the service, so these
/// require an authenticated session with a sufficient role.
pub fn management<S: IsSession + 'static>() -> Tools<S> {
    Tools::new()
        .tool(
            "create_game",
            "Add a game to the catalog. Requires authentication; `author` must be \
             owned by the current user unless it has a moderation role.",
            |mut s: Session<S>, args: games::create::Args| async move {
                s.games().create().call(args).await
            },
        )
        .tool(
            "update_game",
            "Update a game by id. All `update` fields are optional patches: \
             omitted fields are kept as is.",
            |mut s: Session<S>, args: games::update::Args| async move {
                s.games().update().call(args).await
            },
        )
        .tool(
            "create_author",
            "Create a game author (developer/circle). Omit `owner` to own it yourself; \
             creating authors for other users requires at least the admin role.",
            |mut s: Session<S>, args: authors::create::Args| async move {
                s.authors().create().call(args).await
            },
        )
        .tool(
            "update_author",
            "Update an author by selector. All `update` fields are optional patches: \
             omitted fields are kept as is.",
            |mut s: Session<S>, args: authors::update::Args| async move {
                s.authors().update().call(args).await
            },
        )
        .tool(
            "create_board",
            "Create a forum board with an initial message.",
            |mut s: Session<S>, args: boards::create::Args| async move {
                s.boards().create().call(args).await
            },
        )
        .tool(
            "edit_board",
            "Edit a forum board: text and slug are optional patches, \
             omitted fields are kept as is.",
            |mut s: Session<S>, args: boards::edit::Args| async move {
                s.boards().edit().call(args).await
            },
        )
        .tool(
            "delete_board",
            "Delete a forum board.",
            |mut s: Session<S>, args: boards::delete::Args| async move {
                s.boards().delete().call(args).await
            },
        )
        .tool(
            "edit_blog",
            "Edit a blog, selected by the id of the owning entity. All fields are \
             optional patches, omitted fields are kept as is; patching every field \
             to null deletes the blog metadata entirely.",
            |mut s: Session<S>, args: blogs::edit::Args| async move {
                s.blogs().edit().call(args).await
            },
        )
        .tool(
            "create_article",
            "Publish an article to a blog. `content` is CommonMark. Requires \
             authentication and ownership of the blog's entity.",
            |mut s: Session<S>, args: articles::create::Args| async move {
                s.articles().create().call(args).await
            },
        )
        .tool(
            "edit_article",
            "Edit a blog article: title and content are optional patches, \
             omitted fields are kept as is.",
            |mut s: Session<S>, args: articles::edit::Args| async move {
                s.articles().edit().call(args).await
            },
        )
        .tool(
            "delete_article",
            "Delete a blog article.",
            |mut s: Session<S>, args: articles::delete::Args| async move {
                s.articles().delete().call(args).await
            },
        )
        .tool(
            "update_user",
            "Update a user profile. Omit `user` to update the currently authenticated one; \
             updating others requires a moderation role. All `update` fields are optional \
             patches: omitted fields are kept as is.",
            |mut s: Session<S>, args: users::update::Args| async move {
                s.users().update().call(args).await
            },
        )
}

/// Forum write tools. Authorization is enforced by the service,
/// so these require an authenticated session.
pub fn forum_posting<S: IsSession + 'static>() -> Tools<S> {
    Tools::new()
        .tool(
            "create_thread",
            "Create a forum thread on a board with an initial message. Requires authentication.",
            |mut s: Session<S>, args: threads::create::Args| async move {
                s.threads().create().call(args).await
            },
        )
        .tool(
            "post_message",
            "Post a message to a forum thread. Requires authentication.",
            |mut s: Session<S>, args: messages::post::Args| async move {
                s.messages().post().call(args).await
            },
        )
        .tool(
            "edit_message",
            "Edit a forum message. Requires authentication and ownership.",
            |mut s: Session<S>, args: messages::edit::Args| async move {
                s.messages().edit().call(args).await
            },
        )
}
