use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::module::{
    provider::{
        application::application::ApplicationProvider, command_execution::CommandExecutionProvider,
        encoding::EncodingProvider, explorer::explorer::ExplorerProvider, find::FSFinderProvider,
        math::MathEvaluationProvider, search::WebSearchProvider,
        system_action::system_action::SystemActionProvider,
        unit_conversion::unit_conversion::UnitConversionProvider,
    },
    suggestion::Suggestion,
    suggestion_provider::PostActivationAction,
};

use super::suggestion_provider::SuggestionProvider;

enum InputLoadingState {
    Empty,
    Loading,
    Done,
}

type Provider = Box<dyn SuggestionProvider + Send + Sync>;

pub struct ProviderManager {
    runtime: tokio::runtime::Runtime,
    unitialized_providers: Option<BTreeMap<String, Provider>>,
    providers: Option<Arc<BTreeMap<String, Provider>>>,

    static_suggestions: Vec<Suggestion>,
    dynamic_suggestions: Arc<tokio::sync::Mutex<HashMap<String, Vec<Suggestion>>>>,

    input_load_state: Arc<tokio::sync::Mutex<HashMap<String, InputLoadingState>>>,
}

impl<'a> ProviderManager {
    pub fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(8)
            .enable_all()
            .build()
            .unwrap();

        let mut providers: BTreeMap<String, Provider> = BTreeMap::new();
        providers.insert(
            ApplicationProvider::ID.to_string(),
            Box::new(ApplicationProvider::new()),
        );
        providers.insert(
            CommandExecutionProvider::ID.to_string(),
            Box::new(CommandExecutionProvider::new()),
        );
        providers.insert(
            MathEvaluationProvider::ID.to_string(),
            Box::new(MathEvaluationProvider::new()),
        );
        providers.insert(
            EncodingProvider::ID.to_string(),
            Box::new(EncodingProvider::new()),
        );
        providers.insert(
            UnitConversionProvider::ID.to_string(),
            Box::new(UnitConversionProvider::new()),
        );
        providers.insert(
            WebSearchProvider::ID.to_string(),
            Box::new(WebSearchProvider::new()),
        );
        providers.insert(
            SystemActionProvider::ID.to_string(),
            Box::new(SystemActionProvider::new()),
        );
        providers.insert(
            ExplorerProvider::ID.to_string(),
            Box::new(ExplorerProvider::new()),
        );
        providers.insert(
            FSFinderProvider::ID.to_string(),
            Box::new(FSFinderProvider::new()),
        );

        Self {
            unitialized_providers: Some(providers),
            providers: None,
            static_suggestions: vec![],
            dynamic_suggestions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            runtime: rt,
            input_load_state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn init(&mut self) {
        dbg!("provideramanger::init");
        let mut providers = self
            .unitialized_providers
            .take()
            .expect("unitialized_providers already taken");
        providers.iter_mut().for_each(|(_, it)| {
            it.init();
        });

        for (_, it) in providers.iter() {
            let mut suggestions = it.load_static_suggestions();
            self.static_suggestions.append(&mut suggestions);
        }

        self.providers = Some(Arc::new(providers));
    }

    pub fn is_initialized(&self) -> bool {
        self.unitialized_providers.is_none()
    }

    // TODO: candidate for concurrency
    fn get_relevant_static_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(input_str) => self
                .static_suggestions
                .iter()
                .filter(|suggestion| {
                    suggestion
                        .title
                        .to_uppercase()
                        .contains(input_str.to_uppercase().as_str())
                })
                .map(|x| x.clone())
                .collect(),
            None => self.static_suggestions.clone(),
        }
    }

    fn require_providers_ref(
        &self,
    ) -> Arc<BTreeMap<String, Box<dyn SuggestionProvider + Send + Sync + 'static>>> {
        self.providers
            .as_ref()
            .expect("expected providers to already have been initialized")
            .clone()
    }

    // TODO: candidate for concurrency
    fn update_dynamic_suggestions(
        &self,
        input: Option<&str>,
        new_dynamic_suggestions: Vec<Suggestion>,
    ) {
        let mut dynamic_suggestions = self.dynamic_suggestions.blocking_lock();
        let key = input.unwrap_or("").to_string();
        dynamic_suggestions.insert(key, new_dynamic_suggestions.clone());
    }

    fn try_get_async_input_load_ownership(&mut self, input_str: &str) -> bool {
        let mut load_state = self.input_load_state.blocking_lock();
        match load_state.get(input_str) {
            Some(state) => match state {
                InputLoadingState::Empty => {
                    load_state.insert(input_str.to_string(), InputLoadingState::Loading);
                    true
                }
                InputLoadingState::Loading => false,
                InputLoadingState::Done => false,
            },
            None => {
                load_state.insert(input_str.to_string(), InputLoadingState::Loading);
                true
            }
        }
    }

    pub fn load_suggestions(&mut self, input: Option<&str>) -> Vec<Suggestion> {
        let mut new_dynamic_suggestions: Vec<Suggestion> = Vec::new();

        // TODO: candidate for concurrency
        for (_, it) in self.require_providers_ref().iter() {
            let mut suggestions = it.load_dynamic_suggestions(input);
            new_dynamic_suggestions.append(&mut suggestions)
        }
        self.update_dynamic_suggestions(input, new_dynamic_suggestions.clone());

        let dynamic_suggestions_clone = self.dynamic_suggestions.clone();
        let providers_ref = self.require_providers_ref().clone();
        let input_load_state_clone = self.input_load_state.clone();
        if let Some(s) = input
            && self.try_get_async_input_load_ownership(s)
        {
            let input_str = s.to_string();
            self.runtime.spawn(async move {
                for (_, it) in providers_ref.iter() {
                    // TODO: only await after dispatching all asyncs
                    let mut new_suggestions =
                        it.load_async_dynamic_suggestions(input_str.clone()).await;
                    let mut dynamic_suggestions = dynamic_suggestions_clone.lock().await;
                    if let Some(input_suggestions) = dynamic_suggestions.get_mut(&input_str) {
                        input_suggestions.append(&mut new_suggestions);
                    };
                }

                let mut load_state = input_load_state_clone.lock().await;
                load_state.insert(input_str.clone(), InputLoadingState::Done);
            });
        }

        for (_, it) in self.require_providers_ref().iter() {
            let mut suggestions = it.load_dynamic_suggestions(input);
            new_dynamic_suggestions.append(&mut suggestions)
        }

        let mut result = self.get_relevant_static_suggestions(input);
        result.append(&mut new_dynamic_suggestions);
        result
    }

    pub fn get_updated_suggestions_result(&self, input: Option<&str>) -> Vec<Suggestion> {
        let mut result = self.get_relevant_static_suggestions(input);
        if let Some(s) = input {
            let dynamic_suggestions = self.dynamic_suggestions.blocking_lock();
            let maybe_dynamic_items = dynamic_suggestions.get(s);
            
            if let Some(dynamic_items) = maybe_dynamic_items {
                let mut other = dynamic_items.clone();
                result.append(&mut other);
            }

            return result;
        }

        vec![]
    }

    fn is_suggestion_match(&self, s: &Suggestion, provider_id: &str, suggestion_id: &str) -> bool {
        s.id == suggestion_id && s.provider_id == provider_id
    }

    fn try_get_static_suggestion(
        &self,
        provider_id: &str,
        suggestion_id: &str,
    ) -> Option<Suggestion> {
        match self
            .static_suggestions
            .iter()
            .find(|s| self.is_suggestion_match(s, provider_id, suggestion_id))
        {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    fn find_referenced_suggestion(
        &self,
        input: &str,
        provider_id: &str,
        suggestion_id: &str,
    ) -> Option<Suggestion> {
        let dynamic_suggestions = self.dynamic_suggestions.blocking_lock();
        match dynamic_suggestions.get(input) {
            Some(suggestions) => match suggestions
                .iter()
                .find(|s| self.is_suggestion_match(s, provider_id, suggestion_id))
            {
                Some(s) => Some(s.clone()),
                None => self.try_get_static_suggestion(provider_id, suggestion_id),
            },
            None => self.try_get_static_suggestion(provider_id, suggestion_id),
        }
    }

    pub fn activate(
        &self,
        input: &str,
        provider_id: &str,
        suggestion_id: &str,
    ) -> PostActivationAction {
        let suggestion = self.find_referenced_suggestion(input, provider_id, suggestion_id)
            .expect(&format!("expected suggestion references to always be valid, but this isn't ({provider_id}, {suggestion_id})"));

        let providers = self.require_providers_ref();

        let provider = providers.get(provider_id).expect(&format!(
            "expected provider references to always be valid, but this isn't {provider_id}"
        ));

        provider.activate(&suggestion)
    }

    pub fn complete(&self, provider_id: &str, suggestion_id: &str, input: &str) -> Option<String> {
        // TODO: look into a way to avoid this duplicated code between activate and complete
        let suggestion = self.find_referenced_suggestion(input, provider_id, suggestion_id)
            .expect(&format!("expected suggestion references to always be valid, but this isn't ({provider_id}, {suggestion_id})"));

        let providers = self.require_providers_ref();

        let provider = providers.get(provider_id).expect(&format!(
            "expected provider references to always be valid, but this isn't {provider_id}"
        ));

        provider.complete(&suggestion, input)
    }
}
