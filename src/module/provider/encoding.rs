use evalexpr::Value;
use std::collections::HashMap;

use crate::module::suggestion::Suggestion;
use crate::module::suggestion_provider::SuggestionProvider;
use crate::system;
use base64::prelude::*;

static RESULT_KEY: &str = "result";

pub struct EncodingProvider {}

impl EncodingProvider {
    pub const ID: &str = "system.encoding";

    pub fn new() -> Self {
        Self {}
    }
}

fn attrs(value: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert(RESULT_KEY.to_string(), value.to_string());
    result
}

impl SuggestionProvider for EncodingProvider {
    fn id(&self) -> String {
        EncodingProvider::ID.to_string()
    }

    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(s) => match BASE64_STANDARD.decode(s) {
                Ok(result) => match String::from_utf8(result) {
                    Ok(str) => vec![Suggestion {
                        id: "evaluation.b64".to_owned(),
                        provider_id: self.id(),
                        title: format!("Decoded Base64: {}", &str),
                        description: None,
                        icon_path: None,
                        attributes: attrs(&str),
                    }],
                    Err(_) => vec![],
                },
                Err(_) => vec![],
            },
            None => vec![],
        }
    }

    fn activate(&self, item: &Suggestion) {
        let content = self.load_required_field(item, RESULT_KEY);
        system::clipboard::set_clipboard(&content);
    }

    fn complete(&self, item: &Suggestion, _: &str) -> Option<String> {
        let content = self.load_required_field(item, RESULT_KEY);
        Some(content)
    }
}
