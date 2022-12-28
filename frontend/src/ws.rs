use futures::{SinkExt, StreamExt};
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_utils::errors::JsError;
use yew::platform::pinned::mpsc::{self, UnboundedReceiver, UnboundedSender};
use yew::platform::spawn_local;
use yew_agent::Dispatched;

use crate::chat_bus::{ChatBus, Request};

pub fn ws_connect(full_url: String) -> Result<UnboundedSender<Message>, JsError> {
    let mut event_bus = ChatBus::dispatcher();
    let ws = WebSocket::open(&full_url);
    match ws {
        Ok(ws) => {
            let (game_tx, mut game_rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
                mpsc::unbounded();

            let (mut write, mut read) = ws.split();

            spawn_local(async move {
                while let Some(message) = game_rx.next().await {
                    write.send(message).await.unwrap();
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
                            WebSocketError::ConnectionError => {
                                event_bus.send(Request::EventBusMsg("disconnected".to_string()));
                                log::debug!("Websocket error {:?}", e);
                            }
                            WebSocketError::ConnectionClose(e) => {
                                event_bus.send(Request::EventBusMsg("disconnected".to_string()));
                                log::debug!("Websocket error {:?}", e);
                            }
                            WebSocketError::MessageSendError(e) => {
                                event_bus.send(Request::EventBusMsg("disconnected".to_string()));
                                log::debug!("Websocket error {:?}", e);
                            }
                            _ => {
                                event_bus.send(Request::EventBusMsg("disconnected".to_string()));
                                log::debug!("Unexpected webscocket error")
                            }
                        },
                    }
                }
                event_bus.send(Request::EventBusMsg("disconnected".to_string()));
                log::debug!("websocket closed")
            });
            Ok(game_tx)
        }
        Err(e) => Err(e),
    }
}
