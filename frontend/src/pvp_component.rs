use crate::chat_bus::{ChatBus, Request};
use crate::routes::Route;
use regex::Regex;

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::{futures::WebSocket, Message};
use std::rc::Rc;

use web_sys::window;
use web_sys::{Element, MouseEvent};
use yew::platform::pinned::mpsc::{self, UnboundedReceiver, UnboundedSender};
use yew::platform::spawn_local;
use yew_agent::{Bridge, Bridged, Dispatched};
use yew_router::prelude::*;

use yew::{html, Callback, Component, Html, NodeRef};

pub enum Msg {
    Roll,
    HandleMsg(String),
}

pub struct PvPComponent {
    node_ref: NodeRef,
    tx: UnboundedSender<Message>,
    feed: Vec<String>,
    _producer: Box<dyn Bridge<ChatBus>>,
    start_roll: String,
    status_msg: String,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let node_ref = self.node_ref.clone();

        spawn_local(async move {
            let chat_main = node_ref.cast::<Element>().unwrap();

            chat_main.set_scroll_top(chat_main.scroll_height());
        })
    }
}

impl Component for PvPComponent {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &yew::Context<Self>) -> Self {
        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        let url_split: Vec<&str> = url.split('/').collect();

        let start = "ws://".to_owned();
        let host = url_split[2];
        let ws = "/ws/";
        let game_id = url_split[3];
        let roll_amount = url_split[4];

        let full_url = start + host + ws + game_id;

        let (game_tx, mut game_rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            mpsc::unbounded();

        let ws = WebSocket::open(&full_url).unwrap();
        let (mut write, mut read) = ws.split();

        let mut event_bus = ChatBus::dispatcher();

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
                            log::debug!("Websocket error {:?}", e);
                        }
                        WebSocketError::ConnectionClose(e) => {
                            log::debug!("Websocket error {:?}", e);
                        }
                        WebSocketError::MessageSendError(e) => {
                            log::debug!("Websocket error {:?}", e);
                        }
                        _ => {
                            log::debug!("Unexpected webscocket error")
                        }
                    },
                }
            }
            log::debug!("websocket closed")
        });

        game_tx
            .send_now(Message::Text(String::from(roll_amount)))
            .unwrap();

        let cb = {
            let link = ctx.link().clone();
            move |msg| link.send_message(Msg::HandleMsg(msg))
        };

        Self {
            node_ref: NodeRef::default(),
            tx: game_tx,
            feed: Vec::new(),
            _producer: ChatBus::bridge(Rc::new(cb)),
            start_roll: roll_amount.to_string(),
            status_msg: "".to_string(),
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        html! {
          <body>
          <div class="app-body">
            <header class="header">
              <div>
                <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
                <h1>{"1v1 me bruv"}</h1>
                {"To invite someone to play, give this URL: "}
                <br/>
                {url}
              </div>
            </header>
            <div class="msger">
              <main class="msger-chat" id="chat-main" ref={self.node_ref.clone()}>
                <div class="dets">
                 {"start roll: "}{&self.start_roll}
                  {
                    self.feed.clone().into_iter().map(|name| {
                      html!{
                          //key={name.clone()} fix for ios not working here in pvp
                        <div class="msg" >
                          {" "}{name}
                        </div>
                      }
                    }).collect::<Html>()
                  }
                </div>
              </main>
            </div>
            <div>
              <button onclick={on_click} class="roll-button">{roll_emoji}
              <div>{&self.status_msg}</div></button>
            </div>
          </div>
        </body>
              }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                let roll = "rolling".to_string();
                self.tx.send_now(Message::Text(String::from(roll))).unwrap();

                self.scroll_top();

                true
            }
            Msg::HandleMsg(result) => {
                let re = Regex::new(r"\d").unwrap();

                let contains_number = re.is_match(&result);

                if contains_number == true {
                    //sends message to gamechat v ector
                    self.feed.push(result);

                    //clear status message
                    let clear_event = "";
                    self.status_msg = clear_event.to_string();
                } else {
                    //update status message
                    self.status_msg = result;
                }

                true
            }
        }
    }
}
