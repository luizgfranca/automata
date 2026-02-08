use std::collections::HashMap;
use evalexpr::Value;

use crate::module::{suggestion_provider::SuggestionProvider};
use crate::module::suggestion::Suggestion;

static RESULT_EXP_KEY: &str = "result";

pub struct MathEvaluationProvider {}

impl MathEvaluationProvider {
    pub const ID: &str = "system.math";

    pub fn new() -> Self {
        Self {}
    }
}

fn attrs(value: Value) -> HashMap<String, String>{
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert(RESULT_EXP_KEY.to_string(), value.to_string());
    result
}

impl SuggestionProvider for MathEvaluationProvider {
    fn id(&self) -> String {
        MathEvaluationProvider::ID.to_string()
    }

    fn activate(&self, _: &Suggestion) {
        todo!()
    }

    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(exp) => {
                match evalexpr::eval(exp) {
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
}
