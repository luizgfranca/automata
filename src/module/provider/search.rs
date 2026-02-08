use std::collections::HashMap;

use crate::module::suggestion::Suggestion;
use crate::module::suggestion_provider::SuggestionProvider;

static QUERY_KEY: &str = "query";

pub struct WebSearchProvider {}

impl WebSearchProvider {
    pub const ID: &str = "system.web.search";

    pub fn new() -> Self {
        Self {}
    }
}

fn attrs(query: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert(QUERY_KEY.to_string(), query.to_string());
    result
}

impl SuggestionProvider for WebSearchProvider {
    fn id(&self) -> String {
        WebSearchProvider::ID.to_string()
    }

    fn activate(&self, _: &Suggestion) {
        todo!()
    }

    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(s) => {
                vec![Suggestion {
                    id: format!("search.{}", s),
                    provider_id: self.id(),
                    title: format!("WEB SEARCH: {}", s),
                    description: None,
                    icon_path: None,
                    attributes: attrs(s),
                }]
            }
            None => vec![],
        }
    }
}
