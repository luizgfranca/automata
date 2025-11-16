mod suggestions;
mod sysinfo;
mod sysaction;
mod sessionmgr;
mod fsutil;
mod component;

use std::sync::{Arc, Mutex};

use component::suggestion_row::{SuggestionRow, SuggestionRowData};
use gtk4::gio::{self, AppInfo};
use suggestions::SuggestionMgr;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4::gdk::Key;
use gtk4::{self as gtk, gdk, EventControllerKey, ScrolledWindow};

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
                .default_width(1100)
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

        let suggestion_list_ui = gtk::ListBox::new();
        suggestion_list_ui.set_selection_mode(gtk::SelectionMode::Single);
        {
            for it in suggestion_mgr
                .lock()
                .expect("unable to lock suggestionMgr")
                .get_suggestions()
            {
                dbg!(&it);
                // let label = gtk::Label::new(Some(&it.title));
                // suggestion_list_ui.append(&label);
                list_store.append(&SuggestionRowData::new(
                    &it.id, 
                    &it.title,
                    &it.description,
                    it.icon_path.clone()
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
                    it.icon_path.clone()
                ));
            }
        });

        let selection_model = gtk::SingleSelection::new(Some(list_store));
        let list_view = gtk::ListView::new(Some(selection_model), Some(factory));
        let suggestion_mgr_clone = suggestion_mgr.clone();
        list_view.connect_activate(move |list_view, position| {
            let model = list_view.model().unwrap();
            let row_data = model.item(position).and_downcast::<SuggestionRowData>().unwrap();
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

        let suggestion_list_ui_clone = suggestion_list_ui.clone();
        let window_clone = window.clone();
        let suggestion_mgr_clone = suggestion_mgr.clone();
        main_input.connect_activate(move |_| {
            dbg!("main_input.connect_activate");
            let idx: usize = match suggestion_list_ui_clone.selected_row() {
                Some(current) => current.index().try_into().unwrap(),
                None => 0,
            };

            let mgr = suggestion_mgr_clone
                .lock()
                .expect("unable to get suggestion list lock");

            let selected = mgr.get_suggestions()
                .get(idx)
                .expect("suggestion selected index not found on list");

            mgr.run(&selected);
            window_clone.close();
        });



        let window_clone = window.clone();
        let suggestion_mgr_clone = suggestion_mgr.clone();
        suggestion_list_ui.connect_row_activated(move |_, row| {
            dbg!("suggestion_list_ui.connect_row_activated");
            let idx: usize = row.index().try_into().unwrap();

            let mgr = suggestion_mgr_clone
                .lock()
                .expect("unable to get suggestion list lock");

            let selected = mgr.get_suggestions()
                .get(idx)
                .expect("suggestion selected index not found on list");

            mgr.run(&selected);
            window_clone.close();
        });

        let key_controller = EventControllerKey::new();
        let window_clone = window.clone();
        let suggestion_list_ui_clone = suggestion_list_ui.clone();
        let suggestion_mgr_clone = suggestion_mgr.clone();
        let main_input_clone = main_input.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            dbg!("key_controller.connect_key_pressed");
            dbg!(&key);
            match key {
                Key::Escape => window_clone.close(),
                Key::Tab => {
                    let idx: usize = match suggestion_list_ui_clone.selected_row() {
                        Some(current) => current.index().try_into().unwrap(),
                        None => 0
                    };

                    // this is done to avoid a deadlock when on_changed on the main 
                    // input is triggered because both lock the suggestionMgr
                    // TODO: i reeeeally should organize my ownership structure to
                    //       manage this lock better
                    let selected = {
                        let mgr = suggestion_mgr_clone
                            .lock()
                            .expect("unable to get suggestion list lock");

                        mgr.get_suggestions()
                            .get(idx)
                            .expect("suggestion selected index not found on list")
                            .clone()
                    };
                   
                    if let Some(completion) = &selected.completion {
                        dbg!(&completion);
                        main_input_clone.set_text(completion);
                        main_input_clone.set_position(-1);
                    }
                    return gtk::glib::Propagation::Stop;
                }
                Key::Return => {
                    let idx: usize = match suggestion_list_ui_clone.selected_row() {
                        Some(current) => current.index().try_into().unwrap(),
                        None => 0
                    };

                    let mgr = suggestion_mgr_clone
                        .lock()
                        .expect("unable to get suggestion list lock");

                    let selected = mgr.get_suggestions()
                        .get(idx)
                        .expect("suggestion selected index not found on list");

                    mgr.run(&selected);
                    window_clone.close();
                    return gtk::glib::Propagation::Stop;
                }
                Key::Down => {
                    if let Some(current) = suggestion_list_ui_clone.selected_row() {
                        let next_index = current.index() + 1;
                        if let Some(next_row) = suggestion_list_ui_clone.row_at_index(next_index) {
                            suggestion_list_ui_clone.select_row(Some(&next_row));
                        }
                    } else {
                        if let Some(first) = suggestion_list_ui_clone.row_at_index(0) {
                            suggestion_list_ui_clone.select_row(Some(&first));
                        }
                    }
                }
                Key::Up => {
                    if let Some(current) = suggestion_list_ui_clone.selected_row() {
                        let prev_index = current.index() - 1;
                        if prev_index >= 0 {
                            if let Some(prev_row) = suggestion_list_ui_clone.row_at_index(prev_index) {
                                suggestion_list_ui_clone.select_row(Some(&prev_row));
                            }
                        }
                    }
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
