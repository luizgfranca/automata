use std::collections::HashMap;

use freedesktop_desktop_entry::DesktopEntry;
use crate::module::{provider::application::desktop_info::DesktopInfoLoader, suggestion::Suggestion, suggestion_provider::SuggestionProvider};

static CMDLINE_KEY: &str = "cmd";

pub struct ApplicationProvider {
    loader: DesktopInfoLoader,
}

impl ApplicationProvider {
    pub const ID: &str = "system.app";
}

impl ApplicationProvider {
    pub fn new() -> Self {
        Self { loader: DesktopInfoLoader::new() }
    }

    fn get_suggestion_from_desktop_entry(&self, entry: &DesktopEntry) -> Suggestion {
        let name = entry
            .name(&self.loader.locales)
            .expect("desktop entry name expected to be always present")
            .to_string();

        let cmd = entry.exec().expect(&self.assert_msg("expected all desktopp entries to have an exec attribute"));

        let description = match entry.comment(&self.loader.locales) {
            Some(comment) => comment.to_string(),
            None => name.clone(),
        };

        let mut attributes: HashMap<String, String> = HashMap::new();
        attributes.insert(CMDLINE_KEY.to_string(), cmd.to_string());

        Suggestion {
            provider_id: self.id(),
            id: entry.id().to_string(),
            title: name.clone(),
            description: Some(description),
            icon_path: entry.icon().map(|s| s.to_string()),
            attributes
        }
    }
}

impl SuggestionProvider for ApplicationProvider {
    fn init(&mut self) {
        self.loader.load();
    }

    fn load_static_suggestions(&self) -> Vec<Suggestion> {
        self.loader.desktop_entries
            .iter()
            .filter(|e| !e.no_display())
            .map(|e| self.get_suggestion_from_desktop_entry(e))
            .collect()
    }

    fn id(&self) -> String {
        ApplicationProvider::ID.to_string()
    }

    fn activate(&self, item: &Suggestion) {
        dbg!(item);
    }
}
