use std::collections::HashMap;

use crate::{module::{suggestion::Suggestion, suggestion_provider::{PostActivationAction, SuggestionProvider}}, system};

static CMDLINE_KEY: &str = "cmd";

pub struct CommandExecutionProvider {}

impl CommandExecutionProvider {
    pub const ID: &str = "xsystem.cmd";

    pub fn new() -> Self {
        Self {}
    }
}

impl SuggestionProvider for CommandExecutionProvider {
    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        match input {
            Some(input_str) => {
                if input_str.is_empty() {
                    return vec![]
                }

                let mut attributes: HashMap<String, String> = HashMap::new();
                attributes.insert(CMDLINE_KEY.to_string(), input_str.to_string());

                std::vec![Suggestion {
                    provider_id: String::from(CommandExecutionProvider::ID),
                    id: String::from("run"),
                    title: format!("RUN: {input_str}"),
                    description: None,
                    icon_path: None,
                    attributes
                }]
            }
            None => vec![],
        }
    }

    fn activate(&self, item: &Suggestion) -> PostActivationAction{
        let cmd = self.load_required_field(item, CMDLINE_KEY);

        // FIXME: there's no way to correctly separate an argument string if the user
        //        uses simple/double quotes or just puts the string with spaces in there
        let cmdparts: Vec<String> = cmd.split(" ").map(|s| s.to_string()).collect();
        system::cmd::try_run(&cmdparts);
        PostActivationAction::Close
    }

    fn id(&self) -> String {
        String::from(CommandExecutionProvider::ID)
    }
}
