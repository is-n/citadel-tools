#[macro_use] extern crate libcitadel;
use std::env;

use gtk::prelude::*;
use gtk::gio;

use crate::configure_dialog::ConfigureDialog;
use crate::new_realm::NewRealmDialog;
use crate::error::Result;
use crate::realmsd::{RealmConfig, RealmsManagerProxy};

mod realmsd;
mod error;
mod colorscheme;
mod configure_dialog;
mod new_realm;


fn load_realm_names() -> Result<(RealmsManagerProxy<'static>, Vec<String>, RealmConfig)> {
    let manager = RealmsManagerProxy::connect()?;
    let names = manager.realm_names()?;
    let config = manager.default_config()?;
    Ok((manager, names, config))
}

fn new_realm_ui(app: &gtk::Application) {
    let (manager, realms, config) = match load_realm_names() {
        Ok(v) => v,
        Err(err) => {
            err.app_error_dialog(app);
            return;
        }
    };

    let dialog = NewRealmDialog::new();
    dialog.set_realm_names(&realms);
    dialog.set_config(&config);
    app.add_window(&dialog);
    dialog.show_all();

    if dialog.run() == gtk::ResponseType::Ok {
        let realm = dialog.get_realm_name();
        dialog.store_config_settings();
        let changes = dialog.config_changes();
        if let Err(err) = manager.create_new_realm(&realm, changes) {
            err.error_dialog(Some(&dialog));
        }
    }
    dialog.close();
}

fn load_realm_config(realm_name: &str) -> Result<(RealmsManagerProxy<'static>, RealmConfig)> {
    let manager = RealmsManagerProxy::connect()?;
    let config = manager.config(realm_name)?;
    Ok((manager, config))
}

fn configure_realm_ui(app: &gtk::Application, name: &str) {
    let (manager, config) = match load_realm_config(name) {
        Ok(val) => val,
        Err(err) => {
            err.app_error_dialog(app);
            return;
        }
    };

    let dialog = ConfigureDialog::new();
    app.add_window(&dialog);
    dialog.set_config(&config);
    dialog.set_realm_name(name);
    dialog.show_all();

    if dialog.run() == gtk::ResponseType::Ok {
        dialog.store_settings(name);
        let changes = dialog.changes();
        if !changes.is_empty() {
            if let Err(err) = manager.configure_realm(name, changes) {
                err.error_dialog(Some(&dialog));
            }
        }
    }
    dialog.close();
}

fn test_ui(app: &gtk::Application) {
    let config = RealmConfig::new_default(vec![String::from("main"), String::from("foo")]);
    let dialog = ConfigureDialog::new();
    app.add_window(&dialog);
    dialog.set_config(&config);
    dialog.set_title("Configure realm-testing");
    dialog.show_all();

    if dialog.run() == gtk::ResponseType::Ok {
        let changes = dialog.changes();
        println!("Changes: {:?}", changes);
    }

    dialog.close();
}

fn main() {

    let mut args = env::args().collect::<Vec<String>>();


    if args.len() > 1 {
        let first = args.remove(1);
        let application = gtk::Application::new(Some("com.subgraph.RealmConfig"), gio::ApplicationFlags::empty());
        if first.as_str() == "--new" {
            application.connect_activate(new_realm_ui);
        } else if first.as_str() == "--test" {
            application.connect_activate(test_ui);
        } else  {
            application.connect_activate(move |app| {
                configure_realm_ui(app, &first);
            });
        }
        application.run_with_args(&args);
    }
}