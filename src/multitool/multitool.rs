use std::
    sync::{Arc, Mutex}
;

use crate::{
    component::suggestion_row::SuggestionRowData,
    module::{provider_manager::ProviderManager, suggestion_provider::PostActivationAction},
};

pub struct MultitoolApplication {
    // UNWRAP: since don't use poisoning for the lock we can directly unwrap it
    provider_mgr: Arc<Mutex<ProviderManager>>,
}

impl MultitoolApplication {
    pub fn new() -> Self {
        Self {
            provider_mgr: Arc::new(Mutex::new(ProviderManager::new())),
        }
    }

    pub fn initialize(&self) {
        self.provider_mgr.lock().unwrap().init();
    }

    pub fn is_initialized(&self) -> bool {
        self.provider_mgr.lock().unwrap().is_initialized()
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

    pub fn get_relevant_resolved_suggestion_rows(&self, input: Option<&str>) -> Vec<SuggestionRowData> {
        self.provider_mgr
            .lock()
            .unwrap()
            .get_updated_suggestions_result(input)
            .iter()
            .map(|suggestion| SuggestionRowData::from(&suggestion))
            .collect()
    }

    pub fn activate(&self, input: &str, provider_id: &str, suggestion_id: &str) -> PostActivationAction {
        self.provider_mgr
            .lock()
            .unwrap()
            .activate(input, provider_id, suggestion_id)
    }

    pub fn try_get_completion(
        &self,
        provider_id: &str,
        suggestion_id: &str,
        input: &str,
    ) -> Option<String> {
        self.provider_mgr
            .lock()
            .unwrap()
            .complete(provider_id, suggestion_id, input)
    }
}
