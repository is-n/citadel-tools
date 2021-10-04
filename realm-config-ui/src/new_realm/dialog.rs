use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib;
use gtk::CompositeTemplate;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::configure_dialog::ConfigureDialog;
use crate::new_realm::verifier::RealmNameVerifier;
use crate::realmsd::RealmConfig;

#[derive(CompositeTemplate)]
#[template(file = "new-realm-dialog.ui")]
pub struct NewRealmDialog {
    #[template_child]
    pub infobar: TemplateChild<gtk::InfoBar>,

    #[template_child]
    pub infolabel: TemplateChild<gtk::Label>,

    #[template_child]
    pub label: TemplateChild<gtk::Label>,

    #[template_child]
    entry: TemplateChild<gtk::Entry>,

    #[template_child (id="config-button")]
    pub config_button: TemplateChild<gtk::Button>,

    pub realm_names: Rc<RefCell<Vec<String>>>,

    configure_dialog: ConfigureDialog,
}

impl Default for NewRealmDialog {
    fn default() -> Self {
        NewRealmDialog {
            infobar: Default::default(),
            infolabel: Default::default(),
            label: Default::default(),
            entry: Default::default(),
            config_button: Default::default(),
            realm_names: Default::default(),
            configure_dialog: ConfigureDialog::new(),
        }
    }
}

impl NewRealmDialog {
    pub fn set_realm_names(&self, names: &[String]) {
        let mut lock = self.realm_names.borrow_mut();
        lock.clear();
        lock.extend_from_slice(&names)
    }

    pub fn set_config(&self, config: &RealmConfig) {
        self.configure_dialog.set_config(config);
    }

    pub fn get_realm_name(&self) -> String {
        self.entry.text().to_string()
    }

    pub fn config_changes(&self) -> Vec<(String,String)> {
        self.configure_dialog.changes()
    }

    pub fn store_config_settings(&self) {
        let realm_name = self.get_realm_name();
        if !realm_name.is_empty() {
            self.configure_dialog.store_settings(&realm_name);
        }
    }
}


#[glib::object_subclass]
impl ObjectSubclass for NewRealmDialog {
    const NAME: &'static str = "NewRealmDialog";
    type Type = super::NewRealmDialog;
    type ParentType = gtk::Dialog;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }
    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for NewRealmDialog {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        self.configure_dialog.set_transient_for(Some(&self.instance()));
        let verifier = Rc::new(RealmNameVerifier::new(self));

        self.entry.connect_insert_text(glib::clone!(@strong verifier => move |entry, text, pos|{
            if !verifier.verify_insert(entry, text, *pos) {
                entry.stop_signal_emission("insert-text");
            }
        }));

        self.entry.connect_delete_text(glib::clone!(@strong verifier => move |entry, start, end| {
            if !verifier.verify_delete(entry, start, end) {
                entry.stop_signal_emission("delete-text");
            }
        }));

        self.entry.connect_changed(glib::clone!(@strong verifier => move |entry| {
            verifier.changed(entry);
        }));

        let config_dialog = self.configure_dialog.clone();
        let entry = self.entry.clone();
        self.config_button.connect_clicked(move |_b| {
            let name  = entry.text().to_string();
            config_dialog.set_title(&format!("Configure realm-{}", name));
            config_dialog.show_all();
            match config_dialog.run() {
                gtk::ResponseType::Ok => {},
                _ => config_dialog.reset_options(),
            }
            config_dialog.hide();
        });
    }
}

impl DialogImpl for NewRealmDialog {}
impl WindowImpl for NewRealmDialog {}
impl BinImpl for NewRealmDialog {}
impl ContainerImpl for NewRealmDialog {}
impl WidgetImpl for NewRealmDialog {}