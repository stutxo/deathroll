use yew::{html, Html};
use yew_router::prelude::*;

use crate::{homepage::Home, notfound::Notfound, pve_component::PvEComponent, pvp_component::PvPComponent};

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/pve")]
    PvE,
    #[at("/:id/:roll")]
    PvP { id: String, roll: u32 },
    // #[not_found]
    // // #[at("/404")]
    // // NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Home />},
        Route::PvE => html! {<PvEComponent />},
        Route::PvP { id, roll } => html! {<PvPComponent />},
        //Route::NotFound => html! {<Notfound />},
    }
}
