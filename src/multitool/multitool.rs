use std::sync::Mutex;

use crate::{
    component::suggestion_row::SuggestionRowData, module::{provider_manager::ProviderManager, suggestion_provider::PostActivationAction},
};

pub struct MultitoolApplication {
    // UNWRAP: since don't use poisoning for the lock we can directly unwrap it
    provider_mgr: Mutex<ProviderManager>,
}

impl MultitoolApplication {
    pub fn new() -> Self {
        Self {
            provider_mgr: Mutex::new(ProviderManager::new()),
        }
    }

    pub fn initialize(&self) {
        self.provider_mgr.lock().unwrap().init();
    }

    pub fn get_relevant_suggestion_rows(&self, input: Option<&str>) -> Vec<SuggestionRowData> {
        self.provider_mgr
            .lock()
            .unwrap()
            .load_suggestions(input)
            .iter()
            .map(|suggestion| SuggestionRowData::from(&suggestion))
            .collect()
    }

    pub fn activate(&self, provider_id: &str, suggestion_id: &str) -> PostActivationAction {
        self.provider_mgr
            .lock()
            .unwrap()
            .activate(provider_id, suggestion_id)
    }

    pub fn try_get_completion(&self, provider_id: &str, suggestion_id: &str, input: &str) -> Option<String> {
        self.provider_mgr.lock().unwrap().complete(provider_id, suggestion_id, input)
    }
}
