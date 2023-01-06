use futures::channel::mpsc::Sender;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::{futures::WebSocket, Message};
use yew::platform::spawn_local;
use yew_agent::Dispatched;

use crate::services::feed_bus::{FeedBus, Request};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WsMsg {
    Ping,
    Close,
}

#[derive(Clone, Debug)]
pub struct WebsocketService {
    pub tx: Sender<String>,
}
impl WebsocketService {
    pub fn ws_connect(full_url: &String) -> Self {
        let mut event_bus = FeedBus::dispatcher();
        let ws = WebSocket::open(&full_url).unwrap();
       
        let (game_tx, mut game_rx) = futures::channel::mpsc::channel::<String>(1000);

        let (mut write, mut read) = ws.split();

        spawn_local(async move {
            while let Some(message) = game_rx.next().await {
                write.send(Message::Text(message)).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(result) = read.next().await {
                match result {
                    Ok(Message::Text(result)) => {
                        event_bus.send(Request::EventBusMsg(result));
                    }
                    Ok(Message::Bytes(_)) => {}

                    Err(e) => match e {
                        WebSocketError::ConnectionError => {}
                        WebSocketError::ConnectionClose(_) => {
                            event_bus.send(Request::EventBusMsg("disconnect".to_string()));
                            log::debug!("websocket closed");
                        }
                        WebSocketError::MessageSendError(_) => {}
                        _ => {}
                    },
                }
            }
        });

        Self { tx: game_tx }
    }
    pub async fn close(&mut self) {
        self.tx.close().await.unwrap();
    }
}
