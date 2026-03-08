mod component;
mod module;
mod multitool;
mod system;
mod lib;

use std::sync::Arc;
use std::time::Duration;

use component::suggestion_row::{SuggestionRow, SuggestionRowData};
use gtk4::gio::{self};

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4::gdk::Key;
use gtk4::glib::{spawn_async, spawn_future_local};
use gtk4::{self as gtk, EventControllerKey, ScrolledWindow, gdk};

use crate::lib::math;
use crate::module::suggestion_provider::PostActivationAction;
use crate::multitool::multitool::MultitoolApplication;

fn load_css() {
    let display = gdk::Display::default().expect("unable to load default display");
    let p = gtk::CssProvider::new();
    p.load_from_data(
        "
            .main-input {
                font-size: 2rem;
            }
        ",
    );

    gtk::style_context_add_provider_for_display(
        &display,
        &p,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn set_suggestion_list_store_rows(list_store: &gio::ListStore, rows: Vec<SuggestionRowData>) {
    list_store.remove_all();
    rows.iter().for_each(|row| list_store.append(row))
}

fn main() -> glib::ExitCode {
    let multitool = Arc::new(MultitoolApplication::new());

    let app = Application::builder()
        .application_id("com.github.luizgfc.automata")
        .build();

    app.connect_activate(move |app| {
        // avoid multiple instances
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }

        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(1000)
            .default_height(600)
            .title("Automata")
            .decorated(false)
            .build();

        load_css();

        let main_input = gtk::Entry::new();
        main_input.add_css_class("main-input");

        let list_store = gio::ListStore::new::<SuggestionRowData>();
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_factory, item| {
            let row = SuggestionRow::default();
            item.set_child(Some(&row));
        });

        factory.connect_bind(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let data = item.item().and_downcast::<SuggestionRowData>().unwrap();
            let child = item.child().and_downcast::<SuggestionRow>().unwrap();
            child.set_data(&data);
        });
        let selection_model = gtk::SingleSelection::new(Some(list_store.clone()));

        set_suggestion_list_store_rows(&list_store, vec![]);
        let multitool_clone = multitool.clone();
        let list_store_clone = list_store.clone();
        main_input.connect_changed(move |input| {
            dbg!("main_input.connect_changed");
            let input_str: String = input.text().into();
            set_suggestion_list_store_rows(&list_store_clone, multitool_clone.get_relevant_suggestion_rows(Some(&input_str)));
        });

        let multitool_clone = multitool.clone();
        let list_store_clone = list_store.clone();
        glib::idle_add_local(move || {
            multitool_clone.initialize();
            set_suggestion_list_store_rows(&list_store_clone, multitool_clone.get_relevant_suggestion_rows(None));
            glib::ControlFlow::Break
        });

        let multitool_clone = multitool.clone();
        let list_store_clone = list_store.clone();
        let main_input_clone = main_input.clone();
        let selection_model_clone = selection_model.clone();
        glib::timeout_add_local(Duration::from_millis(100), move || {
            dbg!("list update");
            if (!multitool_clone.is_initialized()) {
                return glib::ControlFlow::Continue
            }

            let updated_suggestions = multitool_clone.get_relevant_resolved_suggestion_rows(Some(main_input_clone.text().as_ref()));
            let curr_item_count: usize = list_store_clone.n_items().try_into().unwrap();
            if curr_item_count != updated_suggestions.len() {
                dbg!("updating list with new results");
                set_suggestion_list_store_rows(
                    &list_store_clone, 
                    updated_suggestions
                );
            }

            glib::ControlFlow::Continue
        });

        let main_input_clone = main_input.clone();
        let selection_model = gtk::SingleSelection::new(Some(list_store));
        let list_view = gtk::ListView::new(Some(selection_model.clone()), Some(factory));
        let multitool_clone = multitool.clone();
        list_view.connect_activate(move |list_view, position| {
            let model = list_view.model().unwrap();
            let row_data = model
                .item(position)
                .and_downcast::<SuggestionRowData>()
                .expect("selected item should always be able to downcast to the type defined for its row");
            {
                multitool_clone.activate(main_input_clone.text().as_str(), &row_data.provider(), &row_data.id());
            }

        });

        let suggestion_list_scrollable = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&list_view)
            .vexpand(true)
            .build();

        let selection_model_clone = selection_model.clone();
        let multitool_clone = multitool.clone();
        let window_clone = window.clone();
        let main_input_clone = main_input.clone();
        main_input.connect_activate(move |_| {
            dbg!("main_input.connect_activate");
            let selected = selection_model_clone.selected_item(); 
            if let None = selected {
                return;
            }

            let row_data = selected.and_downcast::<SuggestionRowData>()
                .expect("selected item should always be able to downcast to the type defined for its row");
            {
                let after = multitool_clone.activate(main_input_clone.text().as_str(), &row_data.provider(), &row_data.id());
                if let PostActivationAction::Close = after {
                    window_clone.close();
                }
            }
        });

        let key_controller = EventControllerKey::new();
        let window_clone = window.clone();
        let multitool_clone = multitool.clone();
        let main_input_clone = main_input.clone();
        let selection_model_clone = selection_model.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            dbg!("key_controller.connect_key_pressed");
            dbg!(&key);
            match key {
                Key::Escape => window_clone.close(),
                Key::Tab => {
                    let selected = selection_model_clone.selected_item();

                    let row_data = selected.and_downcast::<SuggestionRowData>()
                        .expect("selected item should always be able to downcast to the type defined for its row");

                    // need to do this in this way to free the lock before changing the input,
                    // which would change the suggestions and create a deadlock
                    // TODO: restructure this
                    let maybe_completion = multitool_clone.try_get_completion(&row_data.provider(), &row_data.id(), main_input_clone.text().as_str());
                    if let Some(completion) = maybe_completion {
                        dbg!(&completion);
                        main_input_clone.set_text(&completion);
                        main_input_clone.set_position(-1);
                    }
                    return gtk::glib::Propagation::Stop;
                }
                Key::Down => {
                    let new_position = math::u32_increment_wrap(
                        selection_model_clone.selected(),
                        0,
                        selection_model_clone.n_items() - 1,
                    );
                    dbg!((selection_model_clone.selected(), &new_position));
                    selection_model_clone.set_selected(new_position);
                    list_view.activate_action(
                        "list.scroll-to-item", 
                        Some(&new_position.to_variant())
                    ).expect(
                        &format!("expected to always be able to scroll to new selected item: {}", new_position) 
                    );
                    return gtk::glib::Propagation::Stop;
                }
                Key::Up => {
                    let new_position = math::u32_decrement_wrap(
                        selection_model_clone.selected(),
                        0,
                        selection_model_clone.n_items() - 1,
                    );
                    dbg!((selection_model_clone.selected(), &new_position));
                    selection_model_clone.set_selected(new_position);
                    list_view.activate_action(
                        "list.scroll-to-item", 
                        Some(&new_position.to_variant())
                    ).expect(
                        &format!("expected to always be able to scroll to new selected item: {}", new_position) 
                    );
                    return gtk::glib::Propagation::Stop;
                }
                _ => (),
            };

            gtk::glib::Propagation::Proceed
        });

        let container = gtk::Box::new(gtk::Orientation::Vertical, 2);
        container.set_hexpand(true);
        container.append(&main_input);
        container.append(&suggestion_list_scrollable);

        window.set_child(Some(&container));
        window.add_controller(key_controller);

        window.present();
    });

    app.run()
}
