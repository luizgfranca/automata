use crate::module::suggestion::Suggestion;

pub trait SuggestionProvider {
    fn id(&self) -> String;

    fn init(&mut self) {}

    fn load_static_suggestions(&self) -> Vec<Suggestion> {
        vec![]
    }

    fn load_dynamic_suggestions(&self, input: Option<&str>) -> Vec<Suggestion> {
        vec![]
    }

    fn activate(&self, item: &Suggestion);

    fn complete(&self, item: &Suggestion, input: &str) -> Option<String> {
        None
    }

    fn assert_msg(&self, msg: &str) -> String {
        format!("{}: {}", self.id(), msg)
    }
}
