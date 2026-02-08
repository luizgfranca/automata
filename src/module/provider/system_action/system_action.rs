use std::collections::HashMap;
use std::str::FromStr;

use crate::module::provider::system_action::session_manager::{SessionManager, SessionOperation};
use crate::module::suggestion::Suggestion;
use crate::module::suggestion_provider::{PostActivationAction, SuggestionProvider};

pub struct SystemActionProvider {
    session_manager: SessionManager,
}

impl SystemActionProvider {
    pub const ID: &str = "system.action";

    pub fn new() -> Self {
        Self {
            session_manager: SessionManager::new(),
        }
    }
}

impl SuggestionProvider for SystemActionProvider {
    fn id(&self) -> String {
        SystemActionProvider::ID.to_string()
    }

    fn activate(&self, item: &Suggestion) -> PostActivationAction {
        let op = SessionOperation::from_str(&item.id)
            .expect("expected SystemAction ids to always be valid members of SessionOperation");

        self.session_manager.perform(&op);
        PostActivationAction::Close
    }

    fn load_static_suggestions(&self) -> Vec<Suggestion> {
        let mut s: Vec<Suggestion> = Vec::new();

        if self.session_manager.enable_suspend {
            s.push(Suggestion {
                provider_id: self.id(),
                id: SessionOperation::Suspend.to_string(),
                title: "Suspend".to_owned(),
                description: Some("Suspend the computer".to_owned()),
                icon_path: None,
                attributes: HashMap::new(),
            });
        }

        if self.session_manager.enable_reboot {
            s.push(Suggestion {
                provider_id: self.id(),
                id: SessionOperation::Reboot.to_string(),
                title: "Restart".to_owned(),
                description: Some("Restart the computer".to_owned()),
                icon_path: None,
                attributes: HashMap::new(),
            });
        }

        if self.session_manager.enable_poweroff {
            s.push(Suggestion {
                provider_id: self.id(),
                id: SessionOperation::PoweOff.to_string(),
                title: "Shutdown".to_owned(),
                description: Some("Poweeer off the system".to_owned()),
                icon_path: None,
                attributes: HashMap::new(),
            });
        }

        s
    }
}
