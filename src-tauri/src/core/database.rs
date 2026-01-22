use surrealdb::engine::local::Db;
use surrealdb::engine::local::SurrealKv;
use surrealdb::engine::remote::http::Http;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HashEntry {
    pub hash: String,
    pub name: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashEntryOutput {
    pub hash: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct DatabaseManager {
    db: Surreal<Db>,
}

impl DatabaseManager {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let db: Surreal<Db> = Surreal::new::<SurrealKv>(path.to_str().unwrap()).await?;
        db.use_ns("tapi").use_db("main").await?;
        Ok(Self { db })
    }

    pub async fn save_hash(&self, hash: String, name: String) -> Result<()> {
        let _: Option<HashEntry> = self.db
            .upsert(("file_hashes", &hash))
            .content(HashEntry {
                hash,
                name,
                created_at: Some(chrono::Utc::now()),
            })
            .await?;
        Ok(())
    }

    pub async fn get_name(&self, hash: &str) -> Result<Option<String>> {
        let entry: Option<HashEntry> = self.db.select(("file_hashes", hash)).await?;
        Ok(entry.map(|e| e.name))
    }

    pub async fn list_all(&self) -> Result<Vec<HashEntryOutput>> {
        let mut entries: Vec<HashEntry> = self.db.select("file_hashes").await?;
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(entries.into_iter().map(|e| HashEntryOutput {
            hash: e.hash,
            name: e.name,
            created_at: e.created_at.map(|c| c.to_rfc3339()).unwrap_or_default(),
        }).collect())
    }

    pub async fn delete_hash(&self, hash: &str) -> Result<()> {
        let _: Option<HashEntry> = self.db.delete(("file_hashes", hash)).await?;
        Ok(())
    }

    pub async fn clear_all(&self) -> Result<()> {
        let _: Vec<HashEntry> = self.db.delete("file_hashes").await?;
        Ok(())
    }

    // --- Dynamic Protocol Support (HTTP/WS) ---

    pub async fn push_to_remote(&self, url: &str, token: &str, user: &str, pass: &str) -> Result<()> {
        let entries: Vec<HashEntry> = self.db.select("file_hashes").await?;
        if entries.is_empty() { return Ok(()); }

        if url.starts_with("ws") {
            let remote_db = Surreal::new::<Ws>(url.trim_start_matches("ws://").trim_start_matches("wss://")).await?;
            self.sync_native(&remote_db, token, user, pass, entries, true).await?;
        } else {
            let remote_db = Surreal::new::<Http>(url.trim_start_matches("http://").trim_start_matches("https://")).await?;
            self.sync_native(&remote_db, token, user, pass, entries, true).await?;
        }
        Ok(())
    }

    pub async fn pull_from_remote(&self, url: &str, token: &str, user: &str, pass: &str) -> Result<()> {
        if url.starts_with("ws") {
            let remote_db = Surreal::new::<Ws>(url.trim_start_matches("ws://").trim_start_matches("wss://")).await?;
            self.sync_native(&remote_db, token, user, pass, vec![], false).await?;
        } else {
            let remote_db = Surreal::new::<Http>(url.trim_start_matches("http://").trim_start_matches("https://")).await?;
            self.sync_native(&remote_db, token, user, pass, vec![], false).await?;
        }
        Ok(())
    }

    async fn sync_native<C: surrealdb::Connection>(&self, remote_db: &Surreal<C>, token: &str, user: &str, pass: &str, local_entries: Vec<HashEntry>, is_push: bool) -> Result<()> {
        // Auth
        if !token.is_empty() {
            remote_db.authenticate(token).await?;
        } else if !user.is_empty() && !pass.is_empty() {
            remote_db.signin(Root { 
                username: user, 
                password: pass 
            }).await?;
        }

        remote_db.use_ns("tapi").use_db("main").await?;

        if is_push {
            let _ = remote_db
                .query("INSERT INTO file_hashes $entries ON DUPLICATE KEY UPDATE name = $after.name, created_at = $after.created_at")
                .bind(("entries", local_entries))
                .await?;
        } else {
            let remote_entries: Vec<HashEntry> = remote_db.select("file_hashes").await?;
            if !remote_entries.is_empty() {
                let _ = self.db
                    .query("INSERT INTO file_hashes $entries ON DUPLICATE KEY UPDATE name = $after.name, created_at = $after.created_at")
                    .bind(("entries", remote_entries))
                    .await?;
            }
        }
        Ok(())
    }
}
