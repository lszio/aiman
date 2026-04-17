use dioxus::prelude::*;

use crate::app::domain::skill::AgentSettingsDTO;
use crate::app::transport::{
    skills_local_agent_apps, skills_local_agent_settings, skills_local_rebuild_links, skills_local_rebuild_links_for_app,
    skills_local_save_agent_settings,
};

#[component]
pub fn Apps() -> Element {
    rsx! { AppsPage {} }
}

#[component]
pub fn AppsPage() -> Element {
    let mut message = use_signal(String::new);
    let mut registry_json = use_signal(String::new);
    let mut whitelist = use_signal(String::new);
    let mut blacklist = use_signal(String::new);
    let apps = use_resource(move || async move { skills_local_agent_apps().await.unwrap_or_default() });
    let settings = use_resource(move || async move { skills_local_agent_settings().await.ok() });

    if let Some(Some(cfg)) = settings.cloned() {
        if registry_json().is_empty() {
            registry_json.set(cfg.agents_registry_json);
            whitelist.set(cfg.whitelist.join(","));
            blacklist.set(cfg.blacklist.join(","));
        }
    }

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Agent Apps" }
                p { "Auto-discovered app skill directories and link management." }
            }
            button {
                class: "btn-primary",
                onclick: move |_| async move {
                    match skills_local_rebuild_links().await {
                        Ok(_) => message.set("rebuilt all skill symlinks".to_string()),
                        Err(err) => message.set(format!("rebuild failed: {err}")),
                    }
                },
                "Rescan & Rebuild Links"
            }
            if !message().is_empty() {
                p { class: "status-message", "{message}" }
            }
            div { class: "editor-form",
                h3 { "Agent Discovery Settings" }
                label { "Whitelist (comma separated app names)" }
                input {
                    value: whitelist(),
                    placeholder: "opencode,cline",
                    oninput: move |evt| whitelist.set(evt.value()),
                }
                label { "Blacklist (comma separated app names)" }
                input {
                    value: blacklist(),
                    placeholder: "example_agent",
                    oninput: move |evt| blacklist.set(evt.value()),
                }
                label { "agents.json" }
                textarea {
                    value: registry_json(),
                    placeholder: "[{{\"app_name\":\"opencode\",\"skills_dir\":\"/home/me/.opencode/skills\"}}]",
                    oninput: move |evt| registry_json.set(evt.value()),
                }
                button {
                    class: "btn-primary",
                    onclick: move |_| async move {
                        let settings = AgentSettingsDTO {
                            agents_registry_json: registry_json(),
                            whitelist: split_csv(whitelist()),
                            blacklist: split_csv(blacklist()),
                        };
                        match skills_local_save_agent_settings(settings).await {
                            Ok(_) => message.set("saved agent settings".to_string()),
                            Err(err) => message.set(format!("save failed: {err}")),
                        }
                    },
                    "Save Settings"
                }
            }
            div { class: "records-list",
                for app in apps.cloned().unwrap_or_default() {
                    article { class: "detail-card",
                        h3 { "{app.app_name}" }
                        p { class: "skill-meta", "Skills dir: {app.skills_dir}" }
                        button {
                            class: "btn-primary",
                            onclick: {
                                let app_name = app.app_name.clone();
                                move |_| {
                                    let app_name_inner = app_name.clone();
                                    async move {
                                        match skills_local_rebuild_links_for_app(app_name_inner.clone()).await {
                                            Ok(_) => message.set(format!("rebuilt links for {app_name_inner}")),
                                            Err(err) => message.set(format!("rebuild failed: {err}")),
                                        }
                                    }
                                }
                            },
                            "Rebuild This App"
                        }
                    }
                }
            }
        }
    }
}

fn split_csv(input: String) -> Vec<String> {
    input
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}
