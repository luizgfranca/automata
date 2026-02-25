mod imp;
use std::path::Path;

use gtk4::{gio, glib, prelude::*, subclass::prelude::*};

use crate::module::suggestion::Suggestion;

// Made based on GTK-RS github example:
// https://github.com/gtk-rs/gtk4-rs/blob/main/examples/list_view_apps_launcher/application_row/mod.rs

glib::wrapper! {
    pub struct SuggestionRow(ObjectSubclass<imp::SuggestionRow>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for SuggestionRow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SuggestionRow {
    pub fn set_data(&self, data: &SuggestionRowData) {
        let imp = self.imp();
        imp.name.set_text(&data.title());
        imp.description.set_text(&data.description());
        if let Some(path) = data.icon() {
            if Path::new(&path).exists() {
                imp.image.set_from_file(Some(&path));
            } else {
                imp.image.set_icon_name(Some(&path));
            }
        }
    }
}

glib::wrapper! {
    pub struct SuggestionRowData(ObjectSubclass<imp::SuggestionRowData>);
}

impl SuggestionRowData {
    pub fn new(id: &str, title: &str, description: &str, icon_path: Option<String>) -> Self {
        let s: Self = glib::Object::new();
        s.imp().id.replace(id.to_string());
        s.imp().title.replace(title.to_string());
        s.imp().description.replace(description.to_string());
        s.imp().icon.replace(icon_path);
        s
    }

    pub fn from(source: &Suggestion) -> Self {
        let s: Self = glib::Object::new();
        s.imp().id.replace(source.id.clone());
        s.imp().provider.replace(source.provider_id.clone());
        s.imp().title.replace(source.title.clone());
        s.imp().description.replace(
            source
                .description
                .as_ref()
                .unwrap_or(&String::new())
                .clone(),
        );
        s.imp().icon.replace(source.icon_path.clone());
        s
    }

    pub fn id(&self) -> String {
        self.imp().id.borrow().clone()
    }

    pub fn provider(&self) -> String {
        self.imp().provider.borrow().clone()
    }

    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn description(&self) -> String {
        self.imp().description.borrow().clone()
    }

    pub fn icon(&self) -> Option<String> {
        self.imp().icon.borrow().clone()
    }

}
