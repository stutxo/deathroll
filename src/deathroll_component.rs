use web_sys::MouseEvent;
use yew::{html, Component, Html};

use rand::Rng;

pub enum Msg {
    Roll,
    DoNothing,
    Reset,
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
        let on_click = if self.game_over == false  {
            ctx.link().callback(move |_: MouseEvent| Msg::Roll)
        } else if self.game_over == true {
                
            ctx.link().callback(move |_: MouseEvent| Msg::Reset)
        } else {
            ctx.link().callback(move |_: MouseEvent| Msg::DoNothing)
        };

        let roll_log = self
            .display_roll
            .iter()
            .map(|value| html! {<p style = "text-align:left">{value}</p>});
        html! {
                <div class="content">
                <p style="font-size:30px">
                {self.roll_amount}
                </p>
                <p>
                 {if self.player_turn == false && self.game_over == false  {"Computer's turn"}
                 else if self.player_turn == true && self.game_over == false {"your turn"}
                 else if self.player_turn == false && self.game_over == true {"YOU DIED!!! DEFEAT!!!"}
                 else {"THE COMPUTER DIED!!! VICTORY!!!"}}
                 </p>
                 <br/>
                <p class="scroll">
                {for roll_log}
                </p>
                <br/>
                <br/>
                <p>
                <button onclick={on_click} style="height:80px;width:100%;font-size:30px">{
                        {if self.game_over == false  {"/roll"}
                        else {"Play again"}} } </button>
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

                //self.player_turn = false;

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
            Msg::Reset => {
                self.roll_amount = 1000000000;
                self.display_roll.clear();
                self.game_over = false;
                self.player_turn = true;

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
