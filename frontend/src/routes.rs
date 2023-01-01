use yew::{html, Html};
use yew_router::prelude::*;

use crate::{homepage::Home, pve_component::PvEComponent, pvp_component::PvPComponent, notfound::Notfound};

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/pve/:roll")]
    PvE { roll: u32 },
    #[at("/:id/")]
    PvP { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}
#[allow(dead_code)]
pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Home />},
        Route::PvE { roll } => html! {<PvEComponent />},
        Route::PvP { id} => html! {<PvPComponent />},
        Route::NotFound => html! {<Notfound />},
    }
}
