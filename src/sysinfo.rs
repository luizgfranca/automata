use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};

#[derive(Debug)]
pub struct SysInfoLoader {
    pub locales: Vec<String>,
    pub desktop_entries: Vec<DesktopEntry>,
}

impl SysInfoLoader {
    pub fn new() -> Self {
        let locales = get_languages_from_env();
        let desktop_entries = Iter::new(default_paths())
                .entries(Some(&locales))
                .collect::<Vec<_>>();
        Self {
            locales,
            desktop_entries,
        }
    }
}
