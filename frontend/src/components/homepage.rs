use gloo_net::http::Request;
use nanoid::nanoid;

use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

use crate::routes::Route;

pub struct Home {
    rules: bool,
    input: NodeRef,
    input_pve: NodeRef,
    pub start_roll: Option<u32>,
    pub start_roll_pve: Option<u32>,
}

pub enum Msg {
    ShowRules,
    HideRules,
    Input(String),
    DoNothing,
    NewPvpGameCustom,
    NewPvpGame(u32),
    NewPveGame(u32),
    NewPveGameCustom,
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            rules: false,
            input: NodeRef::default(),
            input_pve: NodeRef::default(),
            start_roll: None,
            start_roll_pve: None,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let input_ref_pvp = self.input.clone();
        let input_ref_pve = self.input_pve.clone();

        let home = ctx.link().callback(move |_: MouseEvent| Msg::HideRules);

        let pvp = ctx
            .link()
            .callback(move |_: MouseEvent| Msg::NewPvpGameCustom);

        let pve = ctx
            .link()
            .callback(move |_: MouseEvent| Msg::NewPveGameCustom);

        let rules = ctx.link().callback(move |_: MouseEvent| Msg::ShowRules);

        let oninput_pvp = ctx.link().batch_callback(move |_| {
            let input = input_ref_pvp.cast::<HtmlInputElement>();

            input.map(|input| Msg::Input(input.value()))
        });

        let start_game_enter_pvp = ctx.link().callback(move |e: KeyboardEvent| {
            if e.key_code() == 13 {
                Msg::NewPvpGameCustom
            } else {
                Msg::DoNothing
            }
        });

        let oninput_pve = ctx.link().batch_callback(move |_| {
            let input = input_ref_pve.cast::<HtmlInputElement>();

            input.map(|input| Msg::Input(input.value()))
        });

        let start_game_enter_pve = ctx.link().callback(move |e: KeyboardEvent| {
            if e.key_code() == 13 {
                Msg::NewPveGameCustom
            } else {
                Msg::DoNothing
            }
        });

        html! {
        <div>
           <header>
           <button onclick={home} class="title-button">{"deathroll.gg "}{"\u{1F3E0}"}</button>
           <button onclick={rules} class="title-button">{"\u{1F4D6}"}</button>
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

            </header>
            <h3>{"PvP (Multiplayer 1v1) \u{2694}\u{FE0F}"}</h3>
                <button onclick={pvp_roll(100, ctx)}>{ "100" }</button>
                <button onclick={pvp_roll(1000, ctx)}>{ "1000" }</button>
                <button onclick={pvp_roll(10000, ctx)}>{ "10000" }</button>
                <button onclick={pvp_roll(100000, ctx)}>{ "100000" }</button>
                <br/>
                <button onclick={pvp_roll(1000000, ctx)}>{ "1000000" }</button>
                <button onclick={pvp_roll(10000000, ctx)}>{ "10000000" }</button>
                <button onclick={pvp_roll(100000000, ctx)}>{ "100000000" }</button>
                <br/>
                    <input
                    ref ={self.input.clone()}
                    placeholder="custom roll"
                    oninput={oninput_pvp}
                    onkeypress={start_game_enter_pvp}
                    type="text" maxlength="9" min="1" max="100000000" inputmode="numeric" pattern="[0-9]*"
                    title="Non-negative integral number"

                    /> <button onclick={pvp}>{ "custom game" }</button>
                <h3>{"PvE (CPU) \u{1F916}"}</h3>
                <button onclick={pve_roll(100, ctx)}>{ "100" }</button>
                <button onclick={pve_roll(1000, ctx)}>{ "1000" }</button>
                <button onclick={pve_roll(10000, ctx)}>{ "10000" }</button>
                <button onclick={pve_roll(100000, ctx)}>{ "100000" }</button>
                <br/>
                <button onclick={pve_roll(1000000, ctx)}>{ "1000000" }</button>
                <button onclick={pve_roll(10000000, ctx)}>{ "10000000" }</button>
                <button onclick={pve_roll(100000000, ctx)}>{ "100000000" }</button>
                <br/>
                    <input
                    ref ={self.input_pve.clone()}
                    placeholder="custom roll"
                    oninput={oninput_pve}
                    onkeypress={start_game_enter_pve}
                    type="text" maxlength="9" min="1" max="100000000" inputmode="numeric" pattern="[0-9]*"
                    title="Non-negative integral number"

                    /> <button onclick={pve}>{ "custom game" }</button>
        </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowRules => {
                if !self.rules {
                    self.rules = true
                } else if self.rules {
                    self.rules = false
                }
                true
            }
            Msg::HideRules => {
                self.rules = false;
                true
            }
            Msg::Input(msg) => {
                let start_roll: u32 = msg.trim().parse::<u32>().unwrap_or(0);

                self.start_roll = Some(start_roll);
                true
            }
            Msg::NewPvpGameCustom => {
                if self.start_roll != Some(0) {
                    let navigator = ctx.link().navigator().unwrap();

                    let game_id = nanoid!(8);
                    let location = web_sys::window().unwrap().location();

                    let host = location.host().unwrap();
                    let protocol = location.protocol().unwrap();

                    let full_url = format!("{protocol}//{host}/ws/{game_id}");

                    let roll = self.start_roll;
                    if let Some(roll) = roll {
                        spawn_local(async move {
                            let req = Request::post(&full_url)
                                .header("Content-Type", "application/json")
                                .body(serde_json::to_string(&roll).unwrap())
                                .send()
                                .await
                                .unwrap();

                            if req.status() == 200 {
                                navigator.push(&Route::PvP { id: game_id });
                            };
                        });
                    }
                }

                true
            }
            Msg::NewPvpGame(num) => {
                let navigator = ctx.link().navigator().unwrap();

                let game_id = nanoid!(8);
                let location = web_sys::window().unwrap().location();

                let host = location.host().unwrap();

                let protocol = location.protocol().unwrap();

                let full_url = format!("{protocol}//{host}/ws/{game_id}");

                spawn_local(async move {
                    let req = Request::post(&full_url)
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&num).unwrap())
                        .send()
                        .await
                        .unwrap();

                    if req.status() == 200 {
                        navigator.push(&Route::PvP { id: game_id });
                    };
                });

                true
            }
            Msg::NewPveGame(num) => {
                let navigator = ctx.link().navigator().unwrap();

                navigator.push(&Route::PvE { roll: num });
                true
            }
            Msg::NewPveGameCustom => {
                if self.start_roll != Some(1) {
                    let navigator = ctx.link().navigator().unwrap();

                    let roll = self.start_roll;
                    if let Some(roll) = roll {
                        navigator.push(&Route::PvE { roll })
                    };
                }
                true
            }
            Msg::DoNothing => true,
        }
    }
}

fn pvp_roll(num: u32, ctx: &yew::Context<Home>) -> Callback<MouseEvent> {
    ctx.link()
        .callback(move |_: MouseEvent| Msg::NewPvpGame(num))
}

fn pve_roll(num: u32, ctx: &yew::Context<Home>) -> Callback<MouseEvent> {
    ctx.link()
        .callback(move |_: MouseEvent| Msg::NewPveGame(num))
}
