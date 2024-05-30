use axum::{routing::get, Router};
use futures::{FutureExt, SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::{frame::coding::CloseCode, CloseFrame, Message},
};

const SERVER: &str = "ws://127.0.0.1:5001/";

async fn hello() -> &'static str {
    "TODO: Dashboard"
}

pub(crate) fn router() -> Router {
    Router::new().route("/", get(hello))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).with_line_number(true).init();

    let app = router();
    let listener = TcpListener::bind("127.0.0.1:3002").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        println!("Usage: rgrok [port]");
        std::process::exit(-1);
    }

    let port = &args[0];

    // Make request to server, grab back URL
    let client = reqwest::Client::new();

    let tunnel = tokio::spawn(spawn_tunnel(port.parse::<u16>().unwrap()));

    loop {}

    Ok(())
}

async fn spawn_tunnel(port: u16) {
    let ws_stream = match connect_async(SERVER).await {
        Ok((stream, response)) => {
            println!("Tunnel ready to be open, server response was {response:?}");
            stream
        }
        Err(e) => {
            println!("Unable to open tunnel: {e}!");
            return;
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    // Ping the server initially
    sender
        .send(Message::Text(
            json!({
                "port": port.to_string()
            })
            .to_string(),
        ))
        .await
        .expect("Can not send!");

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(t) => {
                    println!("Tunnel opened at {}, ready for pings", t);
                },
                _ => todo!()
            }
        }
    });
}
