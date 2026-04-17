use crate::app::domain::skill::{SkillInstallRecord, SkillInstallState, SkillRemoteIndex};
use crate::app::infra::turso_repo;
use crate::app::services::store_service;

pub async fn push_install_state(
    user_id: String,
    skill_id: String,
    installed_version: String,
    install_state: SkillInstallState,
) -> Result<SkillInstallRecord, String> {
    let record = SkillInstallRecord {
        user_id,
        skill_id,
        installed_version,
        install_state,
        sync_version: 1,
        last_synced_at: crate::app::services::skill_service::now_rfc3339(),
    };
    turso_repo::push_install_record(record)
        .await
        .map_err(|err| err.to_string())
}

pub async fn list_sync_records() -> Result<Vec<SkillInstallRecord>, String> {
    turso_repo::list_install_records()
        .await
        .map_err(|err| err.to_string())
}

pub async fn pull_remote_versions() -> Result<Vec<SkillRemoteIndex>, String> {
    store_service::get_store_index()
}

pub async fn get_sync_backend_status() -> String {
    turso_repo::sync_backend_status().await
}
