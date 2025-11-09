use std::path::Path;

pub fn is_dir_path(path_str: &str) -> bool {
    let p = Path::new(path_str);
    p.is_dir()
}
