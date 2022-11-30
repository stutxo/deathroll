use futures::FutureExt;
use web_sys::{Element, MouseEvent};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::{html, Component, Html, NodeRef};

use std::time::Duration;

use rand::Rng;

const INIT_NUM: u32 = 1000;

pub enum Msg {
    Roll,
    DoNothing,
    Reset,
    ComputerInitialized(()),
    PlayerRoll(()),
    PlayerResult(()),
}

pub struct DeathRollComponent {
    roll_amount: u32,
    player_turn: bool,
    game_over: bool,
    display_roll: Vec<u32>,
    player_rolling: bool,
    player_result: bool,
    game_start: bool,
    computer_result: bool,
    node_ref: NodeRef,
}

impl DeathRollComponent {
    fn scroll_top(&self) {
        let node_ref = self.node_ref.clone();

        spawn_local(async move {
            let chat_main = node_ref.cast::<Element>().unwrap();
            let current_scroll_top = chat_main.scroll_top();
            chat_main.set_scroll_top(current_scroll_top + 100000000);
        })
    }
}

impl Component for DeathRollComponent {
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
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let on_click = if self.game_over == false && self.player_rolling == false {
            ctx.link().callback(move |_: MouseEvent| Msg::Roll)
        } else if self.game_over == true && self.player_rolling == false {
            ctx.link().callback(move |_: MouseEvent| Msg::Reset)
        } else if self.player_rolling == true && self.game_over == false {
            ctx.link().callback(move |_: MouseEvent| Msg::DoNothing)
        } else {
            ctx.link().callback(move |_: MouseEvent| Msg::DoNothing)
        };

        let block_roll = ctx.link().callback(move |_: MouseEvent| Msg::DoNothing);

        let roll_log = self.display_roll.iter().map(|value| {
            html! {<div class="msg left-msg">

                <div class="msg-bubble">

                    <div class="msg-text">
                   {value.clone()}
                </div>
                </div>
                </div>

            }
        });

        let prev_turn = if self.game_start == false {
            self.display_roll
                .len()
                .checked_sub(2)
                .map(|i| self.display_roll[i])
                .unwrap()
        } else {
            INIT_NUM
        };

        html! { <div class="msger">
                    <div>
                        <h1>{"deathroll.gg"}</h1>
                        <br/>
                    </div>
                        <p style="font-size:20px">
                        {if self.game_over == false && self.player_turn == false && self.player_rolling == false && self.player_result == false && self.game_start == false    {"computer is rolling "}
                        else if self.game_over == false && self.player_rolling == true && self.player_turn == true && self.player_result == false && self.game_start == false  {"rolling "}
                        else if self.game_over == false && self.player_rolling == true && self.player_turn == true && self.player_result == false && self.game_start == true   {"rolling "}
                        else if self.game_over == false && self.player_rolling == false && self.player_turn == false && self.player_result == true && self.game_start == false {"player1 rolls "}
                        else if self.game_over == false && self.player_rolling == false && self.player_turn == true && self.player_result == false && self.game_start == false {"computer rolls "}
                        else {""}}{self.roll_amount}{" (1-"}

                        {if self.game_over == false && self.player_turn == false && self.player_rolling == false && self.player_result == false && self.game_start == true && self.computer_result == false {self.roll_amount}
                        else if self.game_over == false && self.player_turn == false && self.player_rolling == false && self.player_result == false && self.game_start == false && self.computer_result == false {self.roll_amount}
                        else if self.game_over == false && self.player_rolling == true && self.player_turn == true && self.player_result == false && self.game_start == false && self.computer_result == true{self.roll_amount}
                        else{prev_turn}} {")"}

                         {if self.player_turn == false && self.game_over == true {"YOU DIED!!! RIP!!!"}
                         else if self.player_turn == true && self.game_over == true {"THE COMPUTER DIED!!! VICTORY!!!"}
                        else {""}}
                        <br/>
                        <br/>
                        </p>
                         <main class="msger-chat" id="chat-main" ref={self.node_ref.clone()}>
                            <ul class="item-list">
                                { for roll_log }
                            </ul>
                         </main>
                        <p>
                        <button onclick={if self.player_turn == false && self.game_over == false {block_roll}else{on_click}} style="height:80px;width:100%;font-size:30px;">{
                                {if self.game_over == false && self.player_turn == true && self.player_rolling == false {"/roll"}
                                else if self.game_over == false && self.player_turn == false && self.player_rolling == false  {"rolling..."}
                                else if self.game_over == false && self.player_rolling == true && self.player_turn == true {"rolling..."}
                                else {"Play again"}} } </button>
                        </p>
                </div>
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                self.player_rolling = true;

                let is_initialized = delay_roll();
                ctx.link().send_future(is_initialized.map(Msg::PlayerRoll));

                true
            }
            Msg::DoNothing => {
                log::debug!("Do nothing");
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

                true
            }
            Msg::ComputerInitialized(_) => {
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);
                log::debug!("computer roll: {:?}", self.roll_amount);
                self.scroll_top();
                self.computer_result = true;

                if self.roll_amount == 1 {
                    self.game_over = true;
                    self.player_turn = true;
                    log::debug!("THE COMPUTER DIED!!! VICTORY!!!");
                }

                self.player_turn = true;
                true
            }
            Msg::PlayerRoll(_) => {
                self.game_start = false;
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);
                log::debug!("player roll: {:?}", self.roll_amount);
                self.scroll_top();
                if self.roll_amount == 1 {
                    self.game_over = true;
                    log::debug!("YOU DIED!!! DEFEAT!!!");
                } else {
                    self.player_result = true;
                    let is_initialized = delay_sec_roll();
                    ctx.link()
                        .send_future(is_initialized.map(Msg::PlayerResult));
                }

                self.player_turn = false;
                self.player_rolling = false;

                true
            }
            Msg::PlayerResult(_) => {
                self.player_result = false;
                let is_initialized = delay_roll();
                ctx.link()
                    .send_future(is_initialized.map(Msg::ComputerInitialized));

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

pub async fn delay_roll() {
    sleep(Duration::from_secs(1)).await;
}
pub async fn delay_sec_roll() {
    sleep(Duration::from_secs(1)).await;
}
