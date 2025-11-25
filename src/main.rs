mod component;
mod fsutil;
mod mathutils;
mod sessionmgr;
mod suggestions;
mod sysaction;
mod sysinfo;

use std::sync::{Arc, Mutex};

use component::suggestion_row::{SuggestionRow, SuggestionRowData};
use gtk4::gio::{self};
use mathutils::*;
use suggestions::SuggestionMgr;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4::gdk::Key;
use gtk4::{self as gtk, EventControllerKey, ScrolledWindow, gdk};

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

fn main() -> glib::ExitCode {
    let suggestion_mgr = Arc::new(Mutex::new(SuggestionMgr::new()));

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
            .title("Hello, World!")
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

        {
            for it in suggestion_mgr
                .lock()
                .expect("unable to lock suggestionMgr")
                .get_suggestions()
            {
                dbg!(&it);
                list_store.append(&SuggestionRowData::new(
                    &it.id,
                    &it.title,
                    &it.description,
                    it.icon_path.clone(),
                ));
            }
        }

        let suggestion_mgr_clone = suggestion_mgr.clone();
        let list_store_clone = list_store.clone();
        main_input.connect_changed(move |input| {
            dbg!("main_input.connect_changed");
            let input_str: String = input.text().into();
            let mut mgr = suggestion_mgr_clone
                .lock()
                .expect("unable to lock suggestion list");

            mgr.update(&input_str);
            list_store_clone.remove_all();

            for it in mgr.get_suggestions() {
                list_store_clone.append(&SuggestionRowData::new(
                    &it.id,
                    &it.title,
                    &it.description,
                    it.icon_path.clone(),
                ));
            }
        });

        let selection_model = gtk::SingleSelection::new(Some(list_store));
        let list_view = gtk::ListView::new(Some(selection_model.clone()), Some(factory));
        let suggestion_mgr_clone = suggestion_mgr.clone();
        list_view.connect_activate(move |list_view, position| {
            let model = list_view.model().unwrap();
            let row_data = model
                .item(position)
                .and_downcast::<SuggestionRowData>()
                .expect("selected item should always be able to downcast to the type defined for its row");
            {
                let mgr = suggestion_mgr_clone.lock().expect("SuggestionMgr poisoned");
                mgr.run_by_id(&row_data.id());
            }
        });

        let suggestion_list_scrollable = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&list_view)
            .vexpand(true)
            .build();

        let window_clone = window.clone();
        let suggestion_mgr_clone = suggestion_mgr.clone();
        let selection_model_clone = selection_model.clone();
        main_input.connect_activate(move |_| {
            dbg!("main_input.connect_activate");
            let selected = selection_model_clone.selected_item(); 
            if let None = selected {
                return;
            }

            let row_data = selected.and_downcast::<SuggestionRowData>()
                .expect("selected item should always be able to downcast to the type defined for its row");
            {
                let mgr = suggestion_mgr_clone.lock().expect("SuggestionMgr poisoned");
                mgr.run_by_id(&row_data.id());
            }

            window_clone.close();
        });

        let key_controller = EventControllerKey::new();
        let window_clone = window.clone();
        let suggestion_mgr_clone = suggestion_mgr.clone();
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
                    let suggestion = {
                        let mgr = suggestion_mgr_clone.lock().expect("SuggestionMgr poisoned");
                        mgr.try_get_suggestion_by_id(&row_data.id())
                            .expect(
                                &format!("item with ID {} when on the list_view should alwyas be present on SuggestionManager", row_data.id()
                                )
                            )
                            .clone()
                    };
                    if let Some(completion) = &suggestion.completion {
                        dbg!(&completion);
                        main_input_clone.set_text(completion);
                        main_input_clone.set_position(-1);
                    }
                    return gtk::glib::Propagation::Stop;
                }
                Key::Down => {
                    let new_position = u32_increment_wrap(
                        selection_model_clone.selected(),
                        0,
                        selection_model_clone.n_items() - 1,
                    );
                    dbg!((selection_model_clone.selected(), &new_position));
                    selection_model_clone.set_selected(new_position);
                    return gtk::glib::Propagation::Stop;
                }
                Key::Up => {
                    let new_position = u32_decrement_wrap(
                        selection_model_clone.selected(),
                        0,
                        selection_model_clone.n_items() - 1,
                    );
                    dbg!((selection_model_clone.selected(), &new_position));
                    selection_model_clone.set_selected(new_position);
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
