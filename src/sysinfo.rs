use freedesktop_desktop_entry::{DesktopEntry, Iter, default_paths, get_languages_from_env};
use xdg_utils::{query_default_app, query_mime_info};

const DIRECTORY_MIMETYPE: &str = "inode/directory";
const BROWSER_MIMETYPE: &str = "text/html";

const DEFAULT_SEARCH_URL: &str = "";

#[derive(Debug, Clone)]
pub enum DefaultApplicationType {
    FileExplorer,
    Browser,
    Mime(String),
}

#[derive(Debug)]
pub struct SysInfoLoader {
    pub locales: Vec<String>,
    pub desktop_entries: Vec<DesktopEntry>,
}

pub enum FileOpenError {
    DefaultAppNotFound,
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

    pub fn get_default_app_cmd(&self, app_type: &DefaultApplicationType) -> String {
        match app_type {
            DefaultApplicationType::FileExplorer => query_default_app(DIRECTORY_MIMETYPE)
                .expect("TODO: handle when user does not have a default app to open folders"),
            DefaultApplicationType::Browser => query_default_app(BROWSER_MIMETYPE)
                .expect("TODO: handle when user does not have a default app to open web pages"),
            DefaultApplicationType::Mime(s) => query_default_app(s)
                .expect("TODO: handle when user does not have a default app to open folders"),
        }
    }

    pub fn get_open_cmd(&self, app_type: &DefaultApplicationType, path: &str) -> Vec<String> {
        let mut app_cmd = SysInfoLoader::cmd_str(&self.get_default_app_cmd(app_type));
        app_cmd.push(path.to_string());
        app_cmd
    }

    pub fn try_get_file_mime_type_str(path: &str) -> Option<DefaultApplicationType> {
        match query_mime_info(path) {
            Ok(mime) => Some(DefaultApplicationType::Mime(
                String::from_utf8(mime).expect("expected mimetype identifier string to be UTF-8"),
            )),
            Err(_) => None,
        }
    }

    pub fn cmd_str(s: &str) -> Vec<String> {
        s.split(" ")
            .filter(|it| (!it.contains('%') && !it.contains('@')))
            .map(|it| it.to_string())
            .collect()
    }
    pub fn cmd(e: &DesktopEntry) -> Vec<String> {
        e.parse_exec()
            // BUG: workaround for malformed entries, currently will generate entries that do
            // nothing, which is not good. Refactor this to handle it better (possibly ignore whole
            // entry from suggestion list when this is wrong)
            .unwrap_or(vec![])
            .iter()
            .filter(|it| (!it.contains('%') && !it.contains('@')))
            .map(|it| it.clone())
            .collect()
    }

    fn get_mimetype_default_app(&self, mime: &str) -> &DesktopEntry {
        let app = query_default_app(mime)
            .expect("TODO: handle when user does not have a default app to open folders");

        let entry = self
            .desktop_entries
            .iter()
            .find(|e| e.id() == app)
            .expect(&format!(
                "unexpected: {app}, for {mime}, does not match any desktop entry"
            ));

        entry
    }
}
