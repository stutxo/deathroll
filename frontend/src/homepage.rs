use nanoid::nanoid;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

#[function_component(Home)]
pub fn home() -> Html {
    let roll_emoji = '\u{1F3B2}';
    let skull = '\u{1F480}';

    let navigator = use_navigator().unwrap();
    let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
    let navigator = use_navigator().unwrap();
    let pve = Callback::from(move |_: MouseEvent| navigator.push(&Route::PvE));
    let navigator = use_navigator().unwrap();
    let pvp = Callback::from(move |_: MouseEvent| {
        let id = nanoid!(8);

        navigator.push(&Route::PvP { id: id })
    });

    html! {
    <div class="app-body">
       <header class="header">
       <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
       <button onclick={pve} class="nav-button">{ "PvE" }</button>
       <button onclick={pvp} class="nav-button">{ "PvP" }</button>
       </header>
       <div class="text">

       {"Players take turns rolling a die. The first player rolls the die and the number they roll becomes the maximum number for the next player's roll."}
       <br/>
       <br/>
       {"For example, if the first player rolls a 4, the second player can roll any number from 1 to 4."}
       <br/>
       <br/>
       {"This continues until a player rolls a 1, at which point they lose the game."}

       </div>


       <footer class="nav-bar-bottom">


       </footer>
    </div>
    }
}
