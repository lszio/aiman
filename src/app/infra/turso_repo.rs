use std::env;
use std::fs;
use std::io;
use std::path::Path;

use crate::app::domain::skill::SkillInstallRecord;
use crate::app::domain::skill::SkillInstallState;
use libsql::params;
use libsql::Builder;
use libsql::Connection;

const TURSO_MOCK_FILE: &str = "data/turso_sync_records.json";
const TURSO_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS skill_install_records (
    user_id TEXT NOT NULL,
    skill_id TEXT NOT NULL,
    installed_version TEXT NOT NULL,
    install_state TEXT NOT NULL,
    sync_version INTEGER NOT NULL,
    last_synced_at TEXT NOT NULL,
    PRIMARY KEY (user_id, skill_id)
)
"#;

pub async fn push_install_record(record: SkillInstallRecord) -> io::Result<SkillInstallRecord> {
    if let Ok(conn) = turso_connection().await {
        let _ = init_schema(&conn).await;
        let _ = conn
            .execute(
                "INSERT INTO skill_install_records (user_id, skill_id, installed_version, install_state, sync_version, last_synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(user_id, skill_id) DO UPDATE SET
                    installed_version=excluded.installed_version,
                    install_state=excluded.install_state,
                    sync_version=excluded.sync_version,
                    last_synced_at=excluded.last_synced_at",
                params![
                    record.user_id.clone(),
                    record.skill_id.clone(),
                    record.installed_version.clone(),
                    encode_state(&record.install_state),
                    record.sync_version,
                    record.last_synced_at.clone()
                ],
            )
            .await;
        return Ok(record);
    }

    let mut records = list_install_records().await.unwrap_or_default();
    records.retain(|item| !(item.user_id == record.user_id && item.skill_id == record.skill_id));
    records.push(record.clone());
    save_records(&records)?;
    Ok(record)
}

pub async fn list_install_records() -> io::Result<Vec<SkillInstallRecord>> {
    if let Ok(conn) = turso_connection().await {
        let _ = init_schema(&conn).await;
        let mut rows = conn
            .query(
                "SELECT user_id, skill_id, installed_version, install_state, sync_version, last_synced_at
                 FROM skill_install_records
                 ORDER BY last_synced_at DESC",
                params![],
            )
            .await
            .map_err(to_io_error)?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().await.map_err(to_io_error)? {
            out.push(SkillInstallRecord {
                user_id: row.get(0).map_err(to_io_error)?,
                skill_id: row.get(1).map_err(to_io_error)?,
                installed_version: row.get(2).map_err(to_io_error)?,
                install_state: decode_state(row.get::<String>(3).map_err(to_io_error)?),
                sync_version: row.get(4).map_err(to_io_error)?,
                last_synced_at: row.get(5).map_err(to_io_error)?,
            });
        }
        return Ok(out);
    }

    let path = Path::new(TURSO_MOCK_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let payload = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<Vec<SkillInstallRecord>>(&payload)?;
    Ok(parsed)
}

pub async fn sync_backend_status() -> String {
    if turso_connection().await.is_ok() {
        "remote_turso".to_string()
    } else {
        "fallback_local_file".to_string()
    }
}

fn save_records(records: &[SkillInstallRecord]) -> io::Result<()> {
    if let Some(parent) = Path::new(TURSO_MOCK_FILE).parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(records)?;
    fs::write(TURSO_MOCK_FILE, payload)?;
    Ok(())
}

async fn turso_connection() -> io::Result<Connection> {
    let url = env::var("TURSO_DATABASE_URL")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "TURSO_DATABASE_URL is not set"))?;
    let token = env::var("TURSO_AUTH_TOKEN")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "TURSO_AUTH_TOKEN is not set"))?;
    let db = Builder::new_remote(url, token).build().await.map_err(to_io_error)?;
    db.connect().map_err(to_io_error)
}

async fn init_schema(conn: &Connection) -> io::Result<()> {
    conn.execute(TURSO_TABLE_SQL, params![])
        .await
        .map_err(to_io_error)?;
    Ok(())
}

fn to_io_error<E: std::fmt::Display>(err: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

fn encode_state(state: &SkillInstallState) -> String {
    match state {
        SkillInstallState::NotInstalled => "NotInstalled".to_string(),
        SkillInstallState::Installing => "Installing".to_string(),
        SkillInstallState::Installed => "Installed".to_string(),
        SkillInstallState::Updating => "Updating".to_string(),
        SkillInstallState::Error(msg) => format!("Error:{msg}"),
    }
}

fn decode_state(raw: String) -> SkillInstallState {
    match raw.as_str() {
        "NotInstalled" => SkillInstallState::NotInstalled,
        "Installing" => SkillInstallState::Installing,
        "Installed" => SkillInstallState::Installed,
        "Updating" => SkillInstallState::Updating,
        _ if raw.starts_with("Error:") => SkillInstallState::Error(raw.replacen("Error:", "", 1)),
        _ => SkillInstallState::Error(raw),
    }
}
