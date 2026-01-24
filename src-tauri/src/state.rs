use crate::config::{language::Language, profile::Profile};
use crate::core::database::DatabaseManager;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    #[allow(dead_code)]
    pub language: RwLock<Language>,
    pub profile: Arc<RwLock<Profile>>,
    pub db: Arc<RwLock<Option<DatabaseManager>>>,
}

impl AppState {
    pub fn new() -> Self {
        // Default init, actual loading depends on the platform and setup hook
        Self {
            language: RwLock::new(Language::default()),
            profile: Arc::new(RwLock::new(Profile::default())),
            db: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
