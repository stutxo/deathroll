use crate::routes::Route;
use crate::services::feed_bus::FeedBus;
use crate::services::websockets::{WebsocketService, WsMsg};

use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::time::Duration;
use yew::platform::time::sleep;

use gloo_timers::callback::Interval;
use web_sys::window;
use web_sys::{Element, MouseEvent};

use yew::platform::spawn_local;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

use yew::{html, Callback, Component, Html, NodeRef};

#[derive(Serialize, Deserialize)]
pub enum GameMessage {
    Spectate,
    StartGame(String),
    Disconnect,
    Reconnect,
    NoGameFound,
    P1Join,
    P2Join,
    Status(String),
    GameScore(GameScore),
    StartRoll(String),
    Pong,
    GameOver(String),
}

pub enum CompMsg {
    Roll,
    HandleMsg(String),
    Copy,
    ShowRules,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameScore {
    client_feed: Vec<String>,
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
    start_roll: String,
    rules: bool,
    connected: bool,
    replay: bool,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let feed_ref = self.feed_ref.clone();

        spawn_local(async move {
            let feed_main = feed_ref.cast::<Element>();
            if let Some(feed) = feed_main {
                feed.set_scroll_top(feed.scroll_height());
            }
        })
    }
}

impl Component for PvPComponent {
    type Message = CompMsg;
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
            move |msg| link.send_message(CompMsg::HandleMsg(msg))
        };

        let game_tx: WebsocketService = WebsocketService::ws_connect(&full_url);
        // let mut game_tx_clone = game_tx.clone();
        // // game_tx_clone
        // //     .tx
        // //     .try_send(serde_json::to_string(&WsMsg::Ping).unwrap())
        // //     .unwrap();
        Self {
            feed_ref: NodeRef::default(),
            ws: game_tx,
            feed: Vec::new(),
            _producer: FeedBus::bridge(Rc::new(cb)),
            status_msg: "".to_string(),
            spectator: false,
            game_start: false,
            reconnecting: "".to_string(),
            copy: false,
            join_screen: false,
            full_url,
            start_roll: "".to_string(),
            rules: false,
            connected: false,
            replay: false,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        self.scroll_top();
        let replay = '\u{1F504}';
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        let navigator = ctx.link().navigator().unwrap();
        let copy = ctx.link().callback(move |_: MouseEvent| CompMsg::Copy);
        let close = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

        let on_click = ctx.link().callback(move |_: MouseEvent| CompMsg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        let reset_game = ctx.link().callback(move |_: MouseEvent| CompMsg::Roll);

        let rules = ctx.link().callback(move |_: MouseEvent| CompMsg::ShowRules);

        if !self.connected {
            html! {
            <div>
              <header>
                <div>
                <button onclick={home} class="title-button">{"deathroll.gg "}{"\u{1F3E0}"}</button>
                <button onclick={rules} class="title-button"> {"\u{1F4D6}" }</button>
                if self.rules {
                     <div class="rules">
                     <p>{"Deathrolling is a game made famous by World of Warcraft, where players deathroll for gold."}</p>
                     <p>{"Check out this video for an example of the game in action: "}<a href="https://youtu.be/vshLQqwfnjc?t=1044">{"https://youtu.be/vshLQqwfnjc?t=1044"}</a></p>
                     <ol>
                   <li>{"Players take turns rolling a die."}</li>
                   <li>{"The first player selects a number, and then rolls the die. The number they roll becomes the maximum number for the next player's roll."}</li>
                   <li>{"If a player rolls a 1, they lose the game."}</li>
                     </ol>

                     </div>
                 }
                  {" "}<a href="https://github.com/stum0/deathroll"><i class="fab fa-github-square" style="font-size:25px"></i></a>

                  <h3>{"PvP (Multiplayer 1v1) "}{"\u{2694}\u{FE0F} "}{&self.start_roll}</h3>
                  {"connecting... "}
                  <br/>
                  <br/>
                  <div>
                  <button onclick={close}>{" \u{274C} CANCEL "}</button>
                  <br/>
                  {&self.reconnecting}
                </div>
                </div>
              </header>
              </div>
                }
        } else if !self.spectator && !self.game_start && !self.join_screen {
            html! {
            <div>
              <header>
                <div>
                <button onclick={home} class="title-button">{"deathroll.gg "}{"\u{1F3E0}"}</button>
                <button onclick={rules} class="title-button"> {"\u{1F4D6}" }</button>
                if self.rules {
                     <div class="rules">
                     <p>{"Deathrolling is a game made famous by World of Warcraft, where players deathroll for gold."}</p>
                     <p>{"Check out this video for an example of the game in action: "}<a href="https://youtu.be/vshLQqwfnjc?t=1044">{"https://youtu.be/vshLQqwfnjc?t=1044"}</a></p>
                     <ol>
                   <li>{"Players take turns rolling a die."}</li>
                   <li>{"The first player selects a number, and then rolls the die. The number they roll becomes the maximum number for the next player's roll."}</li>
                   <li>{"If a player rolls a 1, they lose the game."}</li>
                     </ol>

                     </div>
                 }
                  {" "}<a href="https://github.com/stum0/deathroll"><i class="fab fa-github-square" style="font-size:25px"></i></a>

                  <h3>{"PvP (Multiplayer 1v1) "}{"\u{2694}\u{FE0F} "}{&self.start_roll}</h3>
                  {"To invite someone to play, give this URL: "}
                  <br/>
                  <br/>
                  <button onclick={copy} class="url-button">{url}{" "} if !self.copy {{" \u{1F4CB}"}} else {{"\u{2705}"}}</button>
                  <br/>
                  <br/>
                  {"Waiting for player 2 to join..."}
                  <br/>
                  <br/>
                  <div>
                  <button onclick={close}>{" \u{274C} CANCEL "}</button>
                  <br/>

                  {&self.reconnecting}
                </div>
                </div>
              </header>
              </div>
                }
        } else if !self.spectator && self.game_start {
            html! {
            <div>
              <header>
                <div>
                <button onclick={home} class="title-button">{"deathroll.gg "}{"\u{1F3E0}"}</button>
                <button onclick={rules} class="title-button"> {"\u{1F4D6}" }</button>
                if self.rules {
                     <div class="rules">
                     <p>{"Deathrolling is a game made famous by World of Warcraft, where players deathroll for gold."}</p>
                     <p>{"Check out this video for an example of the game in action: "}<a href="https://youtu.be/vshLQqwfnjc?t=1044">{"https://youtu.be/vshLQqwfnjc?t=1044"}</a></p>
                     <ol>
                   <li>{"Players take turns rolling a die."}</li>
                   <li>{"The first player selects a number, and then rolls the die. The number they roll becomes the maximum number for the next player's roll."}</li>
                   <li>{"If a player rolls a 1, they lose the game."}</li>
                     </ol>

                     </div>
                 }
                  {" "}<a href="https://github.com/stum0/deathroll"><i class="fab fa-github-square" style="font-size:25px"></i></a>

                </div>
                <h3>{"PvP (Multiplayer 1v1) "}{"\u{2694}\u{FE0F} "}{&self.start_roll}</h3>
                </header>
              <div>
                <main class="msger-feed" ref={&self.feed_ref}>
                  <div>
                  {"\u{2694}\u{FE0F} "}{&self.start_roll}
                    {
                      self.feed.clone().into_iter().map(|name| {
                        if name.contains("\u{1F9D9}\u{200D}\u{2642}\u{FE0F}") {
                        html!{

                          <div key={name.clone()}style="color:blue">
                            {" "}{name}
                          </div>
                        }
                       } else if name.contains("\u{1F9DF}") {
                          html!{
                            <div key={name.clone()}style="color:green">
                              {" "}{name}
                            </div>
                          }
                        } else {
                          html!{
                            <div key={name.clone()}style="color:black">
                              {" "}{name}
                            </div>
                          }
                        }
                      }).collect::<Html>()

                    }
                  </div>
                </main>
              </div>
              <div>

                if !self.replay  { <button onclick={on_click} class="roll-button">
                {&self.status_msg}</button>
                <br/>
              } else {
                <button class="roll-button">
                {&self.status_msg}</button>
                <br/>
             <button onclick={reset_game} class="roll-button">{replay}</button>
              }
              <br/>
                {&self.reconnecting}
              </div>
            </div>
                }
        } else if !self.spectator && !self.game_start && self.join_screen {
            html! {
            <div>
              <header>
                <div>
                <button onclick={home} class="title-button">{"deathroll.gg "}{"\u{1F3E0}"}</button>
                <button onclick={rules} class="title-button"> {"\u{1F4D6}" }</button>
                if self.rules {
                     <div class="rules">
                     <p>{"Deathrolling is a game made famous by World of Warcraft, where players deathroll for gold."}</p>
                     <p>{"Check out this video for an example of the game in action: "}<a href="https://youtu.be/vshLQqwfnjc?t=1044">{"https://youtu.be/vshLQqwfnjc?t=1044"}</a></p>
                     <ol>
                   <li>{"Players take turns rolling a die."}</li>
                   <li>{"The first player selects a number, and then rolls the die. The number they roll becomes the maximum number for the next player's roll."}</li>
                   <li>{"If a player rolls a 1, they lose the game."}</li>
                     </ol>

                     </div>
                 }
                  {" "}<a href="https://github.com/stum0/deathroll"><i class="fab fa-github-square" style="font-size:25px"></i></a>

                  <h3>{"PvP (Multiplayer 1v1) "}{"\u{2694}\u{FE0F} "}{&self.start_roll}</h3>
                  <div>
                  {"You have been invited to play"}
                  <br/>
                  <br/>
                  <button onclick={on_click}>{" JOIN THE GAME "}</button>
                  <br/>

                  {&self.reconnecting}
                </div>
                </div>
              </header>
              </div>
                }
        } else {
            html! {
            <div>
              <header>
                <div>
                <button onclick={home} class="title-button">{"deathroll.gg "}{"\u{1F3E0}"}</button>
                <button onclick={rules} class="title-button"> {"\u{1F4D6}" }</button>
                if self.rules {
                     <div class="rules">
                     <p>{"Deathrolling is a game made famous by World of Warcraft, where players deathroll for gold."}</p>
                     <p>{"Check out this video for an example of the game in action: "}<a href="https://youtu.be/vshLQqwfnjc?t=1044">{"https://youtu.be/vshLQqwfnjc?t=1044"}</a></p>
                     <ol>
                   <li>{"Players take turns rolling a die."}</li>
                   <li>{"The first player selects a number, and then rolls the die. The number they roll becomes the maximum number for the next player's roll."}</li>
                   <li>{"If a player rolls a 1, they lose the game."}</li>
                     </ol>

                     </div>
                 }
                  {" "}<a href="https://github.com/stum0/deathroll"><i class="fab fa-github-square" style="font-size:25px"></i></a>

                  <h3>{"PvP (Multiplayer 1v1) "}{"\u{2694}\u{FE0F} "}{&self.start_roll}</h3>
                  <h3>{"The arena is full, you are spectating \u{1F50E}"}</h3>
                </div>
              </header>
              <br/>
              <div>
                <main class="msger-feed" ref={&self.feed_ref}>
                  <div>
                  {"\u{2694}\u{FE0F} "}{&self.start_roll}
                  {
                    self.feed.clone().into_iter().map(|name| {
                      if name.contains("\u{1F9D9}\u{200D}\u{2642}\u{FE0F}") {
                      html!{

                        <div key={name.clone()}style="color:blue">
                          {" "}{name}
                        </div>
                      }
                     } else if name.contains("\u{1F9DF}") {
                        html!{
                          <div key={name.clone()}style="color:green">
                            {" "}{name}
                          </div>
                        }
                      } else {
                        html!{
                          <div key={name.clone()}style="color:black">
                            {" "}{name}
                          </div>
                        }
                      }
                    }).collect::<Html>()
                    }
                  </div>
                </main>
              </div>

              <br/>
              {&self.reconnecting}
            </div>
                }
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CompMsg::Roll => {
                self.scroll_top();

                self.ws
                    .tx
                    .try_send(serde_json::to_string(&WsMsg::Roll).unwrap())
                    .unwrap();

                self.replay = false;

                true
            }
            CompMsg::HandleMsg(result) => {
                self.scroll_top();
                self.connected = true;
                self.reconnecting = "".to_string();

                let message: GameMessage = serde_json::from_str(&result).unwrap();
                match message {
                    GameMessage::Disconnect => {
                        let game_tx: WebsocketService =
                            WebsocketService::ws_connect(&self.full_url);
                        self.reconnecting = "\u{1f534} Reconnecting...".to_string();
                        self.ws = game_tx;
                        spawn_local(async move {
                            sleep(Duration::from_secs(3)).await;
                        });
                    }
                    GameMessage::Reconnect => self.game_start = true,
                    GameMessage::Spectate => self.spectator = true,
                    GameMessage::StartGame(msg) => {
                        self.game_start = true;
                        self.status_msg = msg;
                    }
                    GameMessage::NoGameFound => {
                        ctx.link().navigator().unwrap().push(&Route::NotFound)
                    }
                    GameMessage::P1Join => self.join_screen = false,
                    GameMessage::P2Join => self.join_screen = true,
                    GameMessage::Status(msg) => {
                        self.replay = false;
                        self.status_msg = msg;
                    }
                    GameMessage::StartRoll(roll) => self.start_roll = roll,
                    GameMessage::GameScore(feed) => self.feed = feed.client_feed,
                    GameMessage::Pong => {
                        // let mut game_tx_clone = self.ws.tx.clone();

                        // let _interval = Interval::new(20_000, move || {
                        //     game_tx_clone
                        //         .try_send(serde_json::to_string(&WsMsg::Ping).unwrap())
                        //         .unwrap()
                        // });
                    }
                    GameMessage::GameOver(msg) => {
                        self.status_msg = msg;
                        self.replay = true;
                    }
                }

                true
            }
            CompMsg::Copy => {
                #[cfg(web_sys_unstable_apis)]
                let location = window().unwrap().location();
                #[cfg(web_sys_unstable_apis)]
                let url = location.href().unwrap();
                // //must be run with RUSTFLAGS=--cfg=web_sys_unstable_apis for this to work
                #[cfg(web_sys_unstable_apis)]
                if let Some(clipboard) = window().unwrap().navigator().clipboard() {
                    clipboard.write_text(&url);
                    self.copy = true;
                }
                true
            }
            CompMsg::ShowRules => {
                if !self.rules {
                    self.rules = true
                } else if self.rules {
                    self.rules = false
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
        self.ws
            .tx
            .try_send(serde_json::to_string(&WsMsg::Close).unwrap())
            .unwrap();
    }
}
