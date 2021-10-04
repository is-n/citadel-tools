use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::configure_dialog::BoolOption;

#[derive(CompositeTemplate)]
#[template(file = "configure-option-switch.ui")]
pub struct ConfigureOption {
    #[template_child]
    pub name: TemplateChild<gtk::Label>,
    #[template_child]
    pub switch: TemplateChild<gtk::Switch>,

    pub option: RefCell<Option<BoolOption>>,
}

impl Default for ConfigureOption {
    fn default() -> Self {
        ConfigureOption {
            name: Default::default(),
            switch: Default::default(),
            option: RefCell::new(None),
        }
    }
}

impl ConfigureOption {
    pub fn set_bool_option(&self, option: &BoolOption) {
        self.name.set_text(option.description());
        self.switch.set_state(option.value());
        self.switch.connect_state_set(glib::clone!(@strong option => move |_b,v| {
            option.set_value(v);
            Inhibit(false)
        }));
        self.option.borrow_mut().replace(option.clone());
    }

    pub fn update(&self) {
        let option = self.option.borrow();
        if let Some(option) = option.as_ref() {
            self.switch.set_state(option.value());
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ConfigureOption {
    const NAME: &'static str = "ConfigureOption";
    type Type = super::ConfigureOption;
    type ParentType = gtk::ListBoxRow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }
    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ConfigureOption {}
impl WidgetImpl for ConfigureOption {}
impl ContainerImpl for ConfigureOption {}
impl BinImpl for ConfigureOption {}
impl ListBoxRowImpl for ConfigureOption {}
