use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[function_component(Notfound)]
pub fn notfound() -> Html {
    // let roll_emoji = '\u{1F3B2}';
    // let skull = '\u{1F480}';

    let navigator = use_navigator().unwrap();
    let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

    html! {
    <div>
       <header>
       <button onclick={home} class="title-button">{"deathroll.gg "}{"\u{1F3E0}"}</button>
       {" "}<a href="https://github.com/stum0/deathroll"><i class="fab fa-github-square" style="font-size:25px"></i></a>

       </header>
       <h1>{"404, YOU ARE LOST!!"}</h1>
       <footer>


       </footer>
    </div>
    }
}
