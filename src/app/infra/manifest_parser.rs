use crate::app::domain::skill::SkillManifest;

pub fn normalize_manifest(mut manifest: SkillManifest) -> SkillManifest {
    if manifest.entry.trim().is_empty() {
        manifest.entry = "main".to_string();
    }
    if manifest.updated_at.trim().is_empty() {
        manifest.updated_at = crate::app::services::skill_service::now_rfc3339();
    }
    manifest
}
