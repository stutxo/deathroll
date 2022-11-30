use deathroll_component::DeathRollComponent;
use yew::prelude::*;

mod deathroll_component;

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
        <div>
        <DeathRollComponent/>
        </div>
        </>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("App is starting");
    yew::Renderer::<App>::new().render();
}
