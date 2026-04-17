use dioxus::prelude::*;

use crate::app::domain::skill::SkillManifest;
use crate::app::transport::{skills_local_get, skills_local_update_manifest};

#[component]
pub fn SkillEditor(id: String) -> Element {
    rsx! { SkillEditorPage { id } }
}

#[component]
pub fn SkillEditorPage(id: String) -> Element {
    let mut message = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut entry = use_signal(String::new);
    let mut prompts = use_signal(String::new);

    let skill = use_resource({
        let id = id.clone();
        move || {
            let id_inner = id.clone();
            async move { skills_local_get(id_inner.clone()).await.ok().flatten() }
        }
    });

    if let Some(Some(current)) = skill.cloned() {
        if name().is_empty() {
            name.set(current.manifest.name.clone());
            entry.set(current.manifest.entry.clone());
            prompts.set(current.manifest.prompts.join(","));
        }
    }

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Edit Skill" }
                p { "Skill id: {id}" }
            }
            if !message().is_empty() {
                p { class: "status-message", "{message}" }
            }
            div { class: "editor-form",
                label { "Name" }
                input {
                    value: name(),
                    placeholder: "name",
                    oninput: move |evt| name.set(evt.value()),
                }
                label { "Entry" }
                input {
                    value: entry(),
                    placeholder: "entry",
                    oninput: move |evt| entry.set(evt.value()),
                }
                label { "Prompts (comma separated)" }
                textarea {
                    value: prompts(),
                    placeholder: "prompt_a,prompt_b",
                    oninput: move |evt| prompts.set(evt.value()),
                }
                button {
                    class: "btn-primary",
                    onclick: {
                        let skill_id = id.clone();
                        move |_| {
                            let skill_id_inner = skill_id.clone();
                            async move {
                                let updated = SkillManifest {
                                    id: skill_id_inner.clone(),
                                    name: name(),
                                    version: "0.1.0".to_string(),
                                    entry: entry(),
                                    prompts: prompts()
                                        .split(',')
                                        .map(|item| item.trim().to_string())
                                        .filter(|item| !item.is_empty())
                                        .collect(),
                                    required_capabilities: vec!["filesystem".to_string()],
                                    enabled: true,
                                    updated_at: String::new(),
                                };
                                match skills_local_update_manifest(skill_id_inner, updated).await {
                                    Ok(_) => message.set("manifest updated".to_string()),
                                    Err(err) => message.set(format!("update failed: {err}")),
                                }
                            }
                        }
                    },
                    "Save"
                }
            }
        }
    }
}
