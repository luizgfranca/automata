use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Suggestion {
    pub provider_id: String,
    pub id: String,
    pub title: String,
    pub description: Option<String>, 
    pub icon_path: Option<String>,
    pub attributes: HashMap<String, String>
}
