use futures::channel::mpsc::Sender;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::{futures::WebSocket, Message};
use yew::platform::spawn_local;
use yew_agent::Dispatched;

use crate::feed_bus::{FeedBus, Request};

#[derive(Clone, Debug)]
pub struct WebsocketService {
    pub tx: Sender<String>,
}
impl WebsocketService {
    pub fn ws_connect() -> Self {
        let location = web_sys::window().unwrap().location();

        let host = location.host().unwrap();
        let protocol = location.protocol().unwrap();
        let ws_protocol = match protocol.as_str() {
            "https:" => "wss:",
            _ => "ws:",
        };
        let url = location.href().unwrap();
        let url_split: Vec<&str> = url.split('/').collect();
        let game_id = url_split[3];

        let full_url = format!("{}//{}/ws/{}", ws_protocol, host, game_id);

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
            while let Some(result) = read.next().await {                match result {
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
