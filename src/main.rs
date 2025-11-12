mod suggestions;
mod sysinfo;
mod sysaction;
mod sessionmgr;
mod fsutil;

use std::process::Command;
use std::sync::{Arc, Mutex};

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

    // BUG: only allow one instance
    let app = Application::builder()
        .application_id("com.github.luizgfc.automata")
        .build();

    app.connect_activate(move |app| {
        let window = Arc::new(
            ApplicationWindow::builder()
                .application(app)
                .default_width(1100)
                .default_height(600)
                .title("Hello, World!")
                .decorated(false)
                .build(),
        );

        load_css();

        let main_input = gtk::Entry::new();
        main_input.add_css_class("main-input");
        let suggestion_list_ui = gtk::ListBox::new();
        suggestion_list_ui.set_selection_mode(gtk::SelectionMode::Single);
        {
            for it in suggestion_mgr
                .lock()
                .expect("unable to lock suggestionMgr")
                .get_suggestions()
            {
                let label = gtk::Label::new(Some(&it.title));
                suggestion_list_ui.append(&label);
            }
        }

        let suggestion_list_scrollable = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&suggestion_list_ui)
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


        let suggestion_list_ui_clone = suggestion_list_ui.clone();
        let suggestion_mgr_clone = suggestion_mgr.clone();
        main_input.connect_changed(move |input| {
            dbg!("main_input.connect_changed");
            let input_str: String = input.text().into();
            let mut mgr = suggestion_mgr_clone
                .lock()
                .expect("unable to lock suggestion list");

            mgr.update(&input_str);

            // TODO: find a cleaner way to empty the UI list
            while let Some(it) = suggestion_list_ui_clone.first_child() {
                suggestion_list_ui_clone.remove(&it);
            }

            for it in mgr.get_suggestions() {
                let label = gtk::Label::new(Some(&it.title));
                suggestion_list_ui_clone.append(&label);
            }

            if let Some(first) = suggestion_list_ui_clone.row_at_index(0) {
                suggestion_list_ui_clone.select_row(Some(&first));
            }
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
