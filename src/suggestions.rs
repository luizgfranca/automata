use std::{env, fs, path::Path, rc::Rc};

use base64::prelude::*;
use derivative::Derivative;
use freedesktop_desktop_entry::DesktopEntry;
use gtk4::{
    gdk::{self, prelude::DisplayExt},
    glib::base64_decode,
};
use wl_clipboard_rs::copy::{MimeType, Options, Source};

use crate::{
    conversionutil,
    sessionmgr::{SessionMgr, SessionOperation},
    sysaction,
    sysinfo::{DefaultApplicationType, SysInfoLoader},
};
use regex::Regex;

#[derive(Debug, Clone)]
pub enum Action {
    NoOp,
    Open(DefaultApplicationType, String),
    Command(Vec<String>),
    Session(SessionOperation),
    CopyToClipboard(String),
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    // TODO: maybe turn this guy into an Option since not all options will have an icon
    //      (ex: command)
    pub icon_path: Option<String>,
    pub action: Action,

    pub completion: Option<String>,
}

pub enum PostRunAction {
    Nothing,
    Close,
}

fn get_brave_search_url(query: &str) -> String {
    let mut url = String::new();
    url.push_str("search.brave.com/search?source=desktop&q=");
    url.push_str(&query.replace(" ", "+"));

    url
}

fn set_clipboard(value: &str) {
    dbg!("setting cilpboard");
    dbg!(value);

    let opts = Options::new();
    let result = opts.copy(
        Source::Bytes(value.to_string().into_bytes().into()),
        MimeType::Autodetect,
    );

    if let Err(e) = result {
        println!("unable to copy to clipboard {}", e);
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SuggestionMgr {
    sysinfo_loader: SysInfoLoader,
    session_mgr: Rc<SessionMgr>,

    // items that don't depend on user input,
    // they are just loaded and don't change dynamically
    static_items: Vec<Suggestion>,

    relevant_items: Vec<Suggestion>,
}

impl SuggestionMgr {
    pub fn new() -> Self {
        let sysinfo_loader = SysInfoLoader::new();
        let session_mgr = Rc::new(SessionMgr::new());
        let static_items = SuggestionMgr::load_static_items(
            &sysinfo_loader.locales,
            &sysinfo_loader.desktop_entries,
            session_mgr.clone(),
        );
        let relevant_items = static_items.clone();

        Self {
            sysinfo_loader,
            static_items,
            relevant_items,
            session_mgr,
        }
    }

    pub fn update(&mut self, input: &str) {
        self.relevant_items = self.get_relevant_items(input);
    }

    pub fn get_suggestions(&self) -> &Vec<Suggestion> {
        &self.relevant_items
    }

    pub fn try_get_suggestion_by_id(&self, id: &str) -> Option<&Suggestion> {
        for it in self.get_suggestions() {
            if it.id == id {
                return Some(&it);
            }
        }

        None
    }

    pub fn run(&self, suggestion: &Suggestion) -> PostRunAction {
        match &suggestion.action {
            Action::NoOp => (),
            Action::Open(app_type, target) => {
                sysaction::try_run(&self.sysinfo_loader.get_open_cmd(app_type, &target))
            }
            Action::Command(cmd) => sysaction::try_run(&cmd),
            Action::Session(op) => self.session_mgr.perform(&op),
            Action::CopyToClipboard(str) => set_clipboard(&str),
        };

        PostRunAction::Close
    }

    pub fn run_by_id(&self, id: &str) -> PostRunAction {
        dbg!("run_by_id {}", id);
        let s = self.try_get_suggestion_by_id(id).expect(&format!(
            "Expected referenced suggestionId to always be valid, id = {}",
            id
        ));

        self.run(s)
    }

    fn load_static_items(
        locales: &Vec<String>,
        desktop_entries: &Vec<DesktopEntry>,
        session_mgr: Rc<SessionMgr>,
    ) -> Vec<Suggestion> {
        let mut items: Vec<Suggestion> = desktop_entries
            .iter()
            .filter(|e| !e.no_display())
            .map(|e| Suggestion::from(e, &locales))
            .collect();

        if session_mgr.enable_suspend {
            items.push(Suggestion {
                id: "system.action.suspend".to_owned(),
                title: "Suspend".to_owned(),
                description: "Suspend the computer".to_owned(),
                icon_path: None,
                action: Action::Session(SessionOperation::Suspend),
                completion: None,
            });
        }

        if session_mgr.enable_reboot {
            items.push(Suggestion {
                id: "system.action.restart".to_owned(),
                title: "Restart".to_owned(),
                description: "Restart the computer".to_owned(),
                icon_path: None,
                action: Action::Session(SessionOperation::Reboot),
                completion: None,
            });
        }

        if session_mgr.enable_poweroff {
            items.push(Suggestion {
                id: "system.action.poweroff".to_owned(),
                title: "Shutdown".to_owned(),
                description: "Poweeer off the system".to_owned(),
                icon_path: None,
                action: Action::Session(SessionOperation::PoweOff),
                completion: None,
            });
        }

        items
    }

    fn load_dynamic_items(&self, input: &str) -> Vec<Suggestion> {
        let mut s: Vec<Suggestion> = Vec::new();

        let mut folder_suggestions = self.get_folder_suggestions(input);
        s.append(&mut folder_suggestions);

        let mut math_suggestions = self.get_math_suggestions(input);
        s.append(&mut math_suggestions);

        let mut b64_suggestions = self.get_b64_conversion_suggestions(input);
        s.append(&mut b64_suggestions);

        let mut unit_conversin_suggestions = self.get_unit_conversion_suggestions(input);
        s.append(&mut unit_conversin_suggestions);

        // FIXME: find a way to focus the browser when this is done
        s.push(Suggestion {
            id: "action.search".to_owned(),
            title: format!("Search: '{}'", input),
            // TODO: see what should i add here
            description: String::new(),
            icon_path: None,
            // FIXME: there's no way to correctly separate an argument string, event if the user
            //        uses simple/double quotes or just puts the string with spaces in there
            action: Action::Open(DefaultApplicationType::Browser, get_brave_search_url(input)),
            completion: None,
        });

        s.push(Suggestion {
            id: "system.command".to_owned(),
            title: format!("Run command: '{}'", input),
            // TODO: see what should i add here
            description: String::new(),
            icon_path: None,
            // FIXME: there's no way to correctly separate an argument string, event if the user
            //        uses simple/double quotes or just puts the string with spaces in there
            action: Action::Command(input.split(" ").map(|s| s.to_string()).collect()),
            completion: None,
        });

        s
    }

    // FIXME: adding math resolution as a normal suggestion listItem foor now
    //        there should be a better UI for it
    fn get_math_suggestions(&self, input: &str) -> Vec<Suggestion> {
        match evalexpr::eval(input) {
            Ok(result) => vec![Suggestion {
                id: "evaluation.calc".to_owned(),
                title: format!("Result: '{}'", result),
                description: String::new(),
                icon_path: None,
                action: Action::CopyToClipboard(result.to_string()),
                completion: None,
            }],
            Err(_) => vec![],
        }
    }

    // FIXME: adding math resolution as a normal suggestion listItem foor now
    //        there should be a better UI for it
    fn get_b64_conversion_suggestions(&self, input: &str) -> Vec<Suggestion> {
        match BASE64_STANDARD.decode(input) {
            Ok(result) => match String::from_utf8(result) {
                Ok(str) => vec![Suggestion {
                    id: "evaluation.b64".to_owned(),
                    title: format!("Base64 converted text: '{}'", str),
                    description: String::new(),
                    icon_path: None,
                    action: Action::CopyToClipboard(str),
                    completion: None,
                }],
                Err(_) => vec![],
            },
            Err(_) => vec![],
        }
    }

    fn get_unit_conversion_suggestions(&self, input: &str) -> Vec<Suggestion> {
        let parts: Vec<&str> = input.split("to").collect();
        if parts.len() < 2 {
            return vec![];
        }

        let from_arm = parts[0];
        let to_arm = parts[1];

        let amount_regex = Regex::new(r"-?\d+\.?\d*(?:[eE][+-]?\d+)?").unwrap();
        let unit_regex = Regex::new(r"[a-zA-Z]+").unwrap();

        let amount_result = amount_regex.find(from_arm);
        let unit_result = unit_regex.find(from_arm);

        match (amount_result, unit_result) {
            (Some(amount_content), Some(unit_content)) => {
                let amount: f64 = amount_content.as_str().parse().unwrap();

                match conversionutil::convert(amount, unit_content.as_str(), to_arm.trim()) {
                    Ok(result) => vec![Suggestion {
                        id: "evaluation.unit-conversion".to_owned(),
                        title: format!(
                            "{}{} = {}{}",
                            amount,
                            unit_content.as_str(),
                            result,
                            to_arm.trim()
                        ),
                        description: String::new(),
                        icon_path: None,
                        action: Action::NoOp,
                        completion: None,
                    }],
                    Err(_) => vec![],
                }
            }
            (_, _) => vec![],
        }
    }

    // TODO: search for direct strings on folders of the home dir
    // TODO: tab-complete selected folder suggestion
    fn get_folder_suggestions(&self, input: &str) -> Vec<Suggestion> {
        let mut s: Vec<Suggestion> = Vec::new();
        let home_path = env::var("HOME").expect("expected $HOME to always be defined");
        let starts_with_home_path_subst = input.chars().nth(0).map_or(false, |c| c == '~');
        let final_input_path = if starts_with_home_path_subst {
            input.replace("~", &home_path)
        } else {
            input.to_string()
        };

        let path = Path::new(&final_input_path);
        if path.is_dir() {
            s.push(Suggestion {
                // TODO: this approach is pretty bad
                //       find a good way to reference actions back from list model
                id: format!("system.folder.open {}", input),
                title: format!("Open folder: '{}'", input),
                // TODO: see what should i add here
                description: String::new(),
                icon_path: None,
                // FIXME: there's no way to correctly separate an argument string, event if the user
                //        uses simple/double quotes or just puts the string with spaces in there
                action: Action::Open(
                    DefaultApplicationType::FileExplorer,
                    final_input_path.to_string(),
                ),
                completion: None,
            });
        }

        let maybe_origin = if path.to_string_lossy().ends_with("/") {
            Some(path)
        } else {
            path.parent()
        };

        if let Some(origin) = maybe_origin {
            if let Ok(parent_dir) = fs::read_dir(origin) {
                for entry in parent_dir {
                    if let Ok(e) = entry {
                        let path = e.path();
                        let path_str = path.to_string_lossy();
                        let path_uppercase_str = path_str.to_uppercase();
                        let mut completion = path_str.to_string();
                        completion.push_str("/");
                        if path.is_dir()
                            && path_uppercase_str.contains(&final_input_path.to_uppercase())
                            && !path_uppercase_str.eq(&final_input_path.to_uppercase())
                        {
                            s.push(Suggestion {
                                // TODO: this approach is pretty bad
                                //       find a good way to reference actions back from list model
                                id: format!("system.folder.open {}", path_str),
                                // TODO: investigate what is the risk of using "to_string_lossy" here,
                                //       and if there's a better approach
                                title: format!("Open folder: '{}'", path_str),
                                // TODO: see what should i add here
                                description: String::new(),
                                icon_path: None,
                                // FIXME: there's no way to correctly separate an argument string, event if the user
                                //        uses simple/double quotes or just puts the string with spaces in there
                                action: Action::Open(
                                    DefaultApplicationType::FileExplorer,
                                    path.to_string_lossy().into(),
                                ),
                                completion: Some(completion),
                            });
                        }
                    }
                }
            }
        }

        s
    }

    fn filter_relevant_static_items(&self, input: &str) -> Vec<Suggestion> {
        self.static_items
            .iter()
            .filter(|it| {
                it.title
                    .to_uppercase()
                    .contains(input.to_uppercase().as_str())
            })
            .map(|it| it.clone())
            .collect()
    }

    fn get_relevant_items(&self, input: &str) -> Vec<Suggestion> {
        let mut relevant_items = self.filter_relevant_static_items(input);
        relevant_items.append(&mut self.load_dynamic_items(input));
        relevant_items
    }
}

impl Suggestion {
    fn from(e: &DesktopEntry, locales: &Vec<String>) -> Self {
        let name = e
            .name(locales)
            .expect("desktop entry name expected to be always present")
            .to_string();

        let description = match e.comment(locales) {
            Some(comment) => comment.to_string(),
            None => name.clone(),
        };

        Self {
            id: e.id().to_string(),
            title: name.clone(),
            description,
            icon_path: e.icon().map(|s| s.to_string()),
            action: Action::from(&e),
            completion: None,
        }
    }
}

impl Action {
    // FIXME: we are currently simply ignoring special parameters from the desktop file
    //        we should interpret them and generate valid suggestions corrently based on them
    fn from(e: &DesktopEntry) -> Self {
        Self::Command(SysInfoLoader::cmd(e))
    }
}
