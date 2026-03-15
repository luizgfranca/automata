use async_trait::async_trait;
use evalexpr::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::module::suggestion::Suggestion;
use crate::module::suggestion_provider::PostActivationAction;
use crate::module::suggestion_provider::SuggestionProvider;
use crate::system;

static PATH_KEY: &str = "path";

pub struct FSFinderProvider {}

impl FSFinderProvider {
    pub const ID: &str = "system.fs.finder";

    pub fn new() -> Self {
        Self {}
    }

    fn get_finder_suggestions(&self, input: &str) -> Vec<Suggestion> {
        if !input.starts_with("find ") {
            return vec![];
        }

        let parts: Vec<&str> = input["find ".len()..].split(" in ").collect();
        let pattern = parts
            .get(0)
            .expect("should always have the first item, considering string starts with 'find '")
            .trim();
        let input_location = parts.get(1);

        if pattern.len() == 0 {
            return vec![];
        }

        let home_path = system::desktop::get_home_path();
        let location = if let Some(location) = input_location {
            location.to_owned()
        } else {
            &home_path.to_string()
        };
        if !Path::new(location).is_dir() {
            // println!("location {} is no a directory, ignoring", location);
            return vec![];
        }

        let result = find(location, pattern);
        result
            .split("\n")
            .filter_map(|line| {
                let path = Path::new(line);
                if path.is_file() {
                    Some(Suggestion {
                        id: format!("file.open({})", line),
                        provider_id: self.id(),
                        title: format!("Found file: '{}'", line),
                        description: None,
                        icon_path: None,
                        attributes: attrs(line),
                    })
                } else {
                    // dbg!("not_file");
                    // dbg!(line);
                    None
                }
            })
            .collect()
    }
}

fn attrs(path: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert(PATH_KEY.to_string(), path.to_string());
    result
}

pub fn find(root: &str, name: &str) -> String {
    let mut command = Command::new("find");
    let name_query = format!("*{}*", name);
    command.args(vec![root, "-name", &name_query]);
    dbg!(&command);

    let child = command
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    let output = child.wait_with_output().expect("failed to wait on child");

    let data = output.stdout.to_vec();

    let out = String::from_utf8(data.to_vec()).expect("unable to interpret output as string");
    out
}

#[async_trait]
impl SuggestionProvider for FSFinderProvider {
    fn id(&self) -> String {
        FSFinderProvider::ID.to_string()
    }

    fn activate(&self, item: &Suggestion) -> PostActivationAction {
        let path = &self.load_required_field(item, PATH_KEY);

        let cmd = system::desktop::get_open_cmd(
            &system::desktop::DefaultApplicationType::Mime(
                system::desktop::try_get_file_mimetype(&path)
                    .expect("expected all files to have a mimeType"),
            ),
            &path,
        );

        system::cmd::try_run(&cmd);

        PostActivationAction::Close
    }

    async fn load_async_dynamic_suggestions(&self, input: String) -> Vec<Suggestion> {
        dbg!("started load_async_dynamic_suggestions", &input);
        let suggestions = self.get_finder_suggestions(&input);
        dbg!("loaded load_async_dynamic_suggestions", &input);
        // dbg!(suggestions)
        suggestions
    }

    fn complete(&self, item: &Suggestion, _: &str) -> Option<String> {
        let result = self.load_required_field(item, PATH_KEY);
        Some(result)
    }
}
