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
    ShowRules,
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
    start_roll: String,
    rules: bool,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let feed_ref = self.feed_ref.clone();

        spawn_local(async move {
            let feed_main = feed_ref.cast::<Element>();
            match feed_main {
                Some(feed) => {
                    feed.set_scroll_top(feed.scroll_height());
                }
                None => {}
            }
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

        let game_tx: WebsocketService = WebsocketService::ws_connect(&full_url);
        let mut game_tx_clone = game_tx.clone();
        spawn_local(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                let roll = "ping".to_string();
                game_tx_clone.tx.try_send(roll).unwrap();
            }
        });

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
            full_url: full_url,
            start_roll: "".to_string(),
            rules: false,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        self.scroll_top();

        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        let navigator = ctx.link().navigator().unwrap();
        let copy = ctx.link().callback(move |_: MouseEvent| Msg::Copy);
        let close = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

        // let roll_emoji = '\u{1F3B2}';
        // let skull = '\u{1F480}';

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        let rules = ctx.link().callback(move |_: MouseEvent| Msg::ShowRules);

        if !self.spectator && !self.game_start && !self.join_screen {
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

                  <h3>{"PvP (Multiplayer 1v1) "}{&self.start_roll}</h3>
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
                <h3>{"PvP (Multiplayer 1v1) "}{&self.start_roll}</h3>
                </header>
              <div>
                <main class="msger-feed" ref={self.feed_ref.clone()}>
                  <div class="dets-pvp">
                  {&self.start_roll}
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

                  <h3>{"PvP (Multiplayer 1v1) "}{&self.start_roll}</h3>
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

                  <h3>{"PvP (Multiplayer 1v1) "}{&self.start_roll}</h3>
                  <h3>{"The arena is full, you are spectating \u{1F50E}"}</h3>
                </div>
              </header>
              <br/>
              <div>
                <main class="msger-feed" ref={self.feed_ref.clone()}>
                  <div class="dets-pvp">
                  {&self.start_roll}
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

              <br/>
              {&self.reconnecting}
            </div>
                }
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                self.scroll_top();
                if self.game_start {
                    let roll = "rolling".to_string();
                    self.ws.tx.try_send(roll).unwrap();
                } else {
                    let start = "start".to_string();
                    self.ws.tx.try_send(start).unwrap();
                    let roll = "rolling".to_string();
                    self.ws.tx.try_send(roll).unwrap();
                }

                true
            }
            Msg::HandleMsg(result) => {
                self.scroll_top();
                self.reconnecting = "".to_string();

                //will sort this mess out at somepoint by adding messages
                if result.contains("spec") {
                    self.spectator = true;
                } else if result.contains("disconnect") {
                    let game_tx: WebsocketService = WebsocketService::ws_connect(&self.full_url);
                    spawn_local(async move {
                        sleep(Duration::from_secs(2)).await;
                    });

                    self.reconnecting = "\u{1f534} Reconnecting...".to_string();
                    self.ws = game_tx;
                } else if result.contains("reconn") {
                    self.game_start = true;
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
                } else if result.contains("\u{2694}\u{FE0F}") {
                    self.start_roll = result;
                } else {
                    let feed: GameMsg = serde_json::from_str(&result).unwrap();
                    //sends message to gamefeed vector
                    self.feed = feed.roll_msg;
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
            Msg::ShowRules => {
                if self.rules == false {
                    self.rules = true
                } else if self.rules == true {
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
        self.ws.tx.try_send("close".to_string()).unwrap();
    }
}
