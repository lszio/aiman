use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::domain::skill::{SkillDTO, SkillInstallState, SkillManifest};
use crate::app::infra::{local_fs_repo, manifest_parser};
use crate::app::services::store_service;

fn version_is_newer(current: &str, latest: &str) -> bool {
    current != latest
}

pub fn now_rfc3339() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => format!("{}", dur.as_secs()),
        Err(_) => "0".to_string(),
    }
}

pub fn list_local_skills() -> Result<Vec<SkillDTO>, String> {
    local_fs_repo::synchronize_skill_links().map_err(|err| err.to_string())?;
    let manifests = local_fs_repo::list_skill_manifests().map_err(|err| err.to_string())?;
    let store_index = store_service::get_store_index().unwrap_or_default();
    let out = manifests
        .into_iter()
        .map(|manifest| {
            let remote = store_index
                .iter()
                .find(|item| item.skill_id == manifest.id)
                .map(|item| item.latest_version.clone());
            let local_state = match remote.as_deref() {
                Some(latest) if version_is_newer(&manifest.version, latest) => SkillInstallState::Updating,
                _ => SkillInstallState::Installed,
            };
            SkillDTO {
                manifest,
                local_state,
                remote_latest_version: remote,
                sync_status: None,
            }
        })
        .collect();
    Ok(out)
}

pub fn get_local_skill(skill_id: &str) -> Result<Option<SkillDTO>, String> {
    let manifest = local_fs_repo::get_skill_manifest(skill_id).map_err(|err| err.to_string())?;
    match manifest {
        Some(manifest) => Ok(Some(SkillDTO {
            manifest,
            local_state: SkillInstallState::Installed,
            remote_latest_version: None,
            sync_status: None,
        })),
        None => Ok(None),
    }
}

pub fn install_skill(skill_id: String, requested_version: Option<String>) -> Result<SkillDTO, String> {
    let version = requested_version.unwrap_or_else(|| "0.1.0".to_string());
    let manifest = SkillManifest {
        id: skill_id.clone(),
        name: skill_id.replace('-', " "),
        version,
        entry: "main".to_string(),
        prompts: vec!["default_prompt".to_string()],
        required_capabilities: vec!["filesystem".to_string()],
        enabled: true,
        updated_at: now_rfc3339(),
    };
    let manifest = manifest_parser::normalize_manifest(manifest);
    local_fs_repo::save_skill_manifest(&manifest).map_err(|err| err.to_string())?;
    local_fs_repo::synchronize_skill_links().map_err(|err| err.to_string())?;
    Ok(SkillDTO {
        manifest,
        local_state: SkillInstallState::Installed,
        remote_latest_version: None,
        sync_status: Some("local_saved".to_string()),
    })
}

pub fn uninstall_skill(skill_id: String) -> Result<(), String> {
    local_fs_repo::remove_skill_manifest(&skill_id).map_err(|err| err.to_string())?;
    local_fs_repo::synchronize_skill_links().map_err(|err| err.to_string())
}

pub fn set_skill_enabled(skill_id: String, enabled: bool) -> Result<SkillDTO, String> {
    let existing = local_fs_repo::get_skill_manifest(&skill_id).map_err(|err| err.to_string())?;
    let mut manifest = existing.ok_or_else(|| format!("skill_not_found: {skill_id}"))?;
    manifest.enabled = enabled;
    manifest.updated_at = now_rfc3339();
    local_fs_repo::save_skill_manifest(&manifest).map_err(|err| err.to_string())?;
    local_fs_repo::synchronize_skill_links().map_err(|err| err.to_string())?;
    Ok(SkillDTO {
        manifest,
        local_state: SkillInstallState::Installed,
        remote_latest_version: None,
        sync_status: Some("toggled".to_string()),
    })
}

pub fn update_skill_manifest(skill_id: String, updated: SkillManifest) -> Result<SkillDTO, String> {
    if skill_id != updated.id {
        return Err("skill_id_mismatch".to_string());
    }
    let normalized = manifest_parser::normalize_manifest(SkillManifest {
        updated_at: now_rfc3339(),
        ..updated
    });
    local_fs_repo::save_skill_manifest(&normalized).map_err(|err| err.to_string())?;
    local_fs_repo::synchronize_skill_links().map_err(|err| err.to_string())?;
    Ok(SkillDTO {
        manifest: normalized,
        local_state: SkillInstallState::Installed,
        remote_latest_version: None,
        sync_status: Some("manifest_updated".to_string()),
    })
}
