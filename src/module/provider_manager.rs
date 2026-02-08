use std::collections::BTreeMap;

use crate::module::{
    provider::{
        application::application::ApplicationProvider, command_execution::CommandExecutionProvider,
        encoding::EncodingProvider, explorer::explorer::ExplorerProvider,
        math::MathEvaluationProvider, search::WebSearchProvider,
        system_action::system_action::SystemActionProvider,
        unit_conversion::unit_conversion::UnitConversionProvider,
    },
    suggestion::Suggestion,
    suggestion_provider::PostActivationAction,
};

use super::suggestion_provider::SuggestionProvider;

struct RelatedSuggestionAndProvider<'a> {
    suggestion: &'a Suggestion,
    provider: &'a Box<dyn SuggestionProvider>,
}

pub struct ProviderManager {
    providers: BTreeMap<String, Box<dyn SuggestionProvider>>,

    static_suggestions: Vec<Suggestion>,
    dynamic_suggestions: Vec<Suggestion>,
}

impl<'a> ProviderManager {
    pub fn new() -> Self {
        let mut providers: BTreeMap<String, Box<dyn SuggestionProvider>> = BTreeMap::new();
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

        Self {
            providers,
            static_suggestions: vec![],
            dynamic_suggestions: vec![],
        }
    }

    pub fn init(&mut self) {
        dbg!("provideramanger::init");
        self.providers.iter_mut().for_each(|(_, it)| {
            it.init();
        });

        for (_, it) in self.providers.iter() {
            let mut suggestions = it.load_static_suggestions();
            self.static_suggestions.append(&mut suggestions);
        }
    }

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

    pub fn load_suggestions(&mut self, input: Option<&str>) -> Vec<Suggestion> {
        let mut new_dynamic_suggestions: Vec<Suggestion> = Vec::new();
        for (_, it) in self.providers.iter() {
            let mut suggestions = it.load_dynamic_suggestions(input);
            new_dynamic_suggestions.append(&mut suggestions)
        }
        self.dynamic_suggestions = new_dynamic_suggestions.clone();

        let mut result = self.get_relevant_static_suggestions(input);
        result.append(&mut new_dynamic_suggestions);
        result
    }

    fn find_referenced_suggestion(
        &self,
        provider_id: &str,
        suggestion_id: &str,
    ) -> Option<&Suggestion> {
        let suggestion_match_condition =
            |s: &&Suggestion| s.id == suggestion_id && s.provider_id == provider_id;

        self.dynamic_suggestions
            .iter()
            .find(suggestion_match_condition)
            .or_else(|| {
                self.static_suggestions
                    .iter()
                    .find(suggestion_match_condition)
            })
    }

    fn get_suggestion_and_provider_or_fail(
        &'a self,
        provider_id: &str,
        suggestion_id: &str,
    ) -> RelatedSuggestionAndProvider<'a> {
        let suggestion = self.find_referenced_suggestion(provider_id, suggestion_id)
            .expect(&format!("expected suggestion references to always be valid, but this isn't ({provider_id}, {suggestion_id})"));

        let provider = self.providers.get(provider_id).expect(&format!(
            "expected provider references to always be valid, but this isn't {provider_id}"
        ));

        RelatedSuggestionAndProvider {
            suggestion,
            provider,
        }
    }

    pub fn activate(&self, provider_id: &str, suggestion_id: &str) -> PostActivationAction {
        let RelatedSuggestionAndProvider {
            suggestion,
            provider,
        } = self.get_suggestion_and_provider_or_fail(provider_id, suggestion_id);

        provider.activate(suggestion)
    }

    pub fn complete(&self, provider_id: &str, suggestion_id: &str, input: &str) -> Option<String> {
        let RelatedSuggestionAndProvider {
            suggestion,
            provider,
        } = self.get_suggestion_and_provider_or_fail(provider_id, suggestion_id);

        provider.complete(suggestion, input)
    }
}
