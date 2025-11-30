use gtk4::{glib, subclass::prelude::*};

#[derive(Debug, Default, gtk4::CompositeTemplate)]
#[template(file = "template.ui")]
pub struct SuggestionRow {
    #[template_child]
    pub name: TemplateChild<gtk4::Label>,
    #[template_child]
    pub description: TemplateChild<gtk4::Label>,
    #[template_child]
    pub image: TemplateChild<gtk4::Image>,
}

#[glib::object_subclass]
impl ObjectSubclass for SuggestionRow {
    const NAME: &'static str = "SuggestionRow";
    type Type = super::SuggestionRow;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for SuggestionRow {}
impl WidgetImpl for SuggestionRow {}
impl BoxImpl for SuggestionRow {}



#[derive(Default)]
pub struct SuggestionRowData {
    pub id: std::cell::RefCell<String>,
    pub title: std::cell::RefCell<String>,
    pub description: std::cell::RefCell<String>,
    pub icon: std::cell::RefCell<Option<String>>
}

#[glib::object_subclass]
impl ObjectSubclass for SuggestionRowData {
    const NAME: &'static str = "MyData";
    type Type = super::SuggestionRowData;
    type ParentType = glib::Object;
}

impl ObjectImpl for SuggestionRowData {}
