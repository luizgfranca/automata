use freedesktop_desktop_entry::DesktopEntry;

use crate::sysinfo::SysInfoLoader;

#[derive(Debug, Clone)]
pub enum Action {
    Command(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    // TODO: maybe turn this guy into an Option since not all options will have an icon 
    //      (ex: command)
    pub icon_path: String,
    pub action: Action,
}

#[derive(Debug)]
pub struct SuggestionMgr {
    sysinfo_loader: SysInfoLoader,

    // items that don't depend on user input,
    // they are just loaded and don't change dynamically
    static_items: Vec<Suggestion>,
}

impl SuggestionMgr {
    pub fn new() -> Self {
        let sysinfo_loader = SysInfoLoader::new();
        let static_items =
            SuggestionMgr::load_static_items(&sysinfo_loader.locales, &sysinfo_loader.entries);

        Self {
            sysinfo_loader,
            static_items,
        }
    }

    fn load_static_items(
        locales: &Vec<String>,
        desktop_entries: &Vec<DesktopEntry>,
    ) -> Vec<Suggestion> {
        desktop_entries
            .iter()
            // FIXME: find a more correct way to determine if an entry should or not be shown
            .filter(|e| e.exec().is_some())
            .map(|e| Suggestion::from(e, &locales))
            .collect()
    }

    fn load_dynamic_items(&self, input: &str) -> Vec<Suggestion> {
        vec![Suggestion {
            title: format!("Run command: '{}'", input),
            // TODO: see what should i add here
            description: String::new(),
            icon_path: String::new(),
            // FIXME: there's no way to correctly separate an argument string, event if the user
            //        uses simple/double quotes or just puts the string with spaces in there
            action: Action::Command(input.split(" ").map(|s| s.to_string()).collect()),
        }]
    }

    fn filter_relevant_static_items(&self, input: &str) -> Vec<Suggestion> {
        self.static_items
            .iter()
            .filter(|it| {
                it.title
                    .to_uppercase()
                    .contains(input.to_uppercase().as_str())
            })
            .map(|it| it.clone())
            .collect()
    }

    pub fn get_relevant_items(&self, input: &str) -> Vec<Suggestion> {
        let mut relevant_items = self.filter_relevant_static_items(input);
        relevant_items.append(&mut self.load_dynamic_items(input));
        relevant_items
    }
}

impl Suggestion {
    fn from(e: &DesktopEntry, locales: &Vec<String>) -> Self {
        let name = e
            .name(locales)
            .expect("desktop entry name expected to be always present")
            .to_string();

        Self {
            title: name.clone(),
            description: name,        // TODO: find right field to use here
            icon_path: String::new(), // TODO: find right logic for loading the icon
            action: Action::from(&e),
        }
    }
}

impl Action {
    // FIXME: we are currently simply ignoring special parameters from the desktop file
    //        we should interpret them and generate valid suggestions corrently based on them
    fn from(e: &DesktopEntry) -> Self {
        let cmd_parts: Vec<String> = e
            .parse_exec()
            .expect("expected DesktopEntry to have a command if it reached suggestion creation")
            .iter()
            .filter(|it| (!it.contains('%') && !it.contains('@')))
            .map(|it| it.clone())
            .collect();
        Self::Command(cmd_parts)
    }
}
