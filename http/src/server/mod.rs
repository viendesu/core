use eva::{
    component_configs::ComponentConfig, logging as log, perfect_derive, supervisor::SlaveRx,
};

use eyre::Context;
use viendesu_core::{
    errors::AuxResult,
    service::{IsService, Session, SessionMaker, SessionOf},
};

use tokio::net;

pub use self::config::Config;

pub mod config;

mod context;
mod handler;
mod request;
mod response;

mod routes;

/// 24 hours
const CORS_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(86400);

pub trait Types: Send + Sync + 'static {
    type Service: IsService + Clone;
}

pub async fn serve(
    rx: SlaveRx,
    config: ComponentConfig<Config>,
    router: axum::Router,
) -> eyre::Result<()> {
    // TODO: use it.
    _ = rx;
    let config::Config {
        unencrypted,
        ssl: _,
    } = &*config;
    let unencrypted = unencrypted
        .as_ref()
        .expect("SSL-only currently is not supported");

    if !unencrypted.enable {
        return Ok(());
    }

    let listener = net::TcpListener::bind(unencrypted.listen)
        .await
        .wrap_err("failed to bind address")?;

    log::info!(at:% = listener.local_addr().unwrap(); "started HTTP server");

    axum::serve(listener, router)
        .await
        .wrap_err("failed to serve")?;

    Ok(())
}

pub fn make_router<T: Types>(service: T::Service) -> axum::Router {
    use tower_http::cors;

    let scope = handler::RouterScope::root(State::<T> { service });
    routes::make(scope)
        .into_axum()
        .layer(fastrace_axum::FastraceLayer)
        .layer(cors::CorsLayer::very_permissive().max_age(CORS_MAX_AGE))
}

#[perfect_derive(Clone)]
struct State<T: Types> {
    service: T::Service,
}

impl<T: Types> State<T> {
    async fn make_session(&self) -> AuxResult<Session<SessionOf<T::Service>>> {
        Ok(self.service.make_session().await?)
    }
}
