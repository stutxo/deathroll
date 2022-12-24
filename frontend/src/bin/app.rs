
use frontend::routes::{Route, switch};
use yew::prelude::*;
use yew_router::{BrowserRouter, Switch};

#[function_component(Main)]
fn main() -> Html {
    html! {
        <BrowserRouter>
            <main>
                <Switch<Route> render={switch}/>
            </main>
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("App is starting");
    yew::Renderer::<Main>::new().render();
}
