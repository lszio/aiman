use dioxus::prelude::*;

use crate::app::components::skills::card::SkillCard;
use crate::app::state::skills_state::SkillsStateStore;
use crate::app::transport::{
    skills_local_list, skills_local_set_enabled, skills_local_uninstall, skills_sync_backend_status, skills_sync_push,
};

#[component]
pub fn Skills() -> Element {
    rsx! { SkillsListPage {} }
}

#[component]
pub fn SkillsList() -> Element {
    rsx! { SkillsListPage {} }
}

#[component]
pub fn SkillsListPage() -> Element {
    let mut store = use_context::<SkillsStateStore>();
    let mut message = use_signal(String::new);

    let skills = use_resource(move || async move {
        match skills_local_list().await {
            Ok(items) => {
                store.with_mut(|state| state.installed = items.clone());
                items
            }
            Err(_) => Vec::new(),
        }
    });

    let list = skills.cloned().unwrap_or_default();
    let backend_status = use_resource(move || async move {
        skills_sync_backend_status()
            .await
            .unwrap_or_else(|_| "unknown".to_string())
    });

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Installed Skills" }
                p { "Manage installed skills, toggle status, and sync install state." }
                p { class: "sync-status", "Sync backend: {backend_status.cloned().unwrap_or_else(|| \"loading\".to_string())}" }
            }
            if !message().is_empty() {
                p { class: "status-message", "{message}" }
            }
            div { class: "skills-grid",
                for skill in list {
                    div { class: "skill-item",
                        SkillCard { skill: skill.clone() }
                        div { class: "skill-actions",
                            button {
                                class: "btn-primary",
                                onclick: {
                                    let id = skill.manifest.id.clone();
                                    let enabled = !skill.manifest.enabled;
                                    move |_| {
                                        let id_inner = id.clone();
                                        async move {
                                            match skills_local_set_enabled(id_inner.clone(), enabled).await {
                                                Ok(updated) => {
                                                    let _ = skills_sync_push(
                                                        "demo-user".to_string(),
                                                        updated.manifest.id.clone(),
                                                        updated.manifest.version.clone(),
                                                        updated.local_state.clone(),
                                                    )
                                                    .await;
                                                    message.set(format!("updated {}", updated.manifest.id));
                                                }
                                                Err(err) => message.set(format!("toggle failed: {err}")),
                                            }
                                        }
                                    }
                                },
                                if skill.manifest.enabled { "Disable" } else { "Enable" }
                            }
                            button {
                                class: "btn-danger",
                                onclick: {
                                    let id = skill.manifest.id.clone();
                                    move |_| {
                                        let id_inner = id.clone();
                                        async move {
                                            match skills_local_uninstall(id_inner.clone()).await {
                                                Ok(_) => message.set(format!("uninstalled {id_inner}")),
                                                Err(err) => message.set(format!("uninstall failed: {err}")),
                                            }
                                        }
                                    }
                                },
                                "Uninstall"
                            }
                        }
                    }
                }
            }
        }
    }
}
