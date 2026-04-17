use dioxus::prelude::*;

use crate::app::state::skills_state::SkillsStateStore;
use crate::app::transport::{skills_local_install, skills_store_refresh};

#[component]
pub fn SkillsStore() -> Element {
    rsx! { SkillsStorePage {} }
}

#[component]
pub fn SkillsStorePage() -> Element {
    let mut state = use_context::<SkillsStateStore>();
    let mut message = use_signal(String::new);

    let store_index = use_resource(move || async move {
        match skills_store_refresh().await {
            Ok(items) => {
                state.with_mut(|snapshot| snapshot.store_index = items.clone());
                items
            }
            Err(_) => Vec::new(),
        }
    });

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Skill Store" }
                p { "Pull remote index and install skills from the registry cache." }
            }
            if !message().is_empty() {
                p { class: "status-message", "{message}" }
            }
            div { class: "skills-grid",
                for item in store_index.cloned().unwrap_or_default() {
                    article { class: "skill-card",
                        div { class: "skill-card-header",
                            h3 { "{item.skill_id}" }
                            span { class: "skill-badge", "Store" }
                        }
                        p { class: "skill-meta", "Latest: {item.latest_version}" }
                        p { class: "skill-meta", "Published at: {item.published_at}" }
                        p { class: "skill-meta", "Checksum: {item.checksum}" }
                        button {
                            class: "btn-primary",
                            onclick: {
                                let skill_id = item.skill_id.clone();
                                let version = item.latest_version.clone();
                                move |_| {
                                    let skill_id_inner = skill_id.clone();
                                    let version_inner = version.clone();
                                    async move {
                                        match skills_local_install(skill_id_inner.clone(), Some(version_inner.clone())).await {
                                            Ok(_) => message.set(format!("installed {skill_id_inner}@{version_inner}")),
                                            Err(err) => message.set(format!("install failed: {err}")),
                                        }
                                    }
                                }
                            },
                            "Install"
                        }
                    }
                }
            }
        }
    }
}
