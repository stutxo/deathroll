use axum::{
    extract::ws::{Message, WebSocket},
    response::Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;

use crate::game_server::GameServerHandle;

pub async fn handle_socket(
    socket: WebSocket,
    server_tx: Extension<GameServerHandle>,
    game_id: String,
) {
    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();
    let player_id = server_tx.handle_connect(conn_tx, game_id).await;

    let (mut sender, mut receiver) = socket.split();

    tokio::select! {
            _handle_read = async {
        loop {
            if let Some(msg) = receiver.next().await {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Text(msg) => server_tx.handle_send(player_id, msg).await,
                        Message::Binary(_) => {
                            println!("client sent binary data");
                        }
                        Message::Ping(_) => {
                            println!("socket ping");
                        }
                        Message::Pong(_) => {
                            println!("socket pong");
                        }
                        Message::Close(_) => {
                            server_tx.handle_disconnect(player_id);
                            return;
                        }
                    }
                } else {
                    server_tx.handle_disconnect(player_id);
                    return;
                }
            }
        }

    } => {}
        _handle_write = async {
        loop {
            if let Some(message) = conn_rx.recv().await {
                sender.send(Message::Text(message)).await.unwrap();
            } else {
                return anyhow::Ok(())
            }

        }

    } => {}
        };
}
