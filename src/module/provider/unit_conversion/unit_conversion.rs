use crate::{
    module::{
        provider::unit_conversion::conversionlib, suggestion::Suggestion,
        suggestion_provider::SuggestionProvider,
    },
    system,
};
use regex::Regex;
use std::collections::HashMap;

static ORIGINAL_QTY_KEY: &str = "original_qty";
static ORIGINAL_UNIT_KEY: &str = "original_unit";
static CONVERTED_QTY_KEY: &str = "converted_qty";
static CONVERTED_UNIT_KEY: &str = "converted_unit";

pub struct UnitConversionProvider {}

fn attrs(src_qty: f64, src_unit: &str, dst_qty: f64, dst_unit: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert(ORIGINAL_QTY_KEY.to_string(), src_qty.to_string());
    result.insert(ORIGINAL_UNIT_KEY.to_string(), src_unit.to_string());
    result.insert(CONVERTED_QTY_KEY.to_string(), dst_qty.to_string());
    result.insert(CONVERTED_UNIT_KEY.to_string(), dst_unit.to_string());
    result
}

impl UnitConversionProvider {
    pub const ID: &str = "system.unit-conversion";

    pub fn new() -> Self {
        Self {}
    }

    fn get_unit_conversion_suggestions(&self, input: &str) -> Vec<Suggestion> {
        let parts: Vec<&str> = input.split("to").collect();
        if parts.len() < 2 {
            return vec![];
        }

        let from_arm = parts[0];
        let to_arm = parts[1];

        let amount_regex = Regex::new(r"-?\d+\.?\d*(?:[eE][+-]?\d+)?").unwrap();
        let unit_regex = Regex::new(r"[a-zA-Z]+").unwrap();

        let amount_result = amount_regex.find(from_arm);
        let unit_result = unit_regex.find(from_arm);

        match (amount_result, unit_result) {
            (Some(amount_content), Some(unit_content)) => {
                let amount: f64 = amount_content.as_str().parse().unwrap();

                match conversionlib::convert(amount, unit_content.as_str(), to_arm.trim()) {
                    Ok(result) => vec![Suggestion {
                        id: "evaluation.unit-conversion".to_owned(),
                        provider_id: self.id(),
                        title: format!(
                            "{} {} = {} {}",
                            amount,
                            unit_content.as_str(),
                            result,
                            to_arm.trim()
                        ),
                        description: None,
                        icon_path: None,
                        attributes: attrs(amount, unit_content.as_str(), result, to_arm.trim()),
                    }],
                    Err(_) => vec![],
                }
            }
            (_, _) => vec![],
        }
    }
}

impl SuggestionProvider for UnitConversionProvider {
    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(s) => self.get_unit_conversion_suggestions(s),
            None => vec![],
        }
    }

    fn activate(&self, item: &Suggestion) {}

    fn id(&self) -> String {
        String::from(UnitConversionProvider::ID)
    }
}
