use web_sys::MouseEvent;
use yew::{html, Component, Html};

use rand::Rng;

pub enum Msg {
    Roll,
    DoNothing,
}

pub struct DeathRollComponent {
    roll_amount: u32,
    player_turn: bool,
    game_over: bool,
    display_roll: Vec<u32>,
}

impl Component for DeathRollComponent {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            roll_amount: 1000000000,
            player_turn: true,
            game_over: false,
            display_roll: vec![1000000000],
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let on_click = if self.game_over == false {
            ctx.link().callback(move |_: MouseEvent| Msg::Roll)
        } else {
            ctx.link().callback(move |_: MouseEvent| Msg::DoNothing)
        };

        let prev_turn = self.display_roll.iter().map(|value| html! {<p>{value}</p>});
        html! {
                <div>
                <p style="font-size:30px">
                {self.roll_amount}
                </p>
                <p>
                <button onclick={on_click}>{
                        {if self.player_turn == false && self.game_over == false  {"Computer's turn: /roll"}
                        else if self.player_turn == true && self.game_over == false {"your turn: /roll"}
                        else if self.player_turn == false && self.game_over == true {"YOU DIED!!! DEFEAT!!!"}
                        else if self.player_turn == true && self.game_over == true {"THE COMPUTER DIED!!! VICTORY!!!"}
                        else {"Play again"}} } </button>
                </p>
                <p>
                {"log"}
                </p>
                <p style="height:100px;;solid #ccc;font:12px Serif;overflow:auto;">
                {for prev_turn}
                </p>
                </div>
        }
    }
    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                if self.player_turn == false {
                    self.player_turn = true
                } else {
                    self.player_turn = false
                }

                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);
                log::debug!("roll: {:?}", self.roll_amount);

                if self.roll_amount == 1 && self.player_turn == false {
                    log::debug!("YOU DIED!!! DEFEAT!!!");
                    self.game_over = true
                } else {
                    if self.roll_amount == 1 && self.player_turn == true {
                        log::debug!("THE COMPUTER DIED!!! VICTORY!!!");
                        self.game_over = true
                    }
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

    let points = rng.gen_range(1..num);

    points
}
