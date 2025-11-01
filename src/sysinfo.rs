use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};

#[derive(Debug)]
pub struct SysInfoLoader {
    pub locales: Vec<String>,
    pub entries: Vec<DesktopEntry>,
}

impl SysInfoLoader {
    pub fn new() -> Self {
        let locales = get_languages_from_env();
        let entries = Iter::new(default_paths())
                .entries(Some(&locales))
                .collect::<Vec<_>>();
        Self {
            locales,
            entries,
        }
    }
}
