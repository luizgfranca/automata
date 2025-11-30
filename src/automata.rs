use std::sync::{Arc, Mutex, MutexGuard};

use crate::suggestions::{Suggestion, SuggestionMgr};

pub struct ApplicationController {
    pub suggestion_mgr: Mutex<SuggestionMgr>
}

impl ApplicationController {
    pub fn build() -> Arc<ApplicationController> {
        let suggestion_mgr = Mutex::new(SuggestionMgr::new()) ;
        Arc::new(ApplicationController { suggestion_mgr })
    }

    pub fn get_suggestions(&self) -> Vec<Suggestion>{
        let mgr = self.suggestion_mgr
            .lock()
            .expect("unable to get suggestion list lock");

        mgr.get_suggestions().clone()
    }

    pub fn get_nth_suggestion(&self, idx: usize) -> Suggestion {
        let mgr = self.suggestion_mgr
            .lock()
            .expect("unable to get suggestion list lock");
        
        self.get_nth_suggestion_with_lock(&mgr, idx)
    }

    pub fn run_nth_suggestion(&self, idx: usize) {
        let mgr = self.suggestion_mgr
                .lock()
                .expect("unable to get suggestion list lock");
        let suggestion = self.get_nth_suggestion(idx);
        mgr.run(&suggestion);
    }

    pub fn update_suggestions(&self, input_str: &str) {
        let mut mgr = self.suggestion_mgr
                .lock()
                .expect("unable to get suggestion list lock");

        mgr.update(input_str);
    }

    fn get_nth_suggestion_with_lock(&self, mgr: &MutexGuard<'_, SuggestionMgr>, idx: usize) -> Suggestion {
        let s = mgr.get_suggestions()
            .get(idx)
            .expect("suggestion selected index not found on list");

        s.clone()
    }
}
