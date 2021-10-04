use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use libcitadel::terminal::{Base16Scheme, Color};

use crate::colorscheme::colorschemes::ColorSchemes;

#[derive(CompositeTemplate)]
#[template(file = "colorscheme-dialog.ui")]
pub struct ColorSchemeDialog {
    #[template_child(id="colorscheme-tree")]
    tree: TemplateChild<gtk::TreeView>,

    #[template_child]
    treemodel: TemplateChild<gtk::TreeStore>,

    #[template_child(id="colorscheme-label")]
    preview: TemplateChild<gtk::Label>,

    css_provider: gtk::CssProvider,

    colorschemes: ColorSchemes,

    tracker: RefCell<Option<SelectionTracker>>,
}

#[derive(Clone)]
struct SelectionTracker {
    model: gtk::TreeStore,
    selection: gtk::TreeSelection,
    preview: gtk::Label,
    colorschemes: ColorSchemes,
    css_provider: gtk::CssProvider,
}

impl SelectionTracker {
    fn new(dialog: &ColorSchemeDialog) -> Self {
        let tracker = SelectionTracker {
            model: dialog.treemodel.clone(),
            selection: dialog.tree.selection(),
            preview: dialog.preview.clone(),
            colorschemes: dialog.colorschemes.clone(),
            css_provider: dialog.css_provider.clone(),
        };
        tracker.selection.connect_changed(glib::clone!(@strong tracker => move |_| {
            if let Some(id) = tracker.selected_id() {
                if let Some((text, background)) = tracker.colorschemes.preview_scheme(&id) {
                    tracker.set_preview_background(background);
                    tracker.preview.set_markup(&text);
                }
            }
        }));
        tracker
    }

    fn selected_id(&self) -> Option<String> {
        self.selection.selected().and_then(|(model,iter)| {
            model.value(&iter, 0).get::<String>().ok()
        })
    }

    fn set_preview_background(&self, color: Color) {
        const CSS: &str =
r##"
#colorscheme-label {
  background-color: $COLOR;
  font-family: monospace;
  font-size: 14pt;
}
"##;
        let (r, g, b) = color.rgb();
        let css = CSS.replace("$COLOR", &format!("#{:02x}{:02x}{:02x}", r, g, b));
        if let Err(e) = self.css_provider.load_from_data(css.as_bytes()) {
            warn!("Error loading CSS provider data: {}", e);
        }
    }

    fn set_selected_id(&self, id: &str) {
        self.model.foreach(glib::clone!(@strong self.selection as selection => move |model, _path, iter| {
            if let Ok(ref s) = model.value(iter, 0).get::<String>() {
                if s == id {
                    selection.select_iter(iter);
                    return true;
                }
            }
            false
        }))
    }
}

impl ColorSchemeDialog {
    pub fn set_selected_id(&self, colorscheme_id: &str) {
        let tracker = self.tracker.borrow();
        if let Some(tracker) = tracker.as_ref() {
            tracker.set_selected_id(colorscheme_id);
        }
    }

    pub fn get_selected_scheme (&self) -> Option<Base16Scheme> {
        let tracker = self.tracker.borrow();
        tracker.as_ref().and_then(|t| t.selected_id())
            .and_then(|id| Base16Scheme::by_name(&id))
            .cloned()
    }
}

impl Default for ColorSchemeDialog {
    fn default() -> Self {
        ColorSchemeDialog {
            tree: Default::default(),
            treemodel: Default::default(),
            preview: Default::default(),
            css_provider: gtk::CssProvider::new(),
            colorschemes: ColorSchemes::new(),
            tracker: RefCell::new(None),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ColorSchemeDialog {
    const NAME: &'static str = "ColorSchemeDialog";
    type Type = super::ColorSchemeDialog;
    type ParentType = gtk::Dialog;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }
    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ColorSchemeDialog {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        self.preview.set_widget_name("colorscheme-label");
        self.preview.style_context().add_provider(&self.css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        self.colorschemes.populate_tree_model(&self.treemodel);
        let tracker = SelectionTracker::new(self);
        self.tracker.borrow_mut().replace(tracker);
    }
}

impl DialogImpl for ColorSchemeDialog {}
impl WindowImpl for ColorSchemeDialog {}
impl BinImpl for ColorSchemeDialog {}
impl ContainerImpl for ColorSchemeDialog {}
impl WidgetImpl for ColorSchemeDialog {}
