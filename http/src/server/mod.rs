use eva::{component_configs::ComponentConfig, logging as log, supervisor::SlaveRx};

use eyre::Context;
use viendesu_core::service::IsService;

use tokio::net;

pub use self::config::Config;

pub mod config;
pub mod rpc;

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

/// `POST /rpc`, `POST /uploads/{id}` and whatever `mount` adds (e.g. `/mcp`),
/// all behind the tracing and CORS layers — routes added after `.layer()`
/// would not get them.
pub fn make_router<T: Types>(
    service: T::Service,
    mount: impl FnOnce(axum::Router) -> axum::Router,
) -> axum::Router {
    use tower_http::cors;

    mount(rpc::router::<T>(service))
        .layer(fastrace_axum::FastraceLayer)
        .layer(cors::CorsLayer::very_permissive().max_age(CORS_MAX_AGE))
}
