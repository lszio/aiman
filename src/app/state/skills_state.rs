use dioxus::prelude::Signal;

use crate::app::domain::skill::{SkillDTO, SkillInstallRecord, SkillRemoteIndex};

pub type SkillsStateStore = Signal<SkillsState>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillsState {
    pub installed: Vec<SkillDTO>,
    pub store_index: Vec<SkillRemoteIndex>,
    pub sync_records: Vec<SkillInstallRecord>,
    pub sync_status: Option<String>,
}
