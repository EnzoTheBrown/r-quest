use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::{collections::HashMap, fs, path::PathBuf};

fn db_path() -> Result<PathBuf> {
    let mut p = dirs::home_dir().context("cannot determine home directory")?;
    p.push(".config/qwest");
    fs::create_dir_all(&p)?;
    p.push("qwest.sqlite");
    Ok(p)
}

fn open_db() -> Result<Connection> {
    let conn = Connection::open(db_path()?)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        CREATE TABLE IF NOT EXISTS variables (
            name TEXT NOT NULL,
            env TEXT NOT NULL,
            value TEXT NOT NULL,
            project_name TEXT NOT NULL,
            PRIMARY KEY (project_name, env, name)
        );
        CREATE INDEX IF NOT EXISTS ix_vars_proj_env ON variables(project_name, env);
        "#,
    )?;
    Ok(conn)
}

pub fn load_vars(project: &str, env: &str) -> Result<HashMap<String, String>> {
    let conn = open_db()?;
    let mut stmt =
        conn.prepare("SELECT name, value FROM variables WHERE project_name=?1 AND env=?2")?;
    let mut out = HashMap::new();
    let rows = stmt.query_map(params![project, env], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for r in rows {
        let (k, v) = r?;
        out.insert(k, v);
    }
    Ok(out)
}

pub fn upsert_var(project: &str, env: &str, name: &str, value: &str) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        r#"
        INSERT INTO variables (project_name, env, name, value)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(project_name, env, name)
        DO UPDATE SET value=excluded.value
        "#,
        params![project, env, name, value],
    )?;
    Ok(())
}

pub fn upsert_vars(project: &str, env: &str, vars: &HashMap<String, String>) -> Result<()> {
    let mut conn = open_db()?;
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            r#"
            INSERT INTO variables (project_name, env, name, value)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(project_name, env, name)
            DO UPDATE SET value=excluded.value
            "#,
        )?;
        for (k, v) in vars {
            stmt.execute(params![project, env, k, v])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn delete_var(project: &str, env: &str, name: &str) -> Result<()> {
    let conn = open_db()?;
    conn.execute(
        "DELETE FROM variables WHERE project_name=?1 AND env=?2 AND name=?3",
        params![project, env, name],
    )?;
    Ok(())
}
