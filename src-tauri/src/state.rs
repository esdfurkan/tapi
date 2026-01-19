use std::sync::{Mutex, Arc};
use crate::config::{language::Language, profile::Profile};

pub struct AppState {
    #[allow(dead_code)]
    pub language: Mutex<Language>,
    pub profile: Arc<Mutex<Profile>>,
}

impl AppState {
    pub fn new() -> Self {
        // Default init, actual loading depends on the platform and setup hook
        Self {
            language: Mutex::new(Language::default()),
            profile: Arc::new(Mutex::new(Profile::default())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
