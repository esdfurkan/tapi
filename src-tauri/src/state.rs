use crate::config::{language::Language, profile::Profile};
use crate::core::database::DatabaseManager;

pub struct AppState {
    #[allow(dead_code)]
    pub language: Mutex<Language>,
    pub profile: Arc<Mutex<Profile>>,
    pub db: Mutex<Option<DatabaseManager>>,
}

impl AppState {
    pub fn new() -> Self {
        // Default init, actual loading depends on the platform and setup hook
        Self {
            language: Mutex::new(Language::default()),
            profile: Arc::new(Mutex::new(Profile::default())),
            db: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
