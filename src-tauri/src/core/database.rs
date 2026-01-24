use surrealdb::engine::local::Db;
use surrealdb::engine::local::SurrealKv;
use surrealdb::engine::remote::http::Http;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const REMOTE_TIMEOUT_SECS: u64 = 15;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HashEntry {
    pub hash: String,
    pub name: String,
    #[serde(default)]
    pub folder: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashEntryOutput {
    pub hash: String,
    pub name: String,
    #[serde(default)]
    pub folder: String,
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

    pub async fn save_hash(&self, hash: String, name: String, folder: String) -> Result<()> {
        let _: Option<HashEntry> = self.db
            .upsert(("file_hashes", &hash))
            .content(HashEntry {
                hash,
                name,
                folder,
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
            folder: e.folder,
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

    // --- Auth Helper ---
    async fn auth_remote<C: surrealdb::Connection>(remote_db: &Surreal<C>, token: &str, user: &str, pass: &str) -> Result<()> {
        if !token.is_empty() {
            remote_db.authenticate(token).await?;
        } else if !user.is_empty() && !pass.is_empty() {
            remote_db.signin(Root { 
                username: user, 
                password: pass 
            }).await?;
        }
        remote_db.use_ns("tapi").use_db("main").await?;
        Ok(())
    }

    // --- Test Connection (No Side Effects) ---
    pub async fn test_remote_connection(url: &str, token: &str, user: &str, pass: &str) -> Result<()> {
        let timeout = Duration::from_secs(REMOTE_TIMEOUT_SECS);
        
        tokio::time::timeout(timeout, async {
            if url.starts_with("ws") {
                let remote_db = Surreal::new::<Ws>(url.trim_start_matches("ws://").trim_start_matches("wss://")).await?;
                Self::auth_remote(&remote_db, token, user, pass).await?;
            } else {
                let remote_db = Surreal::new::<Http>(url.trim_start_matches("http://").trim_start_matches("https://")).await?;
                Self::auth_remote(&remote_db, token, user, pass).await?;
            }
            Ok::<(), anyhow::Error>(())
        }).await.map_err(|_| anyhow::anyhow!("Connection timeout ({}s)", REMOTE_TIMEOUT_SECS))??;
        
        Ok(())
    }

    // --- Dynamic Protocol Support (HTTP/WS) with Timeout ---

    pub async fn push_to_remote(&self, url: &str, token: &str, user: &str, pass: &str) -> Result<()> {
        let entries: Vec<HashEntry> = self.db.select("file_hashes").await?;
        if entries.is_empty() { return Ok(()); }

        let timeout = Duration::from_secs(REMOTE_TIMEOUT_SECS);
        
        tokio::time::timeout(timeout, async {
            if url.starts_with("ws") {
                let remote_db = Surreal::new::<Ws>(url.trim_start_matches("ws://").trim_start_matches("wss://")).await?;
                self.sync_push(&remote_db, token, user, pass, entries).await?;
            } else {
                let remote_db = Surreal::new::<Http>(url.trim_start_matches("http://").trim_start_matches("https://")).await?;
                self.sync_push(&remote_db, token, user, pass, entries).await?;
            }
            Ok::<(), anyhow::Error>(())
        }).await.map_err(|_| anyhow::anyhow!("Push timeout ({}s)", REMOTE_TIMEOUT_SECS))??;
        
        Ok(())
    }

    pub async fn pull_from_remote(&self, url: &str, token: &str, user: &str, pass: &str) -> Result<()> {
        let timeout = Duration::from_secs(REMOTE_TIMEOUT_SECS);
        
        tokio::time::timeout(timeout, async {
            if url.starts_with("ws") {
                let remote_db = Surreal::new::<Ws>(url.trim_start_matches("ws://").trim_start_matches("wss://")).await?;
                self.sync_pull(&remote_db, token, user, pass).await?;
            } else {
                let remote_db = Surreal::new::<Http>(url.trim_start_matches("http://").trim_start_matches("https://")).await?;
                self.sync_pull(&remote_db, token, user, pass).await?;
            }
            Ok::<(), anyhow::Error>(())
        }).await.map_err(|_| anyhow::anyhow!("Pull timeout ({}s)", REMOTE_TIMEOUT_SECS))??;
        
        Ok(())
    }

    async fn sync_push<C: surrealdb::Connection>(&self, remote_db: &Surreal<C>, token: &str, user: &str, pass: &str, entries: Vec<HashEntry>) -> Result<()> {
        Self::auth_remote(remote_db, token, user, pass).await?;
        
        let _ = remote_db
            .query("INSERT INTO file_hashes $entries ON DUPLICATE KEY UPDATE name = $after.name, folder = $after.folder, created_at = $after.created_at")
            .bind(("entries", entries))
            .await?;
        Ok(())
    }

    async fn sync_pull<C: surrealdb::Connection>(&self, remote_db: &Surreal<C>, token: &str, user: &str, pass: &str) -> Result<()> {
        Self::auth_remote(remote_db, token, user, pass).await?;
        
        let remote_entries: Vec<HashEntry> = remote_db.select("file_hashes").await?;
        if !remote_entries.is_empty() {
            let _ = self.db
                .query("INSERT INTO file_hashes $entries ON DUPLICATE KEY UPDATE name = $after.name, folder = $after.folder, created_at = $after.created_at")
                .bind(("entries", remote_entries))
                .await?;
        }
        Ok(())
    }
}

