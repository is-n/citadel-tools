use gtk::glib;
use glib::subclass::prelude::*;

use crate::realmsd::RealmConfig;

mod dialog;
mod verifier;

glib::wrapper! {
    pub struct NewRealmDialog(ObjectSubclass<dialog::NewRealmDialog>)
        @extends gtk::Dialog, gtk::Window, gtk::Bin, gtk::Container, gtk::Widget,
        @implements gtk::Buildable;
}

impl NewRealmDialog {
    pub fn new() -> Self {
        glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create NewRealmDialog")
    }

    fn instance(&self) -> &dialog::NewRealmDialog {
        dialog::NewRealmDialog::from_instance(self)
    }

    pub fn set_realm_names(&self, names: &[String]) {
        self.instance().set_realm_names(names);
    }

    pub fn set_config(&self, config: &RealmConfig) {
        self.instance().set_config(config);
    }

    pub fn get_realm_name(&self) -> String {
        self.instance().get_realm_name()
    }

    pub fn config_changes(&self) -> Vec<(String,String)> {
        self.instance().config_changes()
    }

    pub fn store_config_settings(&self) {
        self.instance().store_config_settings();
    }
}
