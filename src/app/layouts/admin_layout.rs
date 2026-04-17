use dioxus::prelude::*;

use crate::app::routes::Route;

#[component]
pub fn AdminLayout() -> Element {
    rsx! {
        div { class: "admin-shell",
            aside { class: "admin-sidebar",
                h2 { "AI Manager" }
                p { class: "sidebar-subtitle", "Skill Management V1" }
                nav {
                    ul {
                        li { Link { to: Route::SkillsList {}, "Installed Skills" } }
                        li { Link { to: Route::SkillsStore {}, "Skill Store" } }
                    }
                }
            }
            main { class: "admin-content",
                Outlet::<Route> {}
            }
        }
    }
}
