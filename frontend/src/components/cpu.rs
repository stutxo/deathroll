use futures::FutureExt;
use rand::Rng;
use std::time::Duration;
use std::vec;
use web_sys::{Element, MouseEvent};
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::{html, Callback, Component, Html, NodeRef};
use yew_router::prelude::*;

use crate::routes::Route;

pub enum Msg {
    Roll,
    Reset,
    ComputerInitialized(()),
    PlayerRoll(()),
    PlayerResult(()),
    Input(String),
    Start,
    DoNothing,
    ShowRules,
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
    feed_ref: NodeRef,
    feed: Vec<String>,
    num_input: u32,
    rules: bool,
}

impl PvEComponent {
    fn scroll_top(&self) {
        let feed_ref = self.feed_ref.clone();

        spawn_local(async move {
            let feed_main = feed_ref.cast::<Element>();
            if let Some(feed) = feed_main {
                feed.set_scroll_top(feed.scroll_height());
            }
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

        slash_roll + &borrowed_roll + space + &prev + &end
    }
    fn add_end(&self) -> String {
        if self.game_over {
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
        let location = web_sys::window().unwrap().location();
        let url = location.href().unwrap();
        let url_split: Vec<&str> = url.split('/').collect();

        let roll_amount = url_split[4];

        let num_input = roll_amount.trim().parse::<u32>().unwrap_or(1);

        Self {
            roll_amount: num_input,
            player_turn: true,
            game_over: false,
            display_roll: vec![num_input],
            player_rolling: false,
            player_result: false,
            game_start: true,
            computer_result: false,
            feed_ref: NodeRef::default(),
            feed: Vec::new(),
            num_input,
            rules: false,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let roll_emoji = '\u{1F3B2}';
        let replay = '\u{1F504}';
        // let skull = '\u{1F480}';
        let swords = "\u{2694}\u{FE0F} ";

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);
        let reset_game = ctx.link().callback(move |_: MouseEvent| Msg::Reset);

        let start_game = ctx.link().callback(move |_: MouseEvent| Msg::Start);

        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        let rules = ctx.link().callback(move |_: MouseEvent| Msg::ShowRules);

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
         <div>
         <h3>{"PvE (CPU) \u{1F916} "}{&self.num_input}</h3>
         </div>
        </header>
            <div>
            <main class="msger-feed" ref={self.feed_ref.clone()}>
            <div class="dets">
            {swords}{&self.num_input}
           {
               self.feed.clone().into_iter().map(|name| {
               html!{

               <div key={name.clone()}>
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
            <footer>

            <div>
            if !self.game_over  {<button hidden=true>{""}</button>
             } else {
            <button onclick={reset_game} class="roll-button">{replay}</button>
             }

            </div>
            <div>
            if !self.player_turn && !self.game_over  && !self.game_start {<button hidden=true>{""}</button>
                 } else if !self.player_turn && self.game_over  && !self.game_start  {
                     <button hidden=true>{""}</button>} else if self.player_turn && self.game_over  && !self.game_start {
                         <button hidden=true>{""}</button>} else if self.player_turn && !self.game_over  && self.game_start {
                            <button hidden=true>{""}</button> } else {
                             <button onclick={on_click} class="roll-button">{"\u{1F9D9}\u{200D}\u{2642}\u{FE0F}"}{roll_emoji}</button>
            }
            </div>
            if self.game_start {
            <div>
            <button onclick={start_game} class="roll-button">{roll_emoji}</button>
            </div>
            }
            </footer>
         </div>
         }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        let defeat = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F} \u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480}\u{1F480} DEFEAT!!!".to_string();
        let victory = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F} \u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6}\u{1F3C6} VICTORY!!!".to_string();
        match msg {
            Msg::Roll => {
                self.game_start = false;
                self.player_turn = false;
                self.scroll_top();

                let slash_roll: String = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F} /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll + space + &value;
                self.feed.push(is_rolling);

                let is_initialized = delay_roll();
                ctx.link().send_future(is_initialized.map(Msg::PlayerRoll));

                true
            }
            Msg::Reset => {
                let location = web_sys::window().unwrap().location();
                let url = location.href().unwrap();
                let url_split: Vec<&str> = url.split('/').collect();

                let roll_amount = url_split[4];

                log::debug!("{:?}", roll_amount);

                let num_input: u32 = roll_amount.trim().parse::<u32>().unwrap_or(1);
                self.roll_amount = num_input;
                self.display_roll.clear();
                self.game_over = false;
                self.player_turn = true;
                self.display_roll.push(num_input);
                self.game_start = true;
                self.computer_result = false;
                self.feed.clear();
                self.num_input = num_input;

                true
            }
            Msg::ComputerInitialized(_) => {
                self.scroll_top();

                self.computer_result = true;
                self.roll_amount = roll(self.roll_amount);
                self.display_roll.push(self.roll_amount);

                //log::debug!("computer roll: {:?}", self.roll_amount);

                self.computer_result = true;

                if self.roll_amount == 1 {
                    self.game_over = true;
                    self.player_turn = true;

                    let slash_roll = " \u{1F916} rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                    self.feed.push(victory);
                } else {
                    let slash_roll: String = " \u{1F916} rolls ".to_owned();
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

                //log::debug!("player roll: {:?}", self.roll_amount);

                self.player_rolling = false;

                if self.roll_amount == 1 {
                    self.game_over = true;

                    let slash_roll = " \u{1F9D9}\u{200D}\u{2642}\u{FE0F} rolled ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                    self.feed.push(defeat);
                    //log::debug!("player died");
                } else {
                    self.player_result = true;
                    let is_initialized = delay_roll();
                    ctx.link()
                        .send_future(is_initialized.map(Msg::PlayerResult));

                    let slash_roll: String = " \u{1F9D9}\u{200D}\u{2642}\u{FE0F} rolls ".to_owned();
                    let is_rolling = self.add_to_feed(slash_roll);
                    self.feed.push(is_rolling);
                }

                true
            }
            Msg::PlayerResult(_) => {
                self.scroll_top();

                let slash_roll: String = "\u{1F916} /roll ".to_owned();
                let space = " 1-";
                let value = self.roll_amount.to_string();

                let is_rolling = slash_roll + space + &value;

                self.feed.push(is_rolling);

                self.player_result = false;

                let is_initialized = delay_roll();
                ctx.link()
                    .send_future(is_initialized.map(Msg::ComputerInitialized));

                true
            }
            Msg::Input(input) => {
                let num_input: u32 = input.trim().parse::<u32>().unwrap_or(1);

                self.num_input = num_input;

                true
            }
            Msg::Start => {
                if self.num_input != 1 {
                    //fix bug where game was not reseting correctly
                    self.display_roll.clear();
                    self.display_roll.push(self.num_input);
                    self.roll_amount = self.num_input;
                    //log::debug!("{:?}", self.num_input);
                    ctx.link().send_message(Msg::Roll);
                } else {
                    //log::debug!("ERROR");
                }

                true
            }
            Msg::DoNothing => {
                //log::debug!("Do nothing");
                true
            }
            Msg::ShowRules => {
                if !self.rules {
                    self.rules = true
                } else if self.rules {
                    self.rules = false
                }
                true
            }
        }
    }
}

fn roll(num: u32) -> u32 {
    let mut rng = rand::thread_rng();

    rng.gen_range(1..=num)
}

async fn delay_roll() {
    sleep(Duration::from_secs(1)).await;
}
