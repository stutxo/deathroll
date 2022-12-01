use futures::FutureExt;
use std::time::Duration;
use web_sys::{Element, MouseEvent};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::{html, Component, Html, NodeRef};

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
    print: Vec<String>,
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
        let slash_roll: String = "roll a 1 and you die!! ".to_owned();
        let space = " (1-";
        let value = INIT_NUM.to_string();
        let end = ")";

        let start_roll = slash_roll + space + &value + end;

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
            print: vec![start_roll],
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

        let roll_log = self.print.iter().map(|value| {
            html! {<div class="msg">

                <div class="msg-bubble">

                    <div class="msg-text">
                    {value}
                </div>
                </div>
                </div>
            }
        });

        html! { <div class="msger">
                    <div>
                        <h1>{"deathroll.gg"}</h1>
                    </div>
                        <main class="msger-chat" id="chat-main" ref={self.node_ref.clone()}>

                        {for roll_log}

                        </main>
                       <div>
                       <br/>
                       <br/>
                        <button onclick={if self.player_turn == false && self.game_over == false {block_roll}else{on_click}}>{
                                {if self.game_over == false && self.player_turn == true && self.player_rolling == false {"/roll"}
                                else if self.game_over == false && self.player_turn == false && self.player_rolling == false  {"rolling..."}
                                else if self.game_over == false && self.player_rolling == true && self.player_turn == true {"rolling..."}
                                else {"Play again"}} } </button>
                        </div>
                </div>
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                self.player_rolling = true;

                let slash_roll: String = "player: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll + space + &value;

                self.print.push(is_rolling);

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
                self.print.clear();

                let slash_roll: String = "roll a 1 and you die!! ".to_owned();
                let space = " (1-";
                let value = INIT_NUM.to_string();
                let end = ")";

                let start_roll = slash_roll + space + &value + end;

                self.print = vec![start_roll];

                true
            }
            Msg::ComputerInitialized(_) => {
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);

                let prev_turn = if self.game_start == false {
                    self.display_roll
                        .len()
                        .checked_sub(2)
                        .map(|i| self.display_roll[i])
                        .unwrap()
                } else {
                    INIT_NUM
                };

                let slash_roll: String = "computer rolls ".to_owned();
                let borrowed_string = self.roll_amount.to_string();
                let space = " (1-";
                let prev = prev_turn.to_string();
                let end = ")";

                let together = slash_roll.clone() + &borrowed_string + space + &prev + end;

                self.print.push(together);

                log::debug!("computer roll: {:?}", self.roll_amount);
                self.scroll_top();
                self.computer_result = true;

                if self.roll_amount == 1 {
                    self.game_over = true;
                    self.player_turn = true;
                    let death_message = "THE COMPUTER DIED!!! VICTORY!!".to_string();
                    self.print.push(death_message);
                    log::debug!("computer died");
                }

                self.player_turn = true;
                true
            }
            Msg::PlayerRoll(_) => {
                self.game_start = false;
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);

                let prev_turn = if self.game_start == false {
                    self.display_roll
                        .len()
                        .checked_sub(2)
                        .map(|i| self.display_roll[i])
                        .unwrap()
                } else {
                    INIT_NUM
                };

                let slash_roll: String = "player rolls ".to_owned();
                let borrowed_string = self.roll_amount.to_string();
                let space = " (1-";
                let prev = prev_turn.to_string();
                let end = ")";

                let together = slash_roll.clone() + &borrowed_string + space + &prev + end;

                self.print.push(together);

                log::debug!("player roll: {:?}", self.roll_amount);
                self.scroll_top();
                if self.roll_amount == 1 {
                    self.game_over = true;
                    let death_message = "YOU DIED!!! RIP!!!".to_string();
                    self.print.push(death_message);
                    log::debug!("player died");
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

                let slash_roll: String = "computer: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll + space + &value;

                self.print.push(is_rolling);

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
