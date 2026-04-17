use dioxus::prelude::*;

pub mod components;
pub mod domain;
pub mod infra;
pub mod layouts;
pub mod pages;
pub mod routes;
pub mod services;
pub mod state;
pub mod transport;

pub use routes::Route;
pub use state::skills_state::SkillsStateStore;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let skills_state = use_signal(state::skills_state::SkillsState::default);
    use_context_provider(|| skills_state);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}
