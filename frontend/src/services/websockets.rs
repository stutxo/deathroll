use futures::channel::mpsc::Sender;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::{futures::WebSocket, Message};
use std::time::Duration;
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew_agent::Dispatched;

use crate::components::multiplayer::GameMessage;
use crate::services::feed_bus::{FeedBus, Request};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WsMsg {
    Ping,
    Close,
    Roll,
}

#[derive(Clone, Debug)]
pub struct WebsocketService {
    pub tx: Sender<String>,
}
impl WebsocketService {
    pub fn ws_connect(full_url: &str) -> Self {
        let mut event_bus = FeedBus::dispatcher();
        let ws = WebSocket::open(full_url).unwrap();

        let (game_tx, mut game_rx) = futures::channel::mpsc::channel::<String>(1000);

        let (mut write, mut read) = ws.split();

        spawn_local(async move {
            while let Some(message) = game_rx.next().await {
                log::debug!("{:?}", message);
                write.send(Message::Text(message)).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(result) = read.next().await {
                log::debug!("{:?}", result);
                match result {
                    Ok(Message::Text(msg)) => {
                        event_bus.send(Request::EventBusMsg(msg));
                    }
                    Ok(Message::Bytes(_)) => {}

                    Err(e) => match e {
                        WebSocketError::ConnectionError => {}
                        WebSocketError::ConnectionClose(_) => {
                            event_bus.send(Request::EventBusMsg(
                                serde_json::to_string(&GameMessage::Disconnect).unwrap(),
                            ));
                            log::debug!("websocket closed");
                        }
                        WebSocketError::MessageSendError(_) => {}
                        _ => {}
                    },
                }
            }
        });
        let mut game_tx_clone = game_tx.clone();
        spawn_local(async move {
            loop {
                game_tx_clone
                    .try_send(serde_json::to_string(&WsMsg::Ping).unwrap())
                    .unwrap();

                sleep(Duration::from_secs(40)).await;
            }
        });
        Self { tx: game_tx }
    }
    pub async fn close(&mut self) {
        self.tx.close().await.unwrap();
    }
}
