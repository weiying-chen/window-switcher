use glib::clone;
use gtk::gdk::Display;
use gtk::glib;
use gtk::prelude::*;
use window_switcher::xdotool;

use window_switcher::constants::{APP_ID, WINDOW_NAME, WINDOW_WIDTH};

use gtk::{
    Application, ApplicationWindow, Box as Box_, CssProvider, Entry, Label, ListBox, Orientation,
    StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION,
};

use std::cell::RefCell;
use std::rc::Rc;

fn main() -> glib::ExitCode {
    let app = Application::new(Some(APP_ID), Default::default());

    app.connect_startup(|_| {
        let provider = CssProvider::new();

        provider.load_from_data(include_str!("style.css"));

        StyleContext::add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(build_ui);
    app.run()
}

fn populate_list_box(window_ids: &Vec<String>, list_box: &ListBox) {
    for window_id in window_ids.iter().filter(|s| !s.is_empty()) {
        let window_name = xdotool::get_window_name(window_id);

        if !window_name.is_empty() {
            let label = Label::new(Some(&window_name));
            list_box.append(&label);
        }
    }
}

fn clear_list_box(list_box: &ListBox) {
    while let Some(row) = list_box.last_child() {
        list_box.remove(&row);
    }
}

fn build_ui(app: &Application) {
    let vbox = Box_::new(Orientation::Vertical, 0);
    let entry = Entry::new();

    vbox.append(&entry);

    let list_box = ListBox::new();

    vbox.append(&list_box);

    let pattern = "\"\"";
    let window_ids = xdotool::search_windows("--name", &pattern);

    populate_list_box(&window_ids, &list_box);

    let window = ApplicationWindow::new(app);

    window.set_title(Some(WINDOW_NAME));
    window.set_child(Some(&vbox));
    window.set_decorated(false);
    window.set_default_size(WINDOW_WIDTH, -1);

    window.connect_show(clone!(@weak window => move |_| {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let window_name = format!("\"{}\"", WINDOW_NAME);
        let window_id = xdotool::search_windows("--name", &window_name);

        xdotool::center_window(&window_id.join(", "));
    }));

    window.show();

    let window_ids_rc = Rc::new(RefCell::new(Vec::new()));
    let window_ids_clone = Rc::clone(&window_ids_rc);

    entry.connect_changed(clone!(@weak entry, @weak list_box => move |_| {
        clear_list_box(&list_box);

        let pattern = entry.text();
        let mut window_ids = window_ids_clone.borrow_mut();

        *window_ids = xdotool::search_windows("--name", &pattern);

        if window_ids.is_empty() {
            let pattern = "\"\"";

            *window_ids = xdotool::search_windows("--name", &pattern);
        }

        populate_list_box(&window_ids, &list_box);
    }));

    let window_ids_clone = Rc::clone(&window_ids_rc);

    entry.connect_activate(
        clone!(@weak entry, @weak window, @weak list_box => move |_| {
            let mut window_ids = window_ids_clone.borrow_mut();

            // If there are more than one window, the first one that matches will be activated.
            xdotool::activate_window(&window_ids.join(", "));
            clear_list_box(&list_box);

            let pattern = "\"\"";

            *window_ids = xdotool::search_windows("--name", &pattern);
            populate_list_box(&window_ids, &list_box);

            // Without this, `entry.set_text("")` will cause a double-mutable reference error.
            drop(window_ids);
            entry.set_text("");
            window.minimize();
        }),
    );
}
