use crate::chat_bus::ChatBus;
use crate::routes::Route;
use crate::ws::WebsocketService;

use regex::Regex;
use std::rc::Rc;
use std::time::Duration;
use yew::platform::time::sleep;

use web_sys::window;
use web_sys::{Element, MouseEvent};

use yew::platform::spawn_local;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

use yew::{html, Component, Html, NodeRef};

pub enum Msg {
    Roll,
    HandleMsg(String),
    Home,
}

pub struct PvPComponent {
    node_ref: NodeRef,
    ws: WebsocketService,
    feed: Vec<String>,
    _producer: Box<dyn Bridge<ChatBus>>,
    start_roll: String,
    status_msg: String,
    player_icon: String,
    spectator: bool,
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
        let location = web_sys::window().unwrap().location();
        let url = location.href().unwrap();
        let url_split: Vec<&str> = url.split('/').collect();

        let roll_amount = url_split[4];

        let cb = {
            let link = ctx.link().clone();
            move |msg| link.send_message(Msg::HandleMsg(msg))
        };

        let mut game_tx: WebsocketService = WebsocketService::ws_connect();

        game_tx.tx.try_send(roll_amount.to_string()).unwrap();

        Self {
            node_ref: NodeRef::default(),
            ws: game_tx,
            feed: Vec::new(),
            _producer: ChatBus::bridge(Rc::new(cb)),
            start_roll: roll_amount.to_string(),
            status_msg: "".to_string(),
            player_icon: "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}".to_string(),
            spectator: false,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let home = ctx.link().callback(move |_: MouseEvent| Msg::Home);

        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';
        let swords = "\u{2694}\u{FE0F} ";

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();
        if self.spectator == false {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home}>{"deathroll.gg "}{skull}{roll_emoji}</button>
                    <br/>
                    <br/>
                    {"To invite someone to play, give this URL: "}
                    <br/>
                    <h3>{url}</h3>
                  </div>
                </header>
                <div>
                  <main class="msger-chat" ref={self.node_ref.clone()}>
                    <div>
                     {swords}{&self.start_roll}
                      {
                        self.feed.clone().into_iter().map(|name| {
                          html!{

                            <div>
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
                  {&self.player_icon}{"\u{1F3B2} "}{&self.status_msg}</button>
                </div>
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
                    <br/>
                    <br/>
                    {"The arena is full, you are spectating \u{1F50E}"}
                    <br/>
                  </div>
                </header>
                <br/>
                <div>
                  <main class="msger-chat" ref={self.node_ref.clone()}>
                    <div>
                     {swords}{&self.start_roll}
                      {
                        self.feed.clone().into_iter().map(|name| {
                          html!{

                            <div>
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

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                let roll = "rolling".to_string();
                self.ws.tx.try_send(roll).unwrap();

                self.scroll_top();

                true
            }
            Msg::HandleMsg(result) => {
                self.scroll_top();
                let result_clone = result.clone();

                //log::debug!("result {:?}", result);
                let re = Regex::new(r"\d").unwrap();

                let contains_number = re.is_match(&result);
                //need to change this to a match at some point
                if contains_number == true {
                    //sends message to gamechat vector
                    self.feed.push(result);

                    //clear status message

                    self.status_msg = "".to_string();
                } else if result_clone.contains("left the game") {
                    self.feed.push(result);
                } else if result_clone.contains("joined the game") {
                    self.feed.push(result);
                    self.status_msg = "".to_string();
                } else if result_clone.contains("player_icon_set") {
                    self.player_icon = "\u{1F9DF}".to_string();
                } else if result_clone.contains("spectator") {
                    self.spectator = true;
                } else if result_clone.contains("disconnected") {
                    spawn_local(async move {
                        sleep(Duration::from_secs(2)).await;
                    });
                    let game_tx: WebsocketService = WebsocketService::ws_connect();

                    self.status_msg = "reconnecting...".to_string();
                    self.ws = game_tx;
                } else {
                    //update status message
                    self.status_msg = result;
                }

                true
            }
            Msg::Home => {
                let navigator = ctx.link().navigator().unwrap();
                // log::debug!("home");
                let mut ws = self.ws.clone();
                spawn_local(async move {
                    ws.close().await;
                });

                navigator.push(&Route::Home);

                true
            }
        }
    }
    fn destroy(&mut self, _ctx: &yew::Context<Self>) {
        self.ws.tx.try_send("close".to_string()).unwrap();
    }
}
