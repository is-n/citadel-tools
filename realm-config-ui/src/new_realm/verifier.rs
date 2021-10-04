use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use libcitadel::Realm;

use crate::new_realm::dialog::NewRealmDialog;

pub struct RealmNameVerifier {
    ok: gtk::Widget,
    infobar: gtk::InfoBar,
    infolabel: gtk::Label,
    label: gtk::Label,
    config: gtk::Button,
    realms: Rc<RefCell<Vec<String>>>,
}

impl RealmNameVerifier {
    pub fn new(dialog: &NewRealmDialog) -> Self {
        let ok = dialog.instance().widget_for_response(gtk::ResponseType::Ok).expect("No Ok Widget found");
        RealmNameVerifier {
            ok,
            infobar: dialog.infobar.clone(),
            infolabel: dialog.infolabel.clone(),
            label: dialog.label.clone(),
            config: dialog.config_button.clone(),
            realms: dialog.realm_names.clone(),
        }
    }

    pub fn verify_insert(&self, entry: &gtk::Entry, text: &str, pos: i32) -> bool {
        let mut s = entry.text().to_string();
        s.insert_str(pos as usize, text);
        Realm::is_valid_name(&s)
    }

    pub fn verify_delete(&self, entry: &gtk::Entry, start: i32, end: i32) -> bool {
        let mut s = entry.text().to_string();
        let start = start as usize;
        let end = end as usize;
        s.replace_range(start..end, "");
        s.is_empty() || Realm::is_valid_name(&s)
    }

    fn verify_name (&self, name: &String) -> bool {
        if self.realms.borrow().contains(name) {
            self.infolabel.set_markup(&format!("Realm already exists with name <b>realm-{}</b>", name));
            self.infobar.set_revealed(true);
            false
        } else {
            self.infobar.set_revealed(false);
            self.infolabel.set_markup("");
            !name.is_empty()
        }
    }

    pub fn changed(&self, entry: &gtk::Entry) {
        let s = entry.text().to_string();

        if self.verify_name(&s) {
            self.ok.set_sensitive(true);
            self.config.set_sensitive(true);
            self.label.set_markup(&format!("<b>realm-{}</b>", s));
        } else {
            self.ok.set_sensitive(false);
            self.config.set_sensitive(false);
            if s.is_empty() {
                self.label.set_markup("Enter name for new realm:");
            } else {
                self.label.set_markup("");
            }
        }
    }
}
