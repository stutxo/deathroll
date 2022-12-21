use futures::FutureExt;
use rand::Rng;
use std::time::Duration;
use std::vec;
use web_sys::{Element, HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::{html, Callback, Component, Html, NodeRef};
use yew_router::prelude::*;

use crate::Route;

const INIT_NUM: u32 = 100;

pub enum Msg {
    Roll,
    Reset,
    ComputerInitialized(()),
    PlayerRoll(()),
    PlayerResult(()),
    Input(String),
    Start,
    DoNothing,
}

pub struct PvEComponent {
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
}

impl PvEComponent {
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
}

impl Component for PvEComponent {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
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
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
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
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        html! {
         <div class="app-body">
         <header class="header">
         <div>
         <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
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
            if self.player_turn == false && self.game_over == false && self.game_start == false {<button hidden=true>{""}</button>
                 } else if self.player_turn == false && self.game_over == true && self.game_start == false  {
                     <button hidden=true>{""}</button>} else if self.player_turn == true && self.game_over == true && self.game_start == false {
                         <button hidden=true>{""}</button>} else if self.player_turn == true && self.game_over == false && self.game_start == true {
                            <button hidden=true>{""}</button> } else {
                             <button onclick={on_click} class="roll-button">{roll_emoji}</button>
            }
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
            <button onclick={start_game} class="roll-button">{roll_emoji}</button>
            </div>
            }
            </footer>
         </div>
         }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                self.game_start = false;
                self.player_turn = false;
                self.scroll_top();

                let slash_roll: String = "[player]: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll.clone() + space + &value;
                self.feed.push(is_rolling);

                let is_initialized = delay_roll();
                ctx.link().send_future(is_initialized.map(Msg::PlayerRoll));

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
            Msg::ComputerInitialized(_) => {
                self.scroll_top();

                self.computer_result = true;
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);

                log::debug!("computer roll: {:?}", self.roll_amount);

                self.computer_result = true;

                if self.roll_amount == 1 {
                    self.game_over = true;
                    self.player_turn = true;

                    let slash_roll = " rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                } else {
                    let slash_roll: String = " rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                }

                self.player_turn = true;
                true
            }
            Msg::PlayerRoll(_) => {
                self.scroll_top();

                self.computer_result = false;
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);

                log::debug!("player roll: {:?}", self.roll_amount);

                self.player_rolling = false;

                if self.roll_amount == 1 {
                    self.game_over = true;

                    let slash_roll = " rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);

                    log::debug!("player died");
                } else {
                    self.player_result = true;
                    let is_initialized = delay_roll();
                    ctx.link()
                        .send_future(is_initialized.map(Msg::PlayerResult));

                    let slash_roll: String = " rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                }

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

                let is_initialized = delay_roll();
                ctx.link()
                    .send_future(is_initialized.map(Msg::ComputerInitialized));

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
                    log::debug!("ERROR");
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

fn roll(num: u32) -> u32 {
    let mut rng = rand::thread_rng();

    let points = rng.gen_range(1..=num);

    points
}

async fn delay_roll() {
    sleep(Duration::from_secs(1)).await;
}
