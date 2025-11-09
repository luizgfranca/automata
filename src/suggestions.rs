use std::{path::Path, rc::Rc};

use derivative::Derivative;
use freedesktop_desktop_entry::DesktopEntry;
use xdg_utils::query_default_app;

use crate::{fsutil::is_dir_path, sessionmgr::{SessionMgr, SessionOperation}, sysaction, sysinfo::{DefaultApplicationType, SysInfoLoader}};

#[derive(Debug, Clone)]
pub enum Action {
    Open(DefaultApplicationType, String),
    Command(Vec<String>),
    Session(SessionOperation)
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    // TODO: maybe turn this guy into an Option since not all options will have an icon
    //      (ex: command)
    pub icon_path: String,
    pub action: Action,
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

    pub fn run(&self, suggestion: &Suggestion) {
        match &suggestion.action {
            Action::Open(app_type, target) => sysaction::try_run(
                &self.sysinfo_loader.get_open_cmd(app_type, &target)
            ),
            Action::Command(cmd) => sysaction::try_run(&cmd),
            Action::Session(op) => self.session_mgr.perform(&op),
        };
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
            items.push(Suggestion{
                title: "Suspend".to_owned(),
                description: "Suspend the computer".to_owned(),
                icon_path: String::new(),
                action: Action::Session(SessionOperation::Suspend)
            });
        }

        if session_mgr.enable_reboot {
            items.push(Suggestion{
                title: "Restart".to_owned(),
                description: "Restart the computer".to_owned(),
                icon_path: String::new(),
                action: Action::Session(SessionOperation::Reboot)
            });
        }

        if session_mgr.enable_poweroff {
            items.push(Suggestion{
                title: "Shutdown".to_owned(),
                description: "Poweeer off the system".to_owned(),
                icon_path: String::new(),
                action: Action::Session(SessionOperation::PoweOff)
            });
        }

        items
    }

    fn load_dynamic_items(&self, input: &str) -> Vec<Suggestion> {
        let mut s: Vec<Suggestion> = Vec::new();

        if is_dir_path(input) {
            s.push(Suggestion {
                title: format!("Open folder: '{}'", input),
                // TODO: see what should i add here
                description: String::new(),
                icon_path: String::new(),
                // FIXME: there's no way to correctly separate an argument string, event if the user
                //        uses simple/double quotes or just puts the string with spaces in there
                action: Action::Open(DefaultApplicationType::FileExplorer, input.to_string()),
            });
        }

        s.push(Suggestion {
            title: format!("Run command: '{}'", input),
            // TODO: see what should i add here
            description: String::new(),
            icon_path: String::new(),
            // FIXME: there's no way to correctly separate an argument string, event if the user
            //        uses simple/double quotes or just puts the string with spaces in there
            action: Action::Command(input.split(" ").map(|s| s.to_string()).collect()),
        });

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

        Self {
            title: name.clone(),
            description: name,        // TODO: find right field to use here
            icon_path: String::new(), // TODO: find right logic for loading the icon
            action: Action::from(&e),
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
