use gio::SimpleAction;
use gtk::prelude::*;
use gtk::{self, Application, Button};
mod builder;
mod dbus_client;
mod error;
mod rowdata;
mod ui;
use glib::clone;
use gtk::builders;

use libcitadel::CommandLine;
use ui::Ui;

pub use error::{Error, Result};

const APP_ID: &str = "com.subgraph.CitadelInstaller";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);

    // Set keyboard accelerator to trigger "win.close".
    app.set_accels_for_action("win.close", &["<Ctrl>W"]);

    app.run();
}

fn build_ui(app: &gtk::Application) {
    //if !(CommandLine::live_mode() || CommandLine::install_mode()) {

    // if the user is running this program on an already installed system
    if (false) {
        let window = gtk::ApplicationWindow::new(app);

        window.set_title(Some("Citadel Installer"));
        window.set_default_size(600, 400);

        let action_close = SimpleAction::new("close", None);
        action_close.connect_activate(clone!(@weak window => move |_, _| {
            window.close();
        }));
        window.add_action(&action_close);

        let label = gtk::Label::new(Some(
            "Citadel Installer can only be run during install mode",
        ));

        //Create a button with label
        let button = Button::builder()
            .label("Close")
            .action_name("win.close")
            .width_request(40)
            .build();

        // Here we construct the grid that is going contain our buttons.
        let grid = builders::GridBuilder::new()
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .row_spacing(30)
            .column_spacing(6)
            .build();

        grid.attach(&label, 1, 0, 5, 1);
        grid.attach(&button, 3, 1, 1, 1);

        // Add the grid in the window
        window.set_child(Some(&grid));

        window.show();
    } else {
        let ui = Ui::build(app).unwrap();
        //ui.assistant.show_all();
        ui.start();
        // match Ui::build(app) {
        //     Ok(ui) => {
        //         //ui.assistant.show_all();
        //         ui.start();
        //     }
        //     Err(err) => {
        //         println!("Could not start application: {:?}", err);
        //     }
        // }
    }
    //});
}
