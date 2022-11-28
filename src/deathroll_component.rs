use futures::FutureExt;
use web_sys::MouseEvent;
use yew::platform::time::sleep;
use yew::{html, Component, Html};

use std::time::Duration;

use rand::Rng;

const INIT_NUM: u32 = 1000;

pub enum Msg {
    Roll,
    DoNothing,
    Reset,
    ComputerInitialized(()),
    PlayerRoll(()),
}

pub struct DeathRollComponent {
    roll_amount: u32,
    player_turn: bool,
    game_over: bool,
    display_roll: Vec<u32>,
    player_rolling: bool,
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

        let roll_log = self
            .display_roll
            .iter()
            .map(|value| html! {<p style = "text-align:left">{value}</p>});
        html! {
                <div class="content">
                <p style="font-size:25px">
                {if self.game_over == false && self.player_turn == false && self.player_rolling == false {"computer is rolling: 1-"}
                else if self.game_over == false && self.player_rolling == true && self.player_turn == true {"rolling: 1-"}
                else {""}}{self.roll_amount}
                </p>
                <p>
                 {if self.player_turn == false && self.game_over == true {"YOU DIED!!! RIP!!!"}
                 else if self.player_turn == true && self.game_over == true {"THE COMPUTER DIED!!! VICTORY!!!"}
                else {""}}
                 </p>
                 <br/>
                <p class="scroll">
                {for roll_log}
                </p>
                <br/>
                <br/>
                <br/>
                <br/>
                <p>
                <button onclick={if self.player_turn == false && self.game_over == false {block_roll}else{on_click}} style="height:80px;width:100%;font-size:30px">{
                        {if self.game_over == false && self.player_turn == true && self.player_rolling == false {"/roll"}
                        else if self.game_over == false && self.player_turn == false && self.player_rolling == false {"computer is rolling..."}
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

                true
            }
            Msg::ComputerInitialized(_) => {
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);
                log::debug!("computer roll: {:?}", self.roll_amount);

                let check_death = self.roll_amount;

                if check_death == 1 {
                    self.game_over = true;
                    self.player_turn = true;
                    log::debug!("THE COMPUTER DIED!!! VICTORY!!!");
                }

                self.player_turn = true;
                true
            }
            Msg::PlayerRoll(_) => {
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);
                log::debug!("player roll: {:?}", self.roll_amount);

                let check_death = self.roll_amount;

                if check_death == 1 {
                    self.game_over = true;
                    log::debug!("YOU DIED!!! DEFEAT!!!");
                } else {
                    let is_initialized = delay_roll();
                    ctx.link()
                        .send_future(is_initialized.map(Msg::ComputerInitialized));
                }

                self.player_turn = false;
                self.player_rolling = false;

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
    sleep(Duration::from_secs(2)).await;
}
