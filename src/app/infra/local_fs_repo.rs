use std::env;
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use crate::app::domain::skill::{AgentAppDTO, AgentSettingsDTO, SkillManifest, SkillRemoteIndex};
use serde::{Deserialize, Serialize};

const AIMAN_DIR: &str = ".aiman";
const SKILLS_DIR_NAME: &str = "skills";
const MANAGED_DIR_NAME: &str = "_aiman_managed";
const STORE_CACHE_FILE_NAME: &str = "store_index_cache.json";
const STORE_REMOTE_FILE_NAME: &str = "store_remote_index.json";
const AGENTS_REGISTRY_FILE_NAME: &str = "agents.json";
const AGENTS_SETTINGS_FILE_NAME: &str = "agents_settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentRegistration {
    app_name: String,
    skills_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AgentDiscoverySettings {
    whitelist: Vec<String>,
    blacklist: Vec<String>,
}

impl From<AgentRegistration> for AgentAppDTO {
    fn from(value: AgentRegistration) -> Self {
        Self {
            app_name: value.app_name,
            skills_dir: value.skills_dir,
        }
    }
}

fn home_dir() -> io::Result<PathBuf> {
    let home = env::var("HOME").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home))
}

fn aiman_root_dir() -> io::Result<PathBuf> {
    Ok(home_dir()?.join(AIMAN_DIR))
}

fn skills_root_dir() -> io::Result<PathBuf> {
    Ok(aiman_root_dir()?.join(SKILLS_DIR_NAME))
}

fn managed_skills_dir() -> io::Result<PathBuf> {
    Ok(skills_root_dir()?.join(MANAGED_DIR_NAME))
}

fn store_cache_file() -> io::Result<PathBuf> {
    Ok(aiman_root_dir()?.join(STORE_CACHE_FILE_NAME))
}

fn store_remote_file() -> io::Result<PathBuf> {
    Ok(aiman_root_dir()?.join(STORE_REMOTE_FILE_NAME))
}

fn agent_registry_file() -> io::Result<PathBuf> {
    Ok(aiman_root_dir()?.join(AGENTS_REGISTRY_FILE_NAME))
}

fn agent_settings_file() -> io::Result<PathBuf> {
    Ok(aiman_root_dir()?.join(AGENTS_SETTINGS_FILE_NAME))
}

fn ensure_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn skill_path(skill_id: &str) -> PathBuf {
    managed_skills_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(skill_id)
        .join("manifest.json")
}

fn central_link_path(skill_id: &str) -> io::Result<PathBuf> {
    Ok(skills_root_dir()?.join(skill_id))
}

fn maybe_symlink_manifest(path: &Path) -> Option<PathBuf> {
    let manifest_path = path.join("manifest.json");
    if manifest_path.exists() {
        Some(manifest_path)
    } else {
        None
    }
}

#[cfg(unix)]
fn create_or_replace_symlink(src: &Path, dst: &Path) -> io::Result<()> {
    if let Ok(meta) = fs::symlink_metadata(dst) {
        if meta.is_dir() && !meta.file_type().is_symlink() {
            fs::remove_dir_all(dst)?;
        } else {
            fs::remove_file(dst)?;
        }
    }
    ensure_parent(dst)?;
    unix_fs::symlink(src, dst)?;
    Ok(())
}

#[cfg(not(unix))]
fn create_or_replace_symlink(_src: &Path, _dst: &Path) -> io::Result<()> {
    Ok(())
}

fn discover_agent_registrations() -> io::Result<Vec<AgentRegistration>> {
    let settings = load_discovery_settings().unwrap_or_default();
    let mut apps = Vec::new();
    let registry = agent_registry_file()?;
    if registry.exists() {
        let payload = fs::read_to_string(registry)?;
        if let Ok(parsed) = serde_json::from_str::<Vec<AgentRegistration>>(&payload) {
            apps.extend(parsed.into_iter().filter(|item| allow_app(&settings, &item.app_name)));
        }
    }

    // Auto-scan hidden app folders, e.g. ~/.opencode/skills
    let home = home_dir()?;
    for entry in fs::read_dir(home)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with('.') || name == AIMAN_DIR {
            continue;
        }
        let app_name = name.trim_start_matches('.').to_string();
        if app_name.is_empty() {
            continue;
        }
        if !allow_app(&settings, &app_name) {
            continue;
        }
        let skills_dir = path.join(SKILLS_DIR_NAME);
        if skills_dir.exists() && skills_dir.is_dir() {
            apps.push(AgentRegistration {
                app_name,
                skills_dir: skills_dir.to_string_lossy().to_string(),
            });
        }
    }

    apps.sort_by(|a, b| a.app_name.cmp(&b.app_name));
    apps.dedup_by(|a, b| a.app_name == b.app_name && a.skills_dir == b.skills_dir);
    Ok(apps)
}

fn allow_app(settings: &AgentDiscoverySettings, app_name: &str) -> bool {
    if settings.blacklist.iter().any(|item| item == app_name) {
        return false;
    }
    if settings.whitelist.is_empty() {
        return true;
    }
    settings.whitelist.iter().any(|item| item == app_name)
}

fn load_discovery_settings() -> io::Result<AgentDiscoverySettings> {
    let path = agent_settings_file()?;
    if !path.exists() {
        return Ok(AgentDiscoverySettings::default());
    }
    let payload = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<AgentDiscoverySettings>(&payload).unwrap_or_default();
    Ok(parsed)
}

pub fn get_agent_settings() -> io::Result<AgentSettingsDTO> {
    let settings = load_discovery_settings()?;
    let registry_file = agent_registry_file()?;
    let registry_json = if registry_file.exists() {
        fs::read_to_string(registry_file)?
    } else {
        "[]".to_string()
    };
    Ok(AgentSettingsDTO {
        agents_registry_json: registry_json,
        whitelist: settings.whitelist,
        blacklist: settings.blacklist,
    })
}

pub fn save_agent_settings(settings: AgentSettingsDTO) -> io::Result<()> {
    let root = aiman_root_dir()?;
    fs::create_dir_all(&root)?;

    let registry_path = agent_registry_file()?;
    fs::write(registry_path, settings.agents_registry_json)?;

    let discover_settings = AgentDiscoverySettings {
        whitelist: settings.whitelist,
        blacklist: settings.blacklist,
    };
    let payload = serde_json::to_string_pretty(&discover_settings)?;
    fs::write(agent_settings_file()?, payload)?;
    Ok(())
}

pub fn list_agent_apps() -> io::Result<Vec<AgentAppDTO>> {
    let apps = discover_agent_registrations()?;
    Ok(apps.into_iter().map(AgentAppDTO::from).collect())
}

fn managed_skill_ids() -> io::Result<Vec<String>> {
    let mut out = Vec::new();
    let managed_dir = managed_skills_dir()?;
    fs::create_dir_all(&managed_dir)?;
    for entry in fs::read_dir(managed_dir)? {
        let entry = entry?;
        if entry.path().join("manifest.json").exists() {
            out.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(out)
}

pub fn synchronize_skill_links() -> io::Result<()> {
    synchronize_skill_links_internal(None)
}

pub fn synchronize_skill_links_for_app(app_name: &str) -> io::Result<()> {
    synchronize_skill_links_internal(Some(app_name))
}

fn synchronize_skill_links_internal(target_app: Option<&str>) -> io::Result<()> {
    let skills_root = skills_root_dir()?;
    let managed_root = managed_skills_dir()?;
    fs::create_dir_all(&skills_root)?;
    fs::create_dir_all(&managed_root)?;

    // 1) Aiman managed skills -> central links and agent links.
    for skill_id in managed_skill_ids()? {
        let managed_path = managed_root.join(&skill_id);
        create_or_replace_symlink(&managed_path, &central_link_path(&skill_id)?)?;
    }

    let managed_ids = managed_skill_ids()?;
    for app in discover_agent_registrations()? {
        if let Some(target) = target_app {
            if app.app_name != target {
                continue;
            }
        }
        let agent_dir = PathBuf::from(app.skills_dir.clone());
        fs::create_dir_all(&agent_dir)?;
        for skill_id in &managed_ids {
            let src = managed_root.join(skill_id);
            let dst = agent_dir.join(skill_id);
            create_or_replace_symlink(&src, &dst)?;
        }
    }

    // 2) Agent owned skills -> central prefixed links, e.g. opencode-create-skill.
    let skills_root_canonical = fs::canonicalize(&skills_root).unwrap_or(skills_root.clone());
    for app in discover_agent_registrations()? {
        if let Some(target) = target_app {
            if app.app_name != target {
                continue;
            }
        }
        let agent_dir = PathBuf::from(app.skills_dir.clone());
        if !agent_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&agent_dir)? {
            let entry = entry?;
            let source_dir = entry.path();
            let source_canonical = fs::canonicalize(&source_dir).unwrap_or(source_dir.clone());
            if source_canonical.starts_with(&skills_root_canonical) {
                continue;
            }
            let Some(skill_name) = source_dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !source_dir.is_dir() || maybe_symlink_manifest(&source_dir).is_none() {
                continue;
            }
            let prefixed = format!("{}-{}", app.app_name, skill_name);
            let dst = skills_root.join(prefixed);
            create_or_replace_symlink(&source_dir, &dst)?;
        }
    }
    Ok(())
}

pub fn list_skill_manifests() -> io::Result<Vec<SkillManifest>> {
    synchronize_skill_links()?;
    let skills_root = skills_root_dir()?;
    fs::create_dir_all(&skills_root)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(skills_root)? {
        let dir = entry?;
        let name = dir.file_name().to_string_lossy().to_string();
        if name.starts_with('_') {
            continue;
        }
        let manifest_path = dir.path().join("manifest.json");
        if manifest_path.exists() {
            let payload = fs::read_to_string(manifest_path)?;
            if let Ok(manifest) = serde_json::from_str::<SkillManifest>(&payload) {
                out.push(manifest);
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn get_skill_manifest(skill_id: &str) -> io::Result<Option<SkillManifest>> {
    let path = skill_path(skill_id);
    let fallback = central_link_path(skill_id)?.join("manifest.json");
    let candidate = if path.exists() { path } else { fallback };
    if !candidate.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(candidate)?;
    let parsed = serde_json::from_str::<SkillManifest>(&payload)?;
    Ok(Some(parsed))
}

pub fn save_skill_manifest(manifest: &SkillManifest) -> io::Result<()> {
    let path = skill_path(&manifest.id);
    ensure_parent(&path)?;
    let payload = serde_json::to_string_pretty(manifest)?;
    fs::write(path, payload)?;
    let managed_root = managed_skills_dir()?;
    create_or_replace_symlink(&managed_root.join(&manifest.id), &central_link_path(&manifest.id)?)?;
    synchronize_skill_links()?;
    Ok(())
}

pub fn remove_skill_manifest(skill_id: &str) -> io::Result<()> {
    let folder = managed_skills_dir()?.join(skill_id);
    if folder.exists() {
        fs::remove_dir_all(folder)?;
    }
    let central = central_link_path(skill_id)?;
    if central.exists() {
        let meta = fs::symlink_metadata(&central)?;
        if meta.is_dir() && !meta.file_type().is_symlink() {
            fs::remove_dir_all(&central)?;
        } else {
            fs::remove_file(&central)?;
        }
    }
    for app in discover_agent_registrations()? {
        let link = Path::new(&app.skills_dir).join(skill_id);
        if link.exists() {
            let meta = fs::symlink_metadata(&link)?;
            if meta.is_dir() && !meta.file_type().is_symlink() {
                fs::remove_dir_all(link)?;
            } else {
                fs::remove_file(link)?;
            }
        }
    }
    Ok(())
}

fn default_store_index() -> Vec<SkillRemoteIndex> {
    vec![
        SkillRemoteIndex {
            skill_id: "summarizer".to_string(),
            latest_version: "0.1.0".to_string(),
            download_url: "builtin://summarizer".to_string(),
            checksum: "local-dev".to_string(),
            published_at: "2026-01-01T00:00:00Z".to_string(),
        },
        SkillRemoteIndex {
            skill_id: "translator".to_string(),
            latest_version: "0.1.0".to_string(),
            download_url: "builtin://translator".to_string(),
            checksum: "local-dev".to_string(),
            published_at: "2026-01-01T00:00:00Z".to_string(),
        },
    ]
}

pub fn refresh_store_index_cache() -> io::Result<Vec<SkillRemoteIndex>> {
    let aiman_root = aiman_root_dir()?;
    fs::create_dir_all(aiman_root)?;
    let source_path = store_remote_file()?;
    let list = if source_path.exists() {
        let payload = fs::read_to_string(source_path)?;
        serde_json::from_str::<Vec<SkillRemoteIndex>>(&payload)?
    } else {
        default_store_index()
    };
    let cache_payload = serde_json::to_string_pretty(&list)?;
    fs::write(store_cache_file()?, cache_payload)?;
    Ok(list)
}

pub fn get_store_index_cache() -> io::Result<Vec<SkillRemoteIndex>> {
    let path = store_cache_file()?;
    if !path.exists() {
        return refresh_store_index_cache();
    }
    let payload = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<Vec<SkillRemoteIndex>>(&payload)?;
    Ok(parsed)
}
