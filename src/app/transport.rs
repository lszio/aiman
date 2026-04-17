use dioxus::prelude::*;

use crate::app::domain::skill::{SkillDTO, SkillInstallRecord, SkillInstallState, SkillManifest, SkillRemoteIndex};
use crate::app::services::{skill_service, store_service, sync_service};

#[get("/api/skills/local/list")]
pub async fn skills_local_list() -> Result<Vec<SkillDTO>, ServerFnError> {
    skill_service::list_local_skills().map_err(ServerFnError::new)
}

#[get("/api/skills/local/get/:skill_id")]
pub async fn skills_local_get(skill_id: String) -> Result<Option<SkillDTO>, ServerFnError> {
    skill_service::get_local_skill(&skill_id).map_err(ServerFnError::new)
}

#[post("/api/skills/local/install")]
pub async fn skills_local_install(skill_id: String, requested_version: Option<String>) -> Result<SkillDTO, ServerFnError> {
    skill_service::install_skill(skill_id, requested_version).map_err(ServerFnError::new)
}

#[post("/api/skills/local/uninstall")]
pub async fn skills_local_uninstall(skill_id: String) -> Result<(), ServerFnError> {
    skill_service::uninstall_skill(skill_id).map_err(ServerFnError::new)
}

#[post("/api/skills/local/set-enabled")]
pub async fn skills_local_set_enabled(skill_id: String, enabled: bool) -> Result<SkillDTO, ServerFnError> {
    skill_service::set_skill_enabled(skill_id, enabled).map_err(ServerFnError::new)
}

#[post("/api/skills/local/update-manifest")]
pub async fn skills_local_update_manifest(skill_id: String, manifest: SkillManifest) -> Result<SkillDTO, ServerFnError> {
    skill_service::update_skill_manifest(skill_id, manifest).map_err(ServerFnError::new)
}

#[get("/api/skills/store/list")]
pub async fn skills_store_list() -> Result<Vec<SkillRemoteIndex>, ServerFnError> {
    store_service::get_store_index().map_err(ServerFnError::new)
}

#[post("/api/skills/store/refresh")]
pub async fn skills_store_refresh() -> Result<Vec<SkillRemoteIndex>, ServerFnError> {
    store_service::refresh_store_index().map_err(ServerFnError::new)
}

#[post("/api/skills/sync/push")]
pub async fn skills_sync_push(
    user_id: String,
    skill_id: String,
    installed_version: String,
    install_state: SkillInstallState,
) -> Result<SkillInstallRecord, ServerFnError> {
    sync_service::push_install_state(user_id, skill_id, installed_version, install_state)
        .await
        .map_err(ServerFnError::new)
}

#[get("/api/skills/sync/records")]
pub async fn skills_sync_records() -> Result<Vec<SkillInstallRecord>, ServerFnError> {
    sync_service::list_sync_records()
        .await
        .map_err(ServerFnError::new)
}

#[get("/api/skills/sync/remote-versions")]
pub async fn skills_sync_remote_versions() -> Result<Vec<SkillRemoteIndex>, ServerFnError> {
    sync_service::pull_remote_versions()
        .await
        .map_err(ServerFnError::new)
}

#[get("/api/skills/sync/backend-status")]
pub async fn skills_sync_backend_status() -> Result<String, ServerFnError> {
    Ok(sync_service::get_sync_backend_status().await)
}
