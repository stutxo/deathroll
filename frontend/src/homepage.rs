use nanoid::nanoid;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{routes::Route, ws::WebsocketService};

pub struct Home {
    rules: bool,
    input: NodeRef,
    input_pve: NodeRef,
    pub start_roll: Option<u32>,
    pub start_roll_pve: Option<u32>,
    ws: Option<WebsocketService>,
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
            ws: None,
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';

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
           <button onclick={home} class="title-button">{"deathroll.gg \u{1F3E0}"}</button>
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
                if self.rules == false {
                    self.rules = true
                } else if self.rules == true {
                    self.rules = false
                }
            }
            Msg::HideRules => self.rules = false,
            Msg::Input(msg) => {
                let start_roll: u32 = match msg.trim().parse::<u32>() {
                    Ok(parsed_input) => parsed_input,

                    Err(_) => 1,
                };

                self.start_roll = Some(start_roll);
            }
            Msg::NewPvpGameCustom => {
                if self.start_roll != Some(1) {
                    let navigator = ctx.link().navigator().unwrap();

                    let game_id = nanoid!(8);
                    let location = web_sys::window().unwrap().location();

                    let host = location.host().unwrap();
                    let protocol = location.protocol().unwrap();
                    let ws_protocol = match protocol.as_str() {
                        "https:" => "wss:",
                        _ => "ws:",
                    };

                    let full_url = format!("{ws_protocol}//{host}/ws/{game_id}");

                    self.ws = Some(WebsocketService::ws_connect(&full_url));
                    let ws = self.ws.clone();
                    ws.unwrap()
                        .tx
                        .try_send(self.start_roll.unwrap().to_string())
                        .unwrap();

                    let roll = self.start_roll;
                    match roll {
                        Some(roll) => navigator.push(&Route::PvP { id: game_id }),
                        None => {}
                    }
                } else {
                    //log::debug!("ERROR");
                }
            }
            Msg::NewPvpGame(num) => {
                let navigator = ctx.link().navigator().unwrap();

                let game_id = nanoid!(8);
                let location = web_sys::window().unwrap().location();

                let host = location.host().unwrap();
                let protocol = location.protocol().unwrap();
                let ws_protocol = match protocol.as_str() {
                    "https:" => "wss:",
                    _ => "ws:",
                };

                let full_url = format!("{ws_protocol}//{host}/ws/{game_id}");
                self.ws = Some(WebsocketService::ws_connect(&full_url));
                let ws = self.ws.clone();
                ws.unwrap().tx.try_send(num.to_string()).unwrap();

                navigator.push(&Route::PvP { id: game_id })
            }
            Msg::NewPveGame(num) => {
                let navigator = ctx.link().navigator().unwrap();

                navigator.push(&Route::PvE { roll: num })
            }
            Msg::NewPveGameCustom => {
                if self.start_roll != Some(1) {
                    let navigator = ctx.link().navigator().unwrap();

                    let roll = self.start_roll;
                    match roll {
                        Some(roll) => navigator.push(&Route::PvE { roll: roll }),
                        None => {}
                    }
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
    fn destroy(&mut self, _ctx: &yew::Context<Self>) {
        let ws = self.ws.clone();
        match ws {
            Some(mut ws) => {
                ws.tx.try_send("close".to_string()).unwrap();
            }
            None => {}
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
