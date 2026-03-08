use std::path::Path;
use crate::system;

pub fn unravel_path_string(s: &str) -> String {
    let home_path = system::desktop::get_home_path();

    let starts_with_home_path_subst = s.chars().nth(0).map_or(false, |c| c == '~');
    let path = if starts_with_home_path_subst {
        s.replace("~", &home_path)
    } else {
        s.to_string()
    };

    path
}

pub fn try_get_context_folder(path: &Path) -> Option<&Path> {
    if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    }
}
