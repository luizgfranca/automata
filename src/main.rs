use std::process::Command;
use std::sync::Arc;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use gtk4::gdk::Key;
use gtk4::{self as gtk, gdk, EventControllerKey};

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

fn try_executing(command: &str) {
    let output = Command::new(command).spawn();
    match output {
        Err(e) => println!("unable to spawn process {}", e.to_string()),
        _ => (),
    }
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("com.github.luizgfc.automata")
        .build();

    app.connect_activate(move |app| {
        let window = Arc::new(ApplicationWindow::builder()
            .application(app)
            .default_width(420)
            .default_height(40)
            .title("Hello, World!")
            .decorated(false)
            .build()
        );

        load_css();

        let key_controller = EventControllerKey::new();
        let window_ref = window.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == Key::Escape {
                window_ref.close();
            }

            gtk::glib::Propagation::Proceed
        });

        let main_input = gtk::Entry::new();
        main_input.add_css_class("main-input");
        main_input.connect_activate(|entry| {
            dbg!(entry.text());
            let user_input: String = entry.text().into();
            entry.set_text("");
            try_executing(user_input.as_str());
        });

        window.set_child(Some(&main_input));
        window.add_controller(key_controller);

        window.present();
    });

    app.run()
}
