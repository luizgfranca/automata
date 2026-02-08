use freedesktop_desktop_entry::{DesktopEntry, Iter, default_paths, get_languages_from_env};
use xdg_utils::{query_default_app, query_mime_info};

use crate::system::cmd;

const DIRECTORY_MIMETYPE: &str = "inode/directory";
const BROWSER_MIMETYPE: &str = "text/html";

#[derive(Debug, Clone)]
pub enum DefaultApplicationType {
    FileExplorer,
    Browser,
    Mime(String),
}

pub fn try_get_file_mimetype(path: &str) -> Option<String> {
    match query_mime_info(path) {
        Ok(mime) => {
            Some(String::from_utf8(mime).expect("expected mimetype identifier string to be UTF-8"))
        }
        Err(_) => None,
    }
}

pub fn get_default_app_cmd(app_type: &DefaultApplicationType) -> String {
    match app_type {
        DefaultApplicationType::FileExplorer => query_default_app(DIRECTORY_MIMETYPE)
            .expect("TODO: handle when user does not have a default app to open folders"),
        DefaultApplicationType::Browser => query_default_app(BROWSER_MIMETYPE)
            .expect("TODO: handle when user does not have a default app to open web pages"),
        DefaultApplicationType::Mime(s) => query_default_app(s)
            .expect("TODO: handle when user does not have a default app to open folders"),
    }
}

pub fn get_open_cmd(app_type: &DefaultApplicationType, path: &str) -> Vec<String> {
    let mut cmd = cmd::parse_cmd_string(&get_default_app_cmd(app_type));
    cmd.push(path.to_string());
    cmd
}

fn get_mimetype_default_app(mime: &str) -> DesktopEntry {
    let app = query_default_app(mime)
        .expect("TODO: handle when user does not have a default app to open folders");

    // TODO: should create an abstraction to not need to reload all desktop entries
    //       when doing this, since they were already loaded for the application list
    let locales = get_languages_from_env();
    let desktop_entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();

    let entry = desktop_entries
        .iter()
        .find(|e| e.id() == app)
        .expect(&format!(
            "unexpected: {app}, for {mime}, does not match any desktop entry"
        ));

    entry.clone()
}
