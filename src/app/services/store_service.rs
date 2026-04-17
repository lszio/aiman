use crate::app::domain::skill::SkillRemoteIndex;
use crate::app::infra::local_fs_repo;

pub fn refresh_store_index() -> Result<Vec<SkillRemoteIndex>, String> {
    local_fs_repo::refresh_store_index_cache().map_err(|err| err.to_string())
}

pub fn get_store_index() -> Result<Vec<SkillRemoteIndex>, String> {
    local_fs_repo::get_store_index_cache().map_err(|err| err.to_string())
}
