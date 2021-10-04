use gtk::glib;
use glib::subclass::prelude::*;
use libcitadel::terminal::Base16Scheme;

mod dialog;
mod colorschemes;

glib::wrapper! {
    pub struct ColorSchemeDialog(ObjectSubclass<dialog::ColorSchemeDialog>)
        @extends gtk::Dialog, gtk::Window, gtk::Bin, gtk::Container, gtk::Widget,
        @implements gtk::Buildable;
}

impl ColorSchemeDialog {
    pub fn new() -> Self {
        glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create ColorSchemeDialog")
    }

    fn instance(&self) -> &dialog::ColorSchemeDialog {
        dialog::ColorSchemeDialog::from_instance(self)
    }

    pub fn get_selected_scheme(&self) -> Option<Base16Scheme> {
        self.instance().get_selected_scheme()
    }

    pub fn set_selected_scheme(&self, id: &str) {
        self.instance().set_selected_id(id);
    }
}
