use std::collections::HashMap;

use crate::module::suggestion_provider::PostActivationAction;
use crate::module::{suggestion_provider::SuggestionProvider};
use crate::module::suggestion::Suggestion;
use crate::system;

static RESULT_EXP_KEY: &str = "result";

pub struct MathEvaluationProvider {}

impl MathEvaluationProvider {
    pub const ID: &str = "system.math";

    pub fn new() -> Self {
        Self {}
    }
}

fn attrs(value: f64) -> HashMap<String, String>{
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert(RESULT_EXP_KEY.to_string(), value.to_string());
    result
}

impl SuggestionProvider for MathEvaluationProvider {
    fn id(&self) -> String {
        MathEvaluationProvider::ID.to_string()
    }

    fn activate(&self, item: &Suggestion) -> PostActivationAction{
        let result = self.load_required_field(item, RESULT_EXP_KEY);
        system::clipboard::set_clipboard(&result);
        PostActivationAction::Close
    }

    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(exp) => {
                match meval::eval_str(exp) {
                    Ok(result) => {
                        vec![Suggestion {
                            id: "evaluation.calc".to_owned(),
                            provider_id: self.id(),
                            title: format!("RESULT: {}", result),
                            description: None,
                            icon_path: None,
                            attributes: attrs(result)
                        }]
                    },
                    Err(_) => vec![],
                }  
            },
            None => vec![],
        }
    }

    fn complete(&self, item: &Suggestion, _: &str) -> Option<String> {
        let result = self.load_required_field(item, RESULT_EXP_KEY);
        Some(result)
    }
}
