use nanoid::nanoid;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

pub struct Home {
    new_game: bool,
    input: NodeRef,
    pub start_roll: Option<u32>,
}

pub enum Msg {
    ShowNewGame,
    HideNewGame,
    Input(String),
    DoNothing,
    NewPvpGame,
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            new_game: false,
            input: NodeRef::default(),
            start_roll: None,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';

        let input_ref = self.input.clone();

        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        let navigator = ctx.link().navigator().unwrap();
        let pve = Callback::from(move |_: MouseEvent| navigator.push(&Route::PvE));
        let pvp = ctx.link().callback(move |_: MouseEvent| Msg::NewPvpGame);

        let oninput = ctx.link().batch_callback(move |_| {
            let input = input_ref.cast::<HtmlInputElement>();

            input.map(|input| Msg::Input(input.value()))
        });

        let new_game = ctx.link().callback(move |_: MouseEvent| Msg::ShowNewGame);
        let hide_new_game = ctx.link().callback(move |_: MouseEvent| Msg::HideNewGame);
        let start_game_enter = ctx.link().callback(move |e: KeyboardEvent| {
            if e.key_code() == 13 {
                Msg::NewPvpGame
            } else {
                Msg::DoNothing
            }
        });

        html! {
        <div class="app-body">
           <header class="header">
           <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
           <button onclick={pve} class="nav-button">{ "PvE" }</button>
           <button onclick={new_game}> {"PvP" }</button>
           if self.new_game {
                <div class="popup">
                    <input
                    ref ={self.input.clone()}
                    class="input-roll"
                    placeholder="roll amount"
                    oninput={oninput}
                    onkeypress={start_game_enter}
                    type="number" min="0" inputmode="numeric" pattern="[0-9]*"
                    title="Non-negative integral number"
                    />
                    <button onclick={pvp}>{ "new game" }</button>
                    <button onclick={hide_new_game}>{ "cancel" }</button>
                </div>
            } else {
            {""}
            }
            </header>
            <br/>
            {"deathrolling is a game made famous by World of Warcraft, where players deathroll for gold.
            Check out this video for an example of the game in action: "}
            <a href="https://youtu.be/vshLQqwfnjc?t=1044">{"https://youtu.be/vshLQqwfnjc?t=1044"}</a>

            <h3>{"Rules"}</h3>
            <ol>
              <li>{"Players take turns rolling a die."}</li>
              <li>{"The first player selects a number, and then rolls the die. The number they roll becomes the maximum number for the next player's roll."}</li>
              <li>{"If a player rolls a 1, they lose the game."}</li>
            </ol>
            {"code can be found here: "}<a href="https://github.com/stum0/deathroll">{"https://github.com/stum0/deathroll"}</a>

           <footer class="nav-bar-bottom">
           </footer>
        </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowNewGame => self.new_game = true,
            Msg::HideNewGame => self.new_game = false,
            Msg::Input(msg) => {
                let start_roll: u32 = match msg.trim().parse::<u32>() {
                    Ok(parsed_input) => parsed_input,

                    Err(_) => 1,
                };

                self.start_roll = Some(start_roll);
            }
            Msg::NewPvpGame => {
                if self.start_roll != Some(1) {
                    let navigator = ctx.link().navigator().unwrap();

                    let id = nanoid!(8);

                    navigator.push(&Route::PvP {
                        id: id,
                        roll: self.start_roll.unwrap(),
                    })
                } else {
                    //log::debug!("ERROR");
                }
            }
            Msg::DoNothing => {
                //log::debug!("Do nothing");
            }
        }
        true
    }
}
