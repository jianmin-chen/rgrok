use axum::{extract::{Request, Host}, body::Body, routing::any};
use anyhow::Result;
use axum::{extract::ConnectInfo, routing::get, routing::post, Router};
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceExt;

mod errors;
mod tunnel;

async fn dashboard(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> &'static str {
    println!("{addr}");
    "TODO: Some sort of dashboard?"
}

pub(crate) fn router() -> Router {
    let tunnel_router = Router::new().route("/", get(tunnel::ws_tunnel));
    // How the heck does the router for a tunnel work
    Router::new().route("/*path", any(|Host(hostname): Host, request: Request<Body>| async move {
        let is_subdomain: Vec<_> = hostname.match_indices(".").map(|(i, _)| i).collect();
        if is_subdomain.len() == 1 {
            // Open tunnel
            tunnel_router.oneshot(request).await;
        } else {
            // Ping to tunnel
            dbg!("ping to tunnel");
        }
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_line_number(true)
        .init();

    let app = router();
    let listener = TcpListener::bind("127.0.0.1:5001").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tracing::info!("App running on http://{}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
