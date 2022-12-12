use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

pub enum Msg {
    DoNothing,
}

pub struct PvPComponent {}

impl Component for PvPComponent {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        html! {
         <div class="app-body">
         <header class="header">
         <div>
         <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
         <h1>{"1v1 me bruv"}</h1>
         </div>
        </header>
        </div>
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DoNothing => {
                log::debug!("Do nothing");
                true
            }
        }
    }
}
