use dioxus::prelude::*;

use crate::app::domain::skill::SkillDTO;
use crate::app::routes::Route;

#[component]
pub fn SkillCard(skill: SkillDTO) -> Element {
    let id = skill.manifest.id.clone();
    let state_text = match &skill.local_state {
        crate::app::domain::skill::SkillInstallState::NotInstalled => "NotInstalled",
        crate::app::domain::skill::SkillInstallState::Installing => "Installing",
        crate::app::domain::skill::SkillInstallState::Installed => "Installed",
        crate::app::domain::skill::SkillInstallState::Updating => "Updating",
        crate::app::domain::skill::SkillInstallState::Error(_) => "Error",
    };
    rsx! {
        article { class: "skill-card",
            div { class: "skill-card-header",
                h3 { "{skill.manifest.name}" }
                span { class: "skill-badge", "{state_text}" }
            }
            p { class: "skill-meta", "ID: {skill.manifest.id}" }
            p { class: "skill-meta", "Version: {skill.manifest.version}" }
            p { class: "skill-meta", "Enabled: {skill.manifest.enabled}" }
            if let Some(latest) = skill.remote_latest_version.clone() {
                p { class: "skill-meta", "Remote latest: {latest}" }
            }
            div { class: "skill-links",
                Link { to: Route::SkillDetail { id: id.clone() }, "Detail" }
                Link { to: Route::SkillEditor { id }, "Edit" }
            }
        }
    }
}
