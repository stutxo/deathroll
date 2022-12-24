use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;


#[function_component(Notfound)]
pub fn notfound() -> Html {
    
    let roll_emoji = '\u{1F3B2}';
    let skull = '\u{1F480}';

    let navigator = use_navigator().unwrap();
    let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));


    html! {
    <div class="app-body">
       <header class="header">
       <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
       </header>
       <h1 class="text">{"404, YOU ARE LOST!!"}</h1>
       <footer class="nav-bar-bottom">


       </footer>
    </div>
    }
}