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

    fn load_required_field(&self, s: &Suggestion, key: &str) -> String {
        // TODO: there is a way to avoid cloning the string here, but I was too lazy
        s.attributes
            .get(key)
            .expect(
                &self.assert_msg(&format!("field {key} required but not present"))
            ).clone()
    }
}
