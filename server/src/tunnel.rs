use axum::extract::connect_info::ConnectInfo;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Form, Json, Router,
};
use nanoid::nanoid;
use redis::RedisError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;

use crate::errors::ApiError;

#[derive(Debug, Deserialize)]
pub struct TunnelOpts {
    port: String,
}

impl TunnelOpts {
    fn grab_port(&self) -> u16 {
        self.port.parse::<u16>().unwrap()
    }
}

#[derive(Debug)]
enum TunnelOptErrors {
    RedisError(RedisError),
    SerdeError(serde_json::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tunnel {
    uuid: String,
    port: u16,
}

fn uuid() -> String {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
    ];
    nanoid!(10, &alphabet)
}

impl Tunnel {
    fn default() -> Self {
        Self {
            uuid: String::from("test"),
            port: 0,
        }
    }

    fn new(port: u16) -> Result<Self, TunnelOptErrors> {
        let client_result = redis::Client::open("redis://127.0.0.1/");
        match client_result {
            Ok(client) => {
                let mut con_result = client.get_connection();
                match con_result {
                    Ok(mut con) => {
                        let tunnel = Self {
                            uuid: String::from("test"),
                            port,
                        };
                        let json = json!({
                            "uuid": &tunnel.uuid,
                            "port": port
                        });
                        redis::cmd("SET")
                            .arg(&tunnel.uuid)
                            .arg(json.to_string())
                            .execute(&mut con);
                        return Ok(tunnel);
                    }
                    Err(err) => return Err(TunnelOptErrors::RedisError(err)),
                }
            }
            Err(err) => return Err(TunnelOptErrors::RedisError(err)),
        }
    }

    fn update_port(&mut self, port: u16) {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();
        self.port = port;
        let json = json!({
            "uuid": &self.uuid,
            "port": port
        });
        redis::cmd("SET")
            .arg(&self.uuid)
            .arg(json.to_string())
            .execute(&mut con);
    }
}

// pub async fn tunnel(Form(opts): Form<TunnelOpts>) -> impl IntoResponse {
//     let tunnel_result = Tunnel::new(opts.port);
//     match tunnel_result {
//         Ok(tunnel) => {
//             let json = json!({
//                 "uuid": &tunnel.uuid,
//                 "port": &tunnel.port.to_string()
//             });
//             Json(json).into_response()
//         }
//         _ => panic!("TODO: Figure out error handling"),
//     }
// }

pub async fn ws_tunnel(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    // Receive initial confirmation with port info before doing anything further
    let mut tunnel = Tunnel::default();

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    let opts: TunnelOpts = serde_json::from_str(&t).unwrap();
                    if opts.grab_port() != tunnel.port {
                        tunnel.update_port(opts.grab_port());
                        if socket
                            .send(Message::Text(format!("{}.braindump.localhost", tunnel.uuid)))
                            .await
                            .is_ok()
                        {
                            tracing::info!("Tunnel opened to http://{}:{}", who, tunnel.port);
                        }
                    }
                }
                _ => {}
            }
        } else {
            tracing::info!("Tunnel to {who} abruptly disconnected");
            return;
        }
    }
}
