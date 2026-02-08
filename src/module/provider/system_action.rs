use std::collections::HashMap;

use crate::module::suggestion::Suggestion;
use crate::module::suggestion_provider::SuggestionProvider;

static SHUTDOWN_ID: &str = "shutdown";
static RESTART_ID: &str = "restart";
static SUSPEND_ID: &str = "suspend";

pub struct SystemActionProvider {}

impl SystemActionProvider {
    pub const ID: &str = "system.action";

    pub fn new() -> Self {
        Self {}
    }
}

impl SuggestionProvider for SystemActionProvider {
    fn id(&self) -> String {
        SystemActionProvider::ID.to_string()
    }

    fn activate(&self, _: &Suggestion) {
        todo!()
    }

    fn load_static_suggestions(&self) -> Vec<Suggestion> {
        vec![
            Suggestion {
                provider_id: self.id(),
                id: SUSPEND_ID.to_owned(),
                title: "Suspend".to_owned(),
                description: Some("Suspend the computer".to_owned()),
                icon_path: None,
                attributes: HashMap::new(),
            },
            Suggestion {
                provider_id: self.id(),
                id: RESTART_ID.to_owned(),
                title: "Restart".to_owned(),
                description: Some("Restart the computer".to_owned()),
                icon_path: None,
                attributes: HashMap::new(),
            },
            Suggestion {
                provider_id: self.id(),
                id: SHUTDOWN_ID.to_owned(),
                title: "Shutdown".to_owned(),
                description: Some("Poweeer off the system".to_owned()),
                icon_path: None,
                attributes: HashMap::new(),
            },
        ]
    }
}
