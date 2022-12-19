use crate::Route;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};

use futures::FutureExt;
use rand::Rng;
use std::time::Duration;
use std::{default, vec};
use web_sys::window;
use web_sys::{Element, HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::platform::pinned::mpsc::{self, UnboundedReceiver, UnboundedSender};
use yew::platform::spawn_local;
use yew_router::prelude::*;

use yew::platform::time::sleep;
use yew::{html, Callback, Component, Html, NodeRef};

const INIT_NUM: u32 = 100;

pub enum Msg {
    Roll,
    Reset,
    PlayerResult(()),
    Input(String),
    Start,
    DoNothing,
}

pub struct PvPComponent {
    roll_amount: u32,
    player_turn: bool,
    game_over: bool,
    display_roll: Vec<u32>,
    player_rolling: bool,
    player_result: bool,
    game_start: bool,
    computer_result: bool,
    node_ref: NodeRef,
    feed: Vec<String>,
    my_input: NodeRef,
    num_input: u32,
    tx: UnboundedSender<Message>,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let node_ref = self.node_ref.clone();

        spawn_local(async move {
            let chat_main = node_ref.cast::<Element>().unwrap();
            chat_main.set_scroll_top(chat_main.scroll_height());
        })
    }
    fn add_to_feed(&self, slash_roll: String) -> String {
        let prev_turn = {
            self.display_roll
                .len()
                .checked_sub(2)
                .map(|i| self.display_roll[i])
                .unwrap()
        };

        let end = self.add_end();

        let borrowed_roll = self.roll_amount.to_string();
        let space = " (1-";
        let prev = prev_turn.to_string();

        let is_rolling = slash_roll.clone() + &borrowed_roll + space + &prev + &end;

        is_rolling
    }
    fn add_end(&self) -> String {
        if self.game_over == true && self.computer_result == true {
            let end = ") \u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}";
            end.to_string()
        } else if self.game_over == true && self.computer_result == false {
            let end = ") \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}";
            end.to_string()
        } else {
            let end = ") \u{1F3B2}";
            end.to_string()
        }
    }
    // async fn connect(self) {
    //     let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    //     let (tx1, mut rx1): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
    //         mpsc::unbounded();

    //     while let Some(message) = rx1.next().await {
    //         user_ws_tx
    //             .send(Message::Text(String::from("test")))
    //             .await
    //             .unwrap();
    //     }
    // }
}

impl Component for PvPComponent {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        let url_split: Vec<&str> = url.split('/').collect();

        let start = "ws://".to_owned();
        let host = url_split[2];
        let ws = "/ws/";
        let game_id = url_split[3];

        let full_url = start + host + ws + game_id;

        log::debug!("{:?}", game_id);

        let (tx1, mut rx1): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            mpsc::unbounded();
        let (tx2, mut rx2): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            mpsc::unbounded();

        let ws = WebSocket::open(&full_url).unwrap();
        let (mut write, mut read) = ws.split();
        spawn_local(async move {
            while let Some(message) = rx1.next().await {
                log::debug!("{:?}", message);
                write.send(message).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(result) = read.next().await {
                let _msg = match result {
                    Ok(msg) => log::debug!("{:?}", msg),
                    Err(e) => {
                        log::debug!("websocket error{}", e);
                        break;
                    }
                };
            }
        });

        Self {
            roll_amount: INIT_NUM,
            player_turn: true,
            game_over: false,
            display_roll: vec![INIT_NUM],
            player_rolling: false,
            player_result: false,
            game_start: true,
            computer_result: false,
            node_ref: NodeRef::default(),
            feed: Vec::new(),
            my_input: NodeRef::default(),
            num_input: 1,
            tx: tx1,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

        let roll_emoji = '\u{1F3B2}';
        let replay = '\u{1F504}';
        let skull = '\u{1F480}';

        let my_input_ref = self.my_input.clone();

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);
        let reset_game = ctx.link().callback(move |_: MouseEvent| Msg::Reset);

        let oninput = ctx.link().batch_callback(move |_| {
            let input = my_input_ref.cast::<HtmlInputElement>();

            input.map(|input| Msg::Input(input.value()))
        });

        let start_game = ctx.link().callback(move |_: MouseEvent| Msg::Start);
        let start_game_enter = ctx.link().callback(move |e: KeyboardEvent| {
            if e.key_code() == 13 {
                Msg::Start
            } else {
                Msg::DoNothing
            }
        });

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();
        html! {
            <div class="app-body">
            <header class="header">
            <div>
            <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
            <h1>{"1v1 me bruv"}{" "}</h1>
            {url}
            </div>
           </header>
           <div class="msger">
           <main class="msger-chat" id="chat-main" ref={self.node_ref.clone()}>
           <div class="dets">
          {
              self.feed.clone().into_iter().map(|name| {
              html!{

              <div class="msg" key={name.clone()}>
               {" "}{name}
               </div>

              }
              }).collect::
              <Html>
                 ()
                 }
                 </div>

           </main>
           </div>
           <footer class="nav-bar-bottom">

           <div>
           if self.game_over == false{<button hidden=true>{""}</button>
            } else {
           <button onclick={reset_game} class="replay-button">{replay}</button>
            }

           </div>
           <div>

         <button onclick={start_game} class="roll-button">{"start"}</button>
        <button onclick={on_click} class="roll-button">{roll_emoji}</button>

           </div>
           if self.game_start == true {
           <div class="div-input">

           <input
           ref ={self.my_input.clone()}
           class="input-roll"
           placeholder="roll amount"
           oninput={oninput}
           onkeypress={start_game_enter}
           type="number" min="0" inputmode="numeric" pattern="[0-9]*"
           title="Non-negative integral number"
           />

           </div>

           }
           </footer>
        </div>
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                let egg = "100000".to_string();
                self.tx.send_now(Message::Text(String::from(egg))).unwrap();

                self.game_start = false;
                self.player_turn = false;
                self.scroll_top();

                let slash_roll: String = "[player]: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll.clone() + space + &value;
                self.feed.push(is_rolling);

                true
            }
            Msg::Reset => {
                self.roll_amount = INIT_NUM;
                self.display_roll.clear();
                self.game_over = false;
                self.player_turn = true;
                self.display_roll.push(INIT_NUM);
                self.game_start = true;
                self.computer_result = false;
                self.feed.clear();
                self.num_input = 1;

                true
            }
            Msg::PlayerResult(_) => {
                self.scroll_top();

                let slash_roll: String = "[computer]: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll.clone() + space + &value;

                self.feed.push(is_rolling);

                self.player_result = false;

                true
            }
            Msg::Input(input) => {
                let num_input: u32 = match input.trim().parse::<u32>() {
                    Ok(parsed_input) => parsed_input,

                    Err(_) => 1,
                };

                self.num_input = num_input;

                true
            }
            Msg::Start => {
                if self.num_input != 1 {
                    //fix bug where game was not reseting correctly
                    self.display_roll.clear();
                    self.display_roll.push(self.num_input);
                    self.roll_amount = self.num_input;
                    log::debug!("{:?}", self.num_input);

                    ctx.link().send_message(Msg::Roll);
                } else {
                    log::debug!("ERROR NO NUM ENTERED");
                }

                true
            }
            Msg::DoNothing => {
                log::debug!("Do nothing");
                true
            }
        }
    }
}
