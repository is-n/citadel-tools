use gio::prelude::*;
use std::fmt;

use glib::subclass::prelude::*;
use glib::ParamSpec;
use gtk::glib;

use once_cell::sync;
use std::cell::RefCell;

#[derive(Default)]
pub struct RowDataImpl {
    model: RefCell<Option<String>>,
    path: RefCell<Option<String>>,
    size: RefCell<Option<String>>,
    removable: RefCell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for RowDataImpl {
    const NAME: &'static str = "RowData";
    type Type = RowData;
    type ParentType = glib::Object;
}

impl ObjectImpl for RowDataImpl {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: sync::Lazy<Vec<glib::ParamSpec>> = sync::Lazy::new(|| {
            vec![
                glib::ParamSpecString::new(
                    "model",
                    "Model",
                    "Model",
                    None,
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "path",
                    "Path",
                    "Path",
                    None,
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "size",
                    "Size",
                    "Size",
                    None,
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecBoolean::new(
                    "removable",
                    "Removable",
                    "Removable",
                    false,
                    glib::ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "model" => {
                let model = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.model.replace(model);
            }
            "path" => {
                let path = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.path.replace(path);
            }
            "size" => {
                let size = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.size.replace(size);
            }
            "removable" => {
                let removable = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.removable.replace(removable);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "model" => self.model.borrow().to_value(),
            "path" => self.path.borrow().to_value(),
            "size" => self.size.borrow().to_value(),
            "removable" => self.removable.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

glib::wrapper! {
    pub struct RowData(ObjectSubclass<RowDataImpl>);
}

impl RowData {
    pub fn new(model: &str, path: &str, size: &str, removable: bool) -> RowData {
        glib::Object::new(&[
            ("model", &model),
            ("path", &path),
            ("size", &size),
            ("removable", &removable),
        ])
        .expect("Failed to create row data")
    }
}

impl fmt::Display for RowData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
