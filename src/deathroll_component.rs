use futures::FutureExt;
use rand::Rng;
use std::time::Duration;
use std::vec;
use web_sys::{Element, MouseEvent};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::{html, Component, Html, NodeRef};

const INIT_NUM: u32 = 1000000000;

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
    feed: Vec<String>,
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
            let end = ") - VICTORY!!!";
            end.to_string()
        } else if self.game_over == true && self.computer_result == false {
            let end = ") - YOU DIED!!! RIP!!!";
            end.to_string()
        } else {
            let end = ")";
            end.to_string()
        }
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
            feed: Vec::new(),
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

        // let slash_roll: String = "Roll a 1 and you die!! ".to_string();
        // let space = " (1-".to_owned();
        // let value = INIT_NUM.to_string();
        // let end = ")";

        // let start_roll = space + &value + end;

        html! {
        <div class="app-body">
           <div>
              <h1 class="title">{"deathroll.gg"}</h1>
            //   <h1 class="sub-title">{slash_roll}</h1>
            //   <h1 class="start-num">{start_roll}</h1>
           </div>
           <div class="msger">
           <main class="msger-chat" id="chat-main" ref={self.node_ref.clone()}>
           <div class="dets">
              {
              self.feed.clone().into_iter().map(|name| {
              html!{

              <div class="msg" key={name.clone()}>

                    {name}

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
           <button onclick={if self.player_turn == false && self.game_over == false {block_roll}else{on_click}}>{
           {if self.game_over == false && self.player_turn == true && self.player_rolling == false {"/roll"}
           else if self.game_over == false && self.player_turn == false && self.player_rolling == false  {"rolling..."}
           else if self.game_over == false && self.player_rolling == true && self.player_turn == true {"rolling..."}
           else {"play again"}} } </button>
           </div>
           </footer>

        </div>
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                self.player_rolling = true;

                let slash_roll: String = "[player]: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll.clone() + space + &value;
                self.feed.push(is_rolling);

                self.scroll_top();

                let is_initialized = no_delay_roll();
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
                self.feed.clear();

                true
            }
            Msg::ComputerInitialized(_) => {
                self.computer_result = true;
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);

                log::debug!("computer roll: {:?}", self.roll_amount);
                self.scroll_top();
                self.computer_result = true;

                if self.roll_amount == 1 {
                    self.game_over = true;
                    self.player_turn = true;

                    let slash_roll = "computer rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);

                    log::debug!("computer died");
                } else {
                    let slash_roll: String = "computer rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                }

                self.player_turn = true;
                true
            }
            Msg::PlayerRoll(_) => {
                self.computer_result = false;
                self.game_start = false;
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);

                log::debug!("player roll: {:?}", self.roll_amount);

                self.player_rolling = false;

                self.scroll_top();

                if self.roll_amount == 1 {
                    self.game_over = true;

                    let slash_roll = "player rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);

                    log::debug!("player died");
                } else {
                    self.player_result = true;
                    let is_initialized = delay_roll();
                    ctx.link()
                        .send_future(is_initialized.map(Msg::PlayerResult));

                    let slash_roll: String = "player rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                }

                self.player_turn = false;

                true
            }
            Msg::PlayerResult(_) => {
                let slash_roll: String = "[computer]: /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();
                self.scroll_top();
                let is_rolling = slash_roll.clone() + space + &value;

                self.feed.push(is_rolling);
                self.scroll_top();
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

async fn delay_roll() {
    sleep(Duration::from_secs(1)).await;
}

async fn no_delay_roll() {}
