use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillInstallState {
    NotInstalled,
    Installing,
    Installed,
    Updating,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub entry: String,
    pub prompts: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRemoteIndex {
    pub skill_id: String,
    pub latest_version: String,
    pub download_url: String,
    pub checksum: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillInstallRecord {
    pub user_id: String,
    pub skill_id: String,
    pub installed_version: String,
    pub install_state: SkillInstallState,
    pub sync_version: i64,
    pub last_synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillDTO {
    pub manifest: SkillManifest,
    pub local_state: SkillInstallState,
    pub remote_latest_version: Option<String>,
    pub sync_status: Option<String>,
}
