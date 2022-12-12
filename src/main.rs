use homepage_component::Home;
use pve_component::PvEComponent;
use notfound_component::{Notfound};
use yew::prelude::*;
use yew_router::prelude::*;

mod homepage_component;
mod pve_component;
mod notfound_component;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/pve")]
    PvE,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Home />},
        Route::PvE => html! {<PvEComponent />},
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
