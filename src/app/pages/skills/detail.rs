use dioxus::prelude::*;

use crate::app::domain::skill::SkillDTO;
use crate::app::transport::{skills_local_get, skills_sync_records};

#[component]
pub fn SkillDetail(id: String) -> Element {
    rsx! { SkillDetailPage { id } }
}

#[component]
pub fn SkillDetailPage(id: String) -> Element {
    let skill_resource = use_resource({
        let id = id.clone();
        move || {
            let id_inner = id.clone();
            async move { skills_local_get(id_inner.clone()).await.ok().flatten() }
        }
    });
    let sync_resource = use_resource(move || async move { skills_sync_records().await.unwrap_or_default() });

    let skill: Option<SkillDTO> = skill_resource.cloned().unwrap_or(None);
    let records = sync_resource.cloned().unwrap_or_default();

    rsx! {
        section { class: "page-section",
            div { class: "page-header",
                h1 { "Skill Detail" }
                p { "Skill id: {id}" }
            }
            match skill {
                Some(skill) => rsx! { div { class: "detail-card",
                    h2 { "{skill.manifest.name}" }
                    p { class: "skill-meta", "Version: {skill.manifest.version}" }
                    p { class: "skill-meta", "Entry: {skill.manifest.entry}" }
                    p { class: "skill-meta", "Enabled: {skill.manifest.enabled}" }
                    p { class: "skill-meta", "Prompts: {skill.manifest.prompts.join(\", \")}" }
                }},
                None => rsx! { p { class: "status-message", "Skill not installed." } },
            }
            h3 { "Sync Records" }
            div { class: "records-list",
                for record in records.into_iter().filter(|item| item.skill_id == id) {
                    p { class: "record-item", "{record.user_id} -> {record.installed_version} @ {record.last_synced_at}" }
                }
            }
        }
    }
}
