use axum::{
    extract::ws::{Message, WebSocket},
    response::Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{game_server::GameServerHandle, SharedState};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WsMsg {
    Ping,
    Close,
    Text(String),
}

pub async fn handle_socket(
    socket: WebSocket,
    server_tx: Extension<GameServerHandle>,
    game_id: String,
    player_id: Uuid,
    state: SharedState,
) {
    let game_id_clone = game_id.clone();
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();
    let client_tx_clone = client_tx.clone();

    server_tx
        .handle_connect(client_tx_clone, game_id, player_id, state.clone())
        .await;

    let (mut sender, mut receiver) = socket.split();

    tokio::select! {
            _handle_read = async {
        loop {
            if let Some(msg) = receiver.next().await {
                println!("{:?}", msg);
            // if let Some(Ok(Message::Text(text)))  = receiver.next().await {
                let game_id_clone_2 = game_id_clone.clone();
                if let Ok(msg) = msg {
                // if let Ok(msg) =  serde_json::from_str(text.as_str()) {
                    match msg {
                //         WsMsg::Ping => {}
                //         WsMsg::Close => {}
                //         WsMsg::Text(msg) => {}
                        Message::Text(msg) => {
                            // println!("{:?}", msg);
                            if msg.contains("close") {

                                server_tx.handle_disconnect(player_id, game_id_clone_2);
                            } else if msg.contains("ping"){
                                //temp handle ping

                            } else {
                            server_tx.handle_send(player_id, msg, game_id_clone_2).await}
                        }
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
                            server_tx.handle_disconnect(player_id, game_id_clone_2);
                            return;
                        }
                    }
                } else {
                    server_tx.handle_disconnect(player_id, game_id_clone);
                    return;
                }
            }
        }

    } => {}
        _handle_write = async {
        loop {
            if let Some(message) = client_rx.recv().await {
                println!("{:?}", message);
                sender.send(Message::Text(message)).await.unwrap();

            } else {
                return anyhow::Ok(())
            }

        }

    } => {}
        };
}
