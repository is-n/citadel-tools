use std::cell::{Ref, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::colorscheme::ColorSchemeDialog;
use crate::configure_dialog::ConfigOptions;
use crate::configure_dialog::settings::CitadelSettings;
use crate::realmsd::RealmConfig;

#[derive(CompositeTemplate)]
#[template(file = "configure-dialog.ui")]
pub struct ConfigureDialog {
    #[template_child(id="bool-options-box")]
    bool_option_list: TemplateChild<gtk::ListBox>,

    #[template_child(id="overlay-combo")]
    overlay: TemplateChild<gtk::ComboBoxText>,

    #[template_child(id="realmfs-combo")]
    realmfs: TemplateChild<gtk::ComboBoxText>,

    #[template_child(id="color-scheme-button")]
    colorscheme: TemplateChild<gtk::Button>,

    #[template_child(id="frame-color-button")]
    frame_color: TemplateChild<gtk::ColorButton>,

    options: Rc<RefCell<ConfigOptions>>,

    bool_option_rows: RefCell<Vec<super::ConfigureOption>>,

    colorscheme_dialog: ColorSchemeDialog,

    settings: RefCell<CitadelSettings>,

}

impl ConfigureDialog {

    pub fn set_realm_name(&self, name: &str) {
        let color = self.settings.borrow().get_realm_color(Some(name));
        self.frame_color.set_rgba(&color);
    }

    pub fn reset_options(&self) {
        self.options.borrow_mut().reset();
        self.update_options();
    }

    pub fn set_config(&self, config: &RealmConfig) {
        self.options.borrow_mut().configure(config);
        self.realmfs.remove_all();

        self.update_options();
    }

    pub fn changes(&self) -> Vec<(String,String)> {
        self.options.borrow().changes()
    }

    pub fn store_settings(&self, realm_name: &str) {
        let color = self.frame_color.rgba();
        self.settings.borrow_mut().store_realm_color(realm_name, color);
    }

    pub fn options(&self) -> Ref<ConfigOptions> {
        self.options.borrow()
    }

    fn update_realmfs(&self) {
        self.realmfs.remove_all();
        for realmfs in self.options().realmfs_list() {
            self.realmfs.append(Some(realmfs.as_str()), realmfs.as_str());
        }
        let current = self.options().realmfs();
        self.realmfs.set_active_id(Some(&current));
    }

    fn update_options(&self) {
        let rows = self.bool_option_rows.borrow();
        for row in rows.iter() {
            row.update();
        }
        let overlay_id = self.options().overlay_id();
        self.overlay.set_active_id(Some(&overlay_id));

        self.update_realmfs();

        let scheme = self.options().colorscheme();
        self.colorscheme.set_label(scheme.name());
    }

    fn create_option_rows(&self) {
        let mut rows = self.bool_option_rows.borrow_mut();
        let options = self.options.borrow();
        for op in options.bool_options() {
            let w = super::ConfigureOption::new(op);
            self.bool_option_list.add(&w);
            rows.push(w);
        }
    }

    fn setup_overlay(&self) {
        let options = self.options.clone();
        self.overlay.connect_changed(move |combo| {
            if let Some(text) = combo.active_id() {
                options.borrow_mut().set_overlay_id(text.as_str());
            }
        });
    }

    fn setup_realmfs(&self) {
        let options = self.options.clone();
        self.realmfs.connect_changed(move |combo| {
            if let Some(text) = combo.active_text() {
                options.borrow_mut().set_realmfs(text.as_str());
            }
        });
    }

    fn setup_colorscheme(&self) {
        let dialog = self.colorscheme_dialog.clone();
        let options = self.options.clone();

        self.colorscheme.connect_clicked(move |b| {
            dialog.show_all();
            let scheme = options.borrow().colorscheme();
            dialog.set_selected_scheme(scheme.slug());

            match dialog.run() {
                gtk::ResponseType::Ok => {
                    if let Some(scheme) = dialog.get_selected_scheme() {
                        options.borrow_mut().set_colorscheme_id(scheme.slug());
                        b.set_label(scheme.name());
                    }
                },
                _ => {},
            }
            dialog.hide();
        });
    }

    fn setup_frame_color(&self) {
        let color = self.settings.borrow().get_realm_color(None);
        self.frame_color.set_rgba(&color);
    }

    fn setup_widgets(&self) {
        self.create_option_rows();
        self.setup_overlay();
        self.setup_realmfs();
        self.setup_colorscheme();
        self.setup_frame_color();
    }
}

impl Default for ConfigureDialog {
    fn default() -> Self {
        ConfigureDialog {
            bool_option_list: Default::default(),
            overlay: Default::default(),
            realmfs: Default::default(),
            colorscheme: Default::default(),
            frame_color: Default::default(),
            colorscheme_dialog: ColorSchemeDialog::new(),
            options: Rc::new(RefCell::new(ConfigOptions::new())),
            settings: RefCell::new(CitadelSettings::new()),
            bool_option_rows: RefCell::new(Vec::new()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ConfigureDialog {
    const NAME: &'static str = "ConfigureDialog";
    type Type = super::ConfigureDialog;
    type ParentType = gtk::Dialog;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }
    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ConfigureDialog {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.colorscheme_dialog.set_transient_for(Some(&self.instance()));
        self.setup_widgets();
    }
}

impl DialogImpl for ConfigureDialog {}
impl WindowImpl for ConfigureDialog {}
impl BinImpl for ConfigureDialog {}
impl ContainerImpl for ConfigureDialog {}
impl WidgetImpl for ConfigureDialog {}
