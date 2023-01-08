use axum::{
    extract::ws::{Message, WebSocket},
    response::Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    game_server::{GameMessage, GameServerHandle},
    SharedState,
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WsMsg {
    Ping,
    Close,
    Roll,
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
    let client_tx_clone_2 = client_tx.clone();

    server_tx
        .handle_connect(client_tx_clone, game_id, player_id, state.clone())
        .await;

    let (mut sender, mut receiver) = socket.split();

    tokio::select! {
            _handle_read = async {

            while let Some(Ok(Message::Text(text)))  = receiver.next().await {
                let game_id_clone_2 = game_id_clone.clone();

                if let Ok(msg) =  serde_json::from_str(text.as_str()) {
                    match msg {
                        WsMsg::Ping => {client_tx_clone_2.send(serde_json::to_string(&GameMessage::Pong).unwrap()).unwrap()}
                        WsMsg::Close => {server_tx.handle_disconnect(player_id, game_id_clone_2)}
                        WsMsg::Roll => {println!("{:?}", text); server_tx.handle_send(player_id, game_id_clone_2).await}
                    }
                } else {
                    server_tx.handle_disconnect(player_id, game_id_clone);
                    return;
                }
            }


    } => {}
        _handle_write = async {
            while let Some(message) = client_rx.recv().await {
                println!("{:?}", message);
                sender.send(Message::Text(message)).await.unwrap();

            }
    } => {}
        };
}
