use std::cell::Cell;
use std::rc::Rc;

use libcitadel::OverlayType;
use libcitadel::terminal::Base16Scheme;

use crate::realmsd::RealmConfig;

const GPU_TOOLTIP: &str = r#"If enabled the render node device <tt><b>/dev/dri/renderD128</b></tt> will be mounted into the realm container.

If privileged device <tt><b>/dev/dri/card0</b></tt> is also needed set
additional variable in realm configuration file:

    <tt><b>use-gpu-card0 = true</b></tt>

"#;
const WAYLAND_TOOLTIP: &str = "\
If enabled access to Wayland display will be permitted in realm by adding wayland socket to realm.

  <tt><b>/run/user/1000/wayland-0</b></tt>

";

const X11_TOOLTIP: &str = "\
If enabled access to X11 server will be added by mounting directory X11 directory into realm.

  <tt><b>/tmp/.X11-unix</b></tt>
";

const SOUND_TOOLTIP: &str = r#"If enabled allows use of sound inside of realm. The following items will be added:

  <tt><b>/dev/snd</b></tt>
  <tt><b>/dev/shm</b></tt>
  <tt><b>/run/user/1000/pulse</b></tt>
"#;

const SHARED_DIR_TOOLTIP: &str = r#"If enabled the shared directory will be mounted as <tt><b>/Shared</b></tt> in home directory of realm.

This directory is shared between all realms with this option enabled and is an easy way to move files between realms.
"#;

const NETWORK_TOOLTIP: &str = "\
If enabled the realm will have access to the network.
";

const KVM_TOOLTIP: &str = r#"If enabled device <tt><b>/dev/kvm</b></tt> will be added to realm.

This allows use of applications such as Qemu inside of realms.
"#;

const EPHERMERAL_HOME_TOOLTIP: &str = r#"If enabled the home directory of realm will be set up in ephemeral mode.

The ephemeral home directory is set up with the following steps:

  1. Home directory is mounted as tmpfs filesystem
  2. Any files in <tt><b>/realms/skel</b></tt> are copied into home directory
  3. Any files in <tt><b>/realms/realm-$name/skel</b></tt> are copied into home directory.
  4. Any directories listed in config file variable <tt><b>ephemeral_persistent_dirs</b></tt>
     are bind mounted from <tt><b>/realms/realm-$name/home</b></tt> into ephemeral
     home directory.
"#;

const BOOL_OPTIONS: &[(&str, &str, &str)] = &[
    ("use-gpu", "Use GPU in Realm", GPU_TOOLTIP),
    ("use-wayland", "Use Wayland in Realm", WAYLAND_TOOLTIP),
    ("use-x11", "Use X11 in Realm", X11_TOOLTIP),
    ("use-sound", "Use Sound in Realm", SOUND_TOOLTIP),
    ("use-shared-dir", "Mount /Shared directory in Realm", SHARED_DIR_TOOLTIP),
    ("use-network", "Realm has network access", NETWORK_TOOLTIP),
    ("use-kvm", "Use KVM (/dev/kvm) in Realm", KVM_TOOLTIP),
    ("use-ephemeral-home", "Use ephemeral tmpfs mount for home directory", EPHERMERAL_HOME_TOOLTIP),
];

#[derive(Clone)]
pub struct BoolOption {
    id: String,
    description: String,
    tooltip: String,
    original: Rc<Cell<bool>>,
    value: Rc<Cell<bool>>,
}

impl BoolOption {
    fn create_options() -> Vec<BoolOption> {
        let mut bools = Vec::new();
        for (id, description, tooltip) in BOOL_OPTIONS {
            bools.push(BoolOption::new(id, description, tooltip));
        }
        bools
    }

    fn new(id: &str, description: &str, tooltip: &str) -> Self {
        let id = id.to_string();
        let description = description.to_string();
        let tooltip = format!("<b><big>{}</big></b>\n\n{}", description, tooltip);
        let value = Rc::new(Cell::new(false));
        let original = Rc::new(Cell::new(false));
        BoolOption { id, description, tooltip, original, value }
    }

    pub fn value(&self) -> bool {
        self.value.get()
    }

    fn has_changed(&self) -> bool {
        self.value() != self.original.get()
    }

    pub fn set_value(&self, v: bool) {
        self.value.set(v);
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn tooltip(&self) -> &str {
        &self.tooltip
    }

    fn configure(&self, config: &RealmConfig) {
        let v = config.get_bool(self.id());
        self.original.set(v);
        self.value.set(v);
    }

    fn reset(&self) {
        self.set_value(self.original.get());
    }

    fn add_changes(&self, result: &mut Vec<(String, String)>) {
        if self.has_changed() {
            let k = self.id.clone();
            let v = self.value().to_string();
            result.push((k, v))
        }
    }
}

struct OverlayOption {
    original: OverlayType,
    current: OverlayType,
}

impl OverlayOption {
    fn new() -> Self {
        OverlayOption {
            original: OverlayType::None,
            current: OverlayType::None,
        }
    }

    fn overlay_str_to_enum(str: Option<&str>) -> OverlayType {
        match str {
            Some("storage") => OverlayType::Storage,
            Some("tmpfs") => OverlayType::TmpFS,
            Some("none") => OverlayType::None,
            None => OverlayType::None,
            Some(s) => {
                warn!("Unexpected overlay type: {}", s);
                OverlayType::None
            },
        }
    }

    fn set_overlay(&mut self, overlay: &str) {
        self.current = Self::overlay_str_to_enum(Some(overlay));
    }

    fn str_value(&self) -> String {
        self.current.to_str_value()
            .unwrap_or("none").to_string()
    }

    fn configure(&mut self, config: &RealmConfig) {
        let overlay = Self::overlay_str_to_enum(config.get_string("overlay"));
        self.original = overlay;
        self.current = overlay;
    }

    fn reset(&mut self) {
        self.current = self.original;
    }

    fn add_changes(&self, result: &mut Vec<(String, String)>) {
        if self.original != self.current {
            let k = "overlay".to_string();
            let v = self.str_value();
            result.push((k, v));
        }
    }
}

struct RealmFsOption {
    original: String,
    current: String,
    realmfs_list: Vec<String>,
}

impl RealmFsOption {

    fn new() -> Self {
        let base = String::from("base");
        RealmFsOption {
            original: base.clone(),
            current: base.clone(),
            realmfs_list: vec![base],
        }
    }

    fn realmfs_list(&self) -> Vec<String> {
        self.realmfs_list.clone()
    }

    fn current(&self) -> String {
        self.current.clone()
    }

    fn set_current(&mut self, realmfs: &str) {
        self.current = realmfs.to_string();
    }

    fn configure(&mut self, config: &RealmConfig) {
        if let Some(realmfs) = config.get_string("realmfs") {

            self.realmfs_list.clear();
            self.realmfs_list.extend(config.realmfs_list().iter().cloned());
            self.original = realmfs.to_string();
            self.current = realmfs.to_string();
        }
    }

    fn reset(&mut self) {
        self.current = self.original.clone();
    }

    fn add_changes(&self, result: &mut Vec<(String, String)>) {
        if self.current.is_empty() {
            return;
        }

        if self.current != self.original {
            result.push(("realmfs".to_string(), self.current.clone()))
        }
    }
}

const DEFAULT_SCHEME: &str = "default-dark";

struct ColorSchemeOption {
    original: Base16Scheme,
    current: Base16Scheme,
}

impl ColorSchemeOption {
    fn new() -> Self {
        let scheme = Base16Scheme::by_name(DEFAULT_SCHEME)
            .expect("default Base16Scheme");

        ColorSchemeOption {
            original: scheme.clone(),
            current: scheme.clone(),
        }
    }

    fn configure(&mut self, config: &RealmConfig) {
        if let Some(scheme) = config.get_string("terminal-scheme") {
            if let Some(scheme) = Base16Scheme::by_name(scheme) {
                self.original = scheme.clone();
                self.current = scheme.clone();
            }
        }
    }

    fn reset(&mut self) {
        self.set_current(self.original.clone());
    }

    fn set_current(&mut self, scheme: Base16Scheme) {
        self.current = scheme;
    }

    fn set_current_id(&mut self, id: &str) {
        if let Some(scheme) = Base16Scheme::by_name(id) {
            self.set_current(scheme.clone());
        }
    }

    fn current(&self) -> Base16Scheme {
        self.current.clone()
    }

    fn add_changes(&self, result: &mut Vec<(String, String)>) {
        if self.original.slug() != self.current.slug() {
            result.push(("terminal-scheme".to_string(), self.current.slug().to_string()));
        }
    }
}

pub struct ConfigOptions {
    bool_options: Vec<BoolOption>,
    overlay: OverlayOption,
    realmfs: RealmFsOption,
    colorscheme: ColorSchemeOption,
}

impl ConfigOptions {

    pub fn configure(&mut self, config: &RealmConfig) {
        for op in &self.bool_options {
            op.configure(config);
        }
        self.overlay.configure(config);
        self.realmfs.configure(config);
        self.colorscheme.configure(config);

    }

    pub fn reset(&mut self) {
        for op in &self.bool_options {
            op.reset();
        }
        self.overlay.reset();
        self.realmfs.reset();
        self.colorscheme.reset();
    }

    pub fn changes(&self) -> Vec<(String,String)> {
        let mut changes = Vec::new();
        for op in &self.bool_options {
            op.add_changes(&mut changes);
        }
        self.overlay.add_changes(&mut changes);
        self.realmfs.add_changes(&mut changes);
        self.colorscheme.add_changes(&mut changes);
        changes
    }

    pub fn new() -> Self {
        let bool_options = BoolOption::create_options();
        let overlay = OverlayOption::new();
        let realmfs = RealmFsOption::new();
        let colorscheme = ColorSchemeOption::new();
        ConfigOptions {
            bool_options, overlay, realmfs, colorscheme,
        }
    }

    pub fn bool_options(&self) -> &[BoolOption] {
        &self.bool_options
    }

    pub fn realmfs_list(&self) -> Vec<String> {
        self.realmfs.realmfs_list()
    }

    pub fn overlay_id(&self) -> String {
        self.overlay.str_value()
    }

    pub fn set_overlay_id(&mut self, id: &str) {
        self.overlay.set_overlay(id);
    }

    pub fn realmfs(&self) -> String {
        self.realmfs.current()
    }

    pub fn set_realmfs(&mut self, realmfs: &str) {
        self.realmfs.set_current(realmfs);
    }

    pub fn colorscheme(&self) -> Base16Scheme {
        self.colorscheme.current()
    }

    pub fn set_colorscheme_id(&mut self, id: &str) {
        self.colorscheme.set_current_id(id);
    }
}