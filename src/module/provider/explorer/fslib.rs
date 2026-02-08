use std::{env, path::Path};

pub fn unravel_path_string(s: &str) -> String {
    let home_path = env::var("HOME").expect("expected $HOME to always be defined");

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
