use homepage::Home;
use pve_component::PvEComponent;
use pvp_component::PvPComponent;
use notfound::{Notfound};
use yew::prelude::*;
use yew_router::prelude::*;

mod homepage;
mod pve_component;
mod notfound;
mod pvp_component;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/pve")]
    PvE,
    #[at("/pvp")]
    PvP,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Home />},
        Route::PvE => html! {<PvEComponent />},
        Route::PvP => html! {<PvPComponent />},
        Route::NotFound => html! {<Notfound />},
    }
}

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
