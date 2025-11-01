use freedesktop_desktop_entry::DesktopEntry;

use crate::sysinfo::SysInfoLoader;

#[derive(Debug)]
pub enum Action {
    Command(Vec<String>),
}

#[derive(Debug)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
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
