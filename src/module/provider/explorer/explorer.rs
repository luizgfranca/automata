use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use strum_macros::{Display, EnumIter, EnumString};

use crate::lib::vector;
use crate::module::provider::explorer::fslib;
use crate::module::suggestion::Suggestion;
use crate::module::suggestion_provider::SuggestionProvider;
use crate::system;

#[derive(Debug, EnumString, EnumIter, Display)]
enum EntryType {
    #[strum(serialize = "file")]
    File,
    #[strum(serialize = "folder")]
    Folder,
}

static PATH_KEY: &str = "path";
static ENTRY_TYPE_KEY: &str = "entry_type";

pub struct ExplorerProvider {}

fn attrs(entry_type: EntryType, path: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert(PATH_KEY.to_string(), path.to_string());
    result.insert(ENTRY_TYPE_KEY.to_string(), entry_type.to_string());
    result
}

impl ExplorerProvider {
    pub const ID: &str = "system.fs.explorer";

    pub fn new() -> Self {
        Self {}
    }

    fn build_suggestion_for_fs_entry(
        &self,
        path: &Path,
        absolute_path: &str,
        input: &str,
    ) -> Option<Suggestion> {
        if path.is_dir() {
            Some(Suggestion {
                id: format!("folder.open({})", input),
                provider_id: self.id(),
                title: format!("Open folder: '{}'", input),
                description: None,
                icon_path: None,
                attributes: attrs(EntryType::Folder, &absolute_path),
            })
        } else if path.is_file() {
            Some(Suggestion {
                id: format!("file.open({})", input),
                provider_id: self.id(),
                title: format!("Open file: '{}'", input),
                description: None,
                icon_path: None,
                attributes: attrs(EntryType::File, &absolute_path),
            })
        } else {
            dbg!("not_file_or_dir");
            dbg!(absolute_path);
            None
        }
    }

    fn get_fs_suggestions(&self, input: &str) -> Vec<Suggestion> {
        let mut s: Vec<Suggestion> = Vec::new();

        let absolute_path = fslib::unravel_path_string(input);
        let absolute_path_uppecase = absolute_path.to_uppercase();
        let path = Path::new(&absolute_path);

        vector::push_if_some(
            &mut s,
            self.build_suggestion_for_fs_entry(&path, &absolute_path, input),
        );

        let maybe_origin = fslib::try_get_context_folder(&path);
        if let Some(origin) = maybe_origin
            && origin.exists()
        {
            let parent_dir = fs::read_dir(origin).expect(
                "expected that when result is returned for parent path the folder is valid",
            );
            for entry in parent_dir {
                if let Ok(e) = entry {
                    let sibling_path = e.path();
                    let sibling_absolute_path = sibling_path.to_string_lossy();
                    let sibling_absolute_path_uppercase = sibling_absolute_path.to_uppercase();
                    if sibling_absolute_path_uppercase.contains(&absolute_path_uppecase)
                        && !sibling_absolute_path_uppercase.eq(&absolute_path_uppecase)
                    {
                        vector::push_if_some(
                            &mut s,
                            self.build_suggestion_for_fs_entry(
                                &sibling_path,
                                &sibling_absolute_path,
                                &sibling_absolute_path,
                            ),
                        );
                    }
                }
            }
        }

        s
    }
}

impl SuggestionProvider for ExplorerProvider {
    fn id(&self) -> String {
        ExplorerProvider::ID.to_string()
    }

    fn activate(&self, item: &Suggestion) {
        let entry_type = EntryType::from_str(&self.load_required_field(item, ENTRY_TYPE_KEY))
            .expect(&format!(
                "expected value of {ENTRY_TYPE_KEY} field to always be a member of the EntryType enum"
            ));
        let path = self.load_required_field(item, PATH_KEY);

        match entry_type {
            EntryType::File => {
                let cmd = system::desktop::get_open_cmd(
                    &system::desktop::DefaultApplicationType::Mime(
                        system::desktop::try_get_file_mimetype(&path)
                            .expect("expected all files to have a mimeType"),
                    ),
                    &path,
                );

                system::cmd::try_run(&cmd);
            }
            EntryType::Folder => {
                let cmd = system::desktop::get_open_cmd(
                    &system::desktop::DefaultApplicationType::FileExplorer,
                    &path,
                );

                system::cmd::try_run(&cmd);
            }
        }
    }

    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(s) => self.get_fs_suggestions(s),
            None => vec![],
        }
    }
}
