use crate::feed_bus::FeedBus;
use crate::routes::Route;
use crate::ws::WebsocketService;

use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::time::Duration;
use yew::platform::time::sleep;

use web_sys::window;
use web_sys::{Element, MouseEvent};

use yew::platform::spawn_local;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

use yew::{html, Callback, Component, Html, NodeRef};

pub enum Msg {
    Roll,
    HandleMsg(String),
    Copy,
}

pub enum WsMsg {
    Ping(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
struct GameMsg {
    roll_msg: Vec<String>,
}

pub struct PvPComponent {
    feed_ref: NodeRef,
    ws: WebsocketService,
    feed: Vec<String>,
    _producer: Box<dyn Bridge<FeedBus>>,
    status_msg: String,
    spectator: bool,
    game_start: bool,
    reconnecting: String,
    copy: bool,
    join_screen: bool,
    full_url: String,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let feed_ref = self.feed_ref.clone();

        if self.game_start {
            spawn_local(async move {
                let feed_main = feed_ref.cast::<Element>().unwrap();

                feed_main.set_scroll_top(feed_main.scroll_height());
            })
        }
    }
}

impl Component for PvPComponent {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &yew::Context<Self>) -> Self {
        let location = web_sys::window().unwrap().location();
        let url = location.href().unwrap();
        let url_split: Vec<&str> = url.split('/').collect();
        let host = location.host().unwrap();
        let protocol = location.protocol().unwrap();
        let ws_protocol = match protocol.as_str() {
            "https:" => "wss:",
            _ => "ws:",
        };

        let game_id = url_split[3];

        let full_url = format!("{}//{}/ws/{}", ws_protocol, host, game_id);

        let cb = {
            let link = ctx.link().clone();
            move |msg| link.send_message(Msg::HandleMsg(msg))
        };

        let mut game_tx: WebsocketService = WebsocketService::ws_connect(&full_url);

        Self {
            feed_ref: NodeRef::default(),
            ws: game_tx,
            feed: Vec::new(),
            _producer: FeedBus::bridge(Rc::new(cb)),
            status_msg: "".to_string(),
            spectator: false,
            game_start: false,
            reconnecting: "\u{1F7E2}".to_string(),
            copy: false,
            join_screen: false,
            full_url: full_url,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        let navigator = ctx.link().navigator().unwrap();
        let copy = ctx.link().callback(move |_: MouseEvent| Msg::Copy);
        let close = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();
        if !self.spectator && !self.game_start && !self.join_screen {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>

                    <h3>{"1v1 Challenge "}{&self.reconnecting}</h3>
                    {"To invite someone to play, give this URL: "}
                    <br/>
                    <br/>
                    <button onclick={copy} class="url-button">{url}{" "} if !self.copy {{" \u{1F4CB}"}} else {{"\u{2705}"}}</button>
                    <br/>
                    <br/>
                    {"The first person to come to this URL will play with you."}
                    <br/>
                    <br/>
                    {"waiting for player 2 to join..."}
                    <br/>
                    <br/>
                    <div>
                    <button onclick={close}>{" \u{274C} cancel "}</button>
                  </div>
                  </div>
                </header>

                </div>
            </body>
                  }
        } else if !self.spectator && self.game_start {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home}>{"deathroll.gg "}{skull}{roll_emoji}</button>
                  </div>
                  <h3>{"1v1 "}{&self.reconnecting}</h3>
                  </header>
                <div>
                  <main class="msger-feed" ref={self.feed_ref.clone()}>
                    <div class="dets-pvp">
                      {
                        self.feed.clone().into_iter().map(|name| {
                          html!{

                            <div key={name.clone()}>
                              {" "}{name}
                            </div>
                          }
                        }).collect::<Html>()
                      }
                    </div>
                  </main>
                </div>
                <div>

                  <button onclick={on_click} class="roll-button">
                  {&self.status_msg}</button>
                </div>
              </div>
            </body>
                  }
        } else if !self.spectator && !self.game_start && self.join_screen {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home}>{"deathroll.gg "}{skull}{roll_emoji}</button>

                    <h3>{"1v1 Challenge invite "}{&self.reconnecting}</h3>
                    <br/>
                    <div>
                    <button onclick={on_click}>{" join game "}</button>
                  </div>

                  </div>
                </header>
                </div>
            </body>
                  }
        } else {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home}>{"deathroll.gg "}{skull}{roll_emoji}</button>
                    <h3>{"1v1 "}{&self.reconnecting}</h3>
                    <h3>{"The arena is full, you are spectating \u{1F50E}"}</h3>
                  </div>
                </header>
                <br/>
                <div>
                  <main class="msger-feed" ref={self.feed_ref.clone()}>
                    <div>
                      {
                        self.feed.clone().into_iter().map(|name| {
                          html!{

                            <div key={name.clone()}>
                              {" "}{name}
                            </div>
                          }
                        }).collect::<Html>()
                      }
                    </div>
                  </main>
                </div>
              </div>
            </body>
                  }
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
              if self.game_start {
                let roll = "rolling".to_string();
                self.ws.tx.try_send(roll).unwrap();

                self.scroll_top();} else {
                  let start = "start".to_string();
                  self.ws.tx.try_send(start).unwrap();
                  let roll = "rolling".to_string();
                  self.ws.tx.try_send(roll).unwrap();
                }

                true
            }
            Msg::HandleMsg(result) => {
                self.scroll_top();
                log::debug!("{:?}", result);
                //will sort this mess out at somepoint by adding messages
                if result.contains("spec") {
                    self.spectator = true;
                } else if result.contains("disconnect") {
                    let game_tx: WebsocketService = WebsocketService::ws_connect(&self.full_url);
                    spawn_local(async move {
                        sleep(Duration::from_secs(2)).await;
                    });

                    self.reconnecting = "\u{1f534} reconnecting...".to_string();
                    self.ws = game_tx;
                } else if result.contains("reconn") {
                    self.game_start = true;
                    self.reconnecting = "\u{1F7E2}".to_string();
                } else if result.contains("!!!") {
                    self.status_msg = result.to_string();
                } else if result.contains("/roll") {
                    self.status_msg = result.to_string();
                } else if result.contains("rolled") {
                    self.status_msg = result.to_string();
                } else if result.contains("start the game") {
                    self.game_start = true;
                    self.status_msg = result.to_string();
                } else if result.contains("p2 join") {
                    self.join_screen = true;
                } else if result.contains("p1 join") {
                    self.join_screen = false;
                } else {
                    let feed: GameMsg = serde_json::from_str(&result).unwrap();
                    //sends message to gamefeed vector
                    self.feed = feed.roll_msg;
                    // self.game_start = true;
                    //clear status message
                }

                true
            }
            Msg::Copy => {
                let location = window().unwrap().location();
                let url = location.href().unwrap();
                // //must be run with RUSTFLAGS=--cfg=web_sys_unstable_apis for this to work
                if let Some(clipboard) = window().unwrap().navigator().clipboard() {
                    clipboard.write_text(&url);
                    self.copy = true;
                }
                true
            }
        }
    }
    fn destroy(&mut self, _ctx: &yew::Context<Self>) {
        let mut ws = self.ws.clone();
        spawn_local(async move {
            ws.close().await;
        });
        self.ws.tx.try_send("close".to_string()).unwrap();
    }
}
