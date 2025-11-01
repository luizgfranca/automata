use std::env::{self, VarError};
use std::fs::DirEntry;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use freedesktop_desktop_entry::{
    DesktopEntry, Iter, PathSource, default_paths, get_languages_from_env,
};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4::gdk::Key;
use gtk4::{self as gtk, EventControllerKey, gdk};

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

fn try_run(cmd: &Vec<String>) {
    if let Some(app) = cmd.get(0) {
        let mut command = Command::new(app);
        let args = &cmd[1..];
        for it in args {
            command.arg(&it);
        }

        let output = command.spawn();
        match output {
            Err(e) => println!("unable to spawn process {}", e.to_string()),
            _ => (),
        }
    }
}


fn get_relevant_entries_for_input(
    entries: &Vec<DesktopEntry>,
    locales: &Vec<String>,
    input_str: &str,
) -> Vec<DesktopEntry> {
    let input_str_selector = input_str.to_uppercase();
    entries
        .iter()
        .filter(|e| {
            e.name(&locales)
                .unwrap()
                .to_uppercase()
                .contains(input_str_selector.as_str())
        })
        .map(|e| e.clone())
        .collect()
}

fn main() -> glib::ExitCode {
    let locales = Arc::new(get_languages_from_env());
    let entries = Arc::new(
        Iter::new(default_paths())
            .entries(Some(&locales))
            .collect::<Vec<_>>(),
    );
    let mut filtered_entries = Arc::new(
        Mutex::new(
            Iter::new(default_paths())
                .entries(Some(&locales))
                .collect::<Vec<_>>(),
        )
    );

    let app = Application::builder()
        .application_id("com.github.luizgfc.automata")
        .build();

    app.connect_activate(move |app| {
        let window = Arc::new(
            ApplicationWindow::builder()
                .application(app)
                .default_width(420)
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

        let suggestion_list_clone = suggestion_list_ui.clone();
        let input_clone = main_input.clone();
        let entries_clone = entries.clone();
        let l = locales.clone();
        let window_ref = window.clone();
        main_input.connect_activate(move |entry| {
            let idx: usize = match suggestion_list_clone.selected_row() {
                Some(current) => current.index().try_into().unwrap(),
                None => 0,
            };

            let input_str: String = input_clone.text().into();

            let relevant = get_relevant_entries_for_input(&entries_clone, &l, &input_str);
            let selected = relevant.get(idx);

            if let Some(selected) = selected {
                let filtered_cmd_parts: Vec<String> = selected.parse_exec().unwrap()
                    .iter()
                    .filter(|it| ( !it.contains('%') && !it.contains('@')))
                    .map(|it| it.clone())
                    .collect();
                let cmd = filtered_cmd_parts.join(" ");
                println!("{}", selected.name(&l).unwrap());
                dbg!(&cmd);
                try_run(&filtered_cmd_parts);
                window_ref.close();
            } else {
                let cmd: Vec<String> = input_str
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect();
                try_run(&cmd);
                window_ref.close();
            }
        });
        let sugg_list_copy = suggestion_list_ui.clone();
        let mut suggestion_list_scrollable = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&suggestion_list_ui)
            .vexpand(true)
            .build();

        let loc = locales.clone();
        let entr = entries.clone();
        let suggestion_list_clone = suggestion_list_ui.clone();
        let scrollable_copy = suggestion_list_scrollable.clone();
        let filtered_entr_clone = filtered_entries.clone();
        main_input.connect_changed(move |input| {
            let input_str: String = input.text().into();
            println!("input {}", &input_str);
            let relevant = get_relevant_entries_for_input(&entr, &loc, &input_str);

            while let Some(it) = suggestion_list_clone.first_child() {
                suggestion_list_clone.remove(&it);
            }
            for entry in relevant.iter() {
                let maybe_app_name = entry.name(&loc);
                if let Some(app_name) = maybe_app_name {
                    let app_name = app_name.into_owned();
                    let label = gtk::Label::new(Some(&app_name));
                    suggestion_list_clone.append(&label);
                }
            }

            let cmd_entry = gtk::Label::new(Some(&format!("Run command: '{}'", &input_str)));
            suggestion_list_clone.append(&cmd_entry);

            if let Some(first) = suggestion_list_clone.row_at_index(0) {
                suggestion_list_clone.select_row(Some(&first));
            }
            // suggestion_list_clone.queue_draw();
        });

        let entries_clone = entries.clone();
        for entry in entries_clone.iter() {
            let maybe_app_name = entry.name(&locales);
            if let Some(app_name) = maybe_app_name {
                let app_name = app_name.into_owned();
                let label = gtk::Label::new(Some(&app_name));
                suggestion_list_ui.append(&label);
            }
        }

        let l = locales.clone();
        let entries_clone = entries.clone();
        let input_clone = main_input.clone();
        let window_ref = window.clone();
        suggestion_list_ui.connect_row_activated(move |_, row| {
            // can unwrap here because the index should always be valid
            let idx: usize = row.index().try_into().unwrap();
            let input_str: String = input_clone.text().into();

            let relevant = get_relevant_entries_for_input(&entries_clone, &l, &input_str);
            let selected = relevant.get(idx);

            if let Some(selected) = selected {
                let filtered_cmd_parts: Vec<String> = selected.parse_exec().unwrap()
                    .iter()
                    .filter(|it| ( !it.contains('%') && !it.contains('@')))
                    .map(|it| it.clone())
                    .collect();
                let cmd = filtered_cmd_parts.join(" ");
                println!("{}", selected.name(&l).unwrap());
                dbg!(&cmd);
                try_run(&filtered_cmd_parts);
                window_ref.close();
            } else {
                let cmd: Vec<String> = input_str
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect();
                try_run(&cmd);
                window_ref.close();
            }
        });

        let key_controller = EventControllerKey::new();
        let window_ref = window.clone();
        let suggestion_list_clone = suggestion_list_ui.clone();
        let input_clone = main_input.clone();
        let entries_clone = entries.clone();
        let l = locales.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                Key::Escape => window_ref.close(),
                Key::Return => {
                    println!("return");
                    let idx: usize = match suggestion_list_clone.selected_row() {
                        Some(current) => current.index().try_into().unwrap(),
                        None => 0,
                    };

                    let input_str: String = input_clone.text().into();

                    let relevant = get_relevant_entries_for_input(&entries_clone, &l, &input_str);
                    let selected = relevant.get(idx);

                    if let Some(selected) = selected {
                        let filtered_cmd_parts: Vec<String> = selected.parse_exec().unwrap()
                            .iter()
                            .filter(|it| ( !it.contains('%') && !it.contains('@')))
                            .map(|it| it.clone())
                            .collect();
                        let cmd = filtered_cmd_parts.join(" ");
                        println!("{}", selected.name(&l).unwrap());
                        dbg!(&cmd);
                        try_run(&filtered_cmd_parts);
                        window_ref.close();
                    } else {
                        let cmd: Vec<String> = input_str
                            .split(" ")
                            .map(|s| s.to_string())
                            .collect();
                        try_run(&cmd);
                        window_ref.close();
                    }
                    return gtk::glib::Propagation::Stop 
                }
                Key::Down => {
                    if let Some(current) = suggestion_list_clone.selected_row() {
                        let next_index = current.index() + 1;
                        if let Some(next_row) = suggestion_list_clone.row_at_index(next_index) {
                            suggestion_list_clone.select_row(Some(&next_row));
                        }
                    } else {
                        if let Some(first) = suggestion_list_clone.row_at_index(0) {
                            suggestion_list_clone.select_row(Some(&first));
                        }
                    }
                }
                Key::Up => {
                    if let Some(current) = suggestion_list_clone.selected_row() {
                        let prev_index = current.index() - 1;
                        if prev_index >= 0 {
                            if let Some(prev_row) = suggestion_list_clone.row_at_index(prev_index) {
                                suggestion_list_clone.select_row(Some(&prev_row));
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
