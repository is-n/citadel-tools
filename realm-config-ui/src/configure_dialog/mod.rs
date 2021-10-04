use gtk::glib;
use gtk::prelude::*;
use glib::subclass::prelude::*;

use crate::realmsd::RealmConfig;
pub use crate::configure_dialog::options::{ConfigOptions,BoolOption};

mod dialog;
mod option_row;
mod options;
mod settings;

glib::wrapper! {
    pub struct ConfigureDialog(ObjectSubclass<dialog::ConfigureDialog>)
        @extends gtk::Dialog, gtk::Window, gtk::Bin, gtk::Container, gtk::Widget,
        @implements gtk::Buildable;
}

impl ConfigureDialog {
    pub fn new() -> Self {
        glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create ConfigureDialog")
    }

    fn instance(&self) -> &dialog::ConfigureDialog {
        dialog::ConfigureDialog::from_instance(self)
    }

    pub fn changes(&self) -> Vec<(String,String)> {
        self.instance().changes()
    }

    pub fn store_settings(&self, realm_name: &str) {
        self.instance().store_settings(realm_name);
    }

    pub fn reset_options(&self) {
        self.instance().reset_options();
    }

    pub fn set_realm_name(&self, name: &str) {
        self.set_title(&format!("Configure realm-{}", name));
        self.instance().set_realm_name(name);
    }

    pub fn set_config(&self, config: &RealmConfig) {
        self.instance().set_config(config);
    }
}

glib::wrapper! {
    pub struct ConfigureOption(ObjectSubclass<option_row::ConfigureOption>)
        @extends gtk::Widget, gtk::Bin, gtk::Container,
        @implements gtk::Buildable, gtk::Actionable;
}

impl ConfigureOption {
    pub fn new(option: &BoolOption) -> Self {
        let widget :Self = glib::Object::new(&[])
            .expect("Failed to create ConfigureOption");
        widget.set_bool_option(option);
        widget
    }

    fn instance(&self) -> &option_row::ConfigureOption {
        option_row::ConfigureOption::from_instance(self)
    }

    pub fn update(&self) {
        self.instance().update();
    }

    fn set_bool_option(&self, option: &BoolOption) {
        self.set_tooltip_markup(Some(option.tooltip()));
        self.instance().set_bool_option(option);
    }
}

