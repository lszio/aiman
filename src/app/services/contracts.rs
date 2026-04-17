use crate::app::domain::skill::{SkillDTO, SkillInstallRecord, SkillManifest, SkillRemoteIndex};

pub trait SkillManager {
    fn list_installed(&self) -> Result<Vec<SkillDTO>, String>;
    fn install(&self, skill_id: String, version: Option<String>) -> Result<SkillDTO, String>;
    fn uninstall(&self, skill_id: String) -> Result<(), String>;
    fn update_manifest(&self, skill_id: String, manifest: SkillManifest) -> Result<SkillDTO, String>;
}

pub trait StoreProvider {
    fn refresh_index(&self) -> Result<Vec<SkillRemoteIndex>, String>;
    fn get_index(&self) -> Result<Vec<SkillRemoteIndex>, String>;
}

pub trait SyncProvider {
    fn push_install_record(&self, record: SkillInstallRecord) -> Result<SkillInstallRecord, String>;
    fn list_install_records(&self) -> Result<Vec<SkillInstallRecord>, String>;
}
