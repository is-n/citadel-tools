use libcitadel::{RealmManager, Realm, OverlayType, Result};
use std::sync::Arc;
use zbus::{dbus_interface, ObjectServer,Connection};
use zvariant::derive::Type;
use std::thread;
use std::collections::HashMap;
use serde::{Serialize,Deserialize};
use crate::events::EventHandler;
use libcitadel::terminal::Base16Scheme;

pub const REALMS_SERVER_OBJECT_PATH: &str = "/com/subgraph/realms";

#[derive(Clone)]
pub struct RealmsManagerServer {
    manager: Arc<RealmManager>,
}

const BOOL_CONFIG_VARS: &[&str] = &[
    "use-gpu", "use-wayland", "use-x11", "use-sound",
    "use-shared-dir", "use-network", "use-kvm", "use-ephemeral-home"
];

fn is_bool_config_variable(variable: &str) -> bool {
    BOOL_CONFIG_VARS.iter().any(|&s| s == variable)
}

fn save_config(realm: &Realm) {
    let path = realm.base_path_file("config");
    if let Err(e) = realm.config().write_to(&path) {
        warn!("Error writing config file {}: {}", path.display(), e);
    }
}

fn configure_realm_boolean_config(realm: &Realm, variable: &str, value: &str) {

    let val = match value {
        "true" => true,
        "false" => false,
        _ => {
            warn!("Not a valid boolean value '{}'", value);
            return;
        },
    };

    realm.with_mut_config(|c| {
        match variable {
            "use-gpu"            if c.gpu() != val            => c.use_gpu = Some(val),
            "use-wayland"        if c.wayland() != val        => c.use_wayland = Some(val),
            "use-x11"            if c.x11() != val            => c.use_x11 = Some(val),
            "use-sound"          if c.sound() != val          => c.use_sound = Some(val),
            "use-shared-dir"     if c.shared_dir() != val     => c.use_shared_dir = Some(val),
            "use-network"        if c.network() != val        => c.use_network = Some(val),
            "use-kvm"            if c.kvm() != val            => c.use_kvm = Some(val),
            "use-ephemeral-home" if c.ephemeral_home() != val => c.use_ephemeral_home = Some(val),
            _ => {},
        }
    });
    save_config(realm);
}

fn configure_realm(realm: &Realm, variable: &str, value: &str) {
    if is_bool_config_variable(variable) {
        configure_realm_boolean_config(realm, variable, value);
    } else if variable == "overlay" {
        if value == "tmpfs" || value == "storage" || value == "none" {
            realm.with_mut_config(|c| {
                c.overlay = Some(value.to_string());
            });
            save_config(realm);
        } else {
            warn!("Invalid storage type '{}'", value);
            return;
        }
    } else if variable == "terminal-scheme" {
        if Base16Scheme::by_name(value).is_none() {
            warn!("No terminal color scheme with name '{}' available", value);
        }
        realm.with_mut_config(|c| {
            c.terminal_scheme = Some(value.to_string());
        });
        save_config(realm);
    } else if variable == "realmfs" {
        warn!("Changing realmfs config variable not implemented");
    } else {
        warn!("Unknown config variable '{}'", variable);
    }
}

impl RealmsManagerServer {

    fn register_events(&self, connection: &Connection) -> Result<()> {
        let events = EventHandler::new(connection.clone(), self.clone());
        self.manager.add_event_handler(move |ev| events.handle_event(ev));
        self.manager.start_event_task()
    }

    pub fn register(connection: &Connection) -> Result<ObjectServer> {
        let manager = RealmManager::load()?;
        let iface = RealmsManagerServer { manager };
        iface.register_events(connection)?;
        let mut object_server = ObjectServer::new(connection);
        object_server.at(REALMS_SERVER_OBJECT_PATH, iface).map_err(context!("ZBus error"))?;
        Ok(object_server)
    }

}


#[dbus_interface(name = "com.subgraph.realms.Manager")]
impl RealmsManagerServer {

    fn set_current(&self, name: &str) {
        if let Some(realm) = self.manager.realm_by_name(name) {
            if let Err(err) = self.manager.set_current_realm(&realm) {
                warn!("set_current_realm({}) failed: {}", name, err);
            }
        }
    }

    fn get_current(&self) -> String {
        match self.manager.current_realm() {
            Some(realm) => realm.name().to_string(),
            None => String::new(),
        }
    }

    fn list(&self) -> Vec<RealmItem> {
        let mut realms = Vec::new();
        for r in self.manager.realm_list() {
            realms.push(RealmItem::new_from_realm(&r));
        }
        realms
    }

    fn start(&self, name: &str) {
        let realm = match self.manager.realm_by_name(name) {
            Some(r) => r,
            None => return,
        };
        let manager = self.manager.clone();

        thread::spawn(move || {
            if let Err(e) = manager.start_realm(&realm) {
                warn!("failed to start realm {}: {}", realm.name(), e);
            }
        });
    }

    fn stop(&self, name: &str) {
        let realm = match self.manager.realm_by_name(name) {
            Some(r) => r,
            None => return,
        };
        let manager = self.manager.clone();

        thread::spawn(move || {
            if let Err(e) = manager.stop_realm(&realm) {
                warn!("failed to stop realm {}: {}", realm.name(), e);
            }
        });
    }

    fn restart(&self, name: &str) {
        let realm = match self.manager.realm_by_name(name) {
            Some(r) => r,
            None => return,
        };
        let manager = self.manager.clone();

        thread::spawn(move || {
            if let Err(e) = manager.stop_realm(&realm) {
                warn!("failed to stop realm {}: {}", realm.name(), e);
            } else if let Err(e) = manager.start_realm(&realm) {
                warn!("failed to restart realm {}: {}", realm.name(), e);
            }
        });
    }

    fn terminal(&self, name: &str) {
        let realm = match self.manager.realm_by_name(name) {
            Some(r) => r,
            None => return,
        };
        let manager = self.manager.clone();

        thread::spawn(move || {
            if !realm.is_active() {
                if let Err(err) = manager.start_realm(&realm) {
                    warn!("failed to start realm {}: {}", realm.name(), err);
                    return;
                }
            }
            if let Err(err) = manager.launch_terminal(&realm) {
                warn!("error launching terminal for realm {}: {}", realm.name(), err);
            }
        });
    }

    fn run(&self, name: &str, args: Vec<String>) {
        let realm = match self.manager.realm_by_name(name) {
            Some(r) => r,
            None => return,
        };
        let manager = self.manager.clone();

        thread::spawn(move || {
            if !realm.is_active() {
                if let Err(err) = manager.start_realm(&realm) {
                    warn!("failed to start realm {}: {}", realm.name(), err);
                    return;
                }
            }
            if let Err(err) = manager.run_in_realm(&realm, &args, true) {
                warn!("error running {:?} in realm {}: {}", args, realm.name(), err);
            }
        });
    }

    fn realm_from_citadel_pid(&self, pid: u32) -> String {
        match self.manager.realm_by_pid(pid) {
            Some(r) => r.name().to_string(),
            None => String::new(),
        }
    }

    fn realm_config(&self, name: &str) -> RealmConfig {
        let realm = match self.manager.realm_by_name(name) {
            Some(r) => r,
            None => return RealmConfig::new(),
        };
        RealmConfig::new_from_realm(&realm)
    }

    fn realm_set_config(&self, name: &str, vars: Vec<(String,String)>) {
        let realm = match self.manager.realm_by_name(name) {
            Some(r) => r,
            None => {
                warn!("No realm named '{}' found in RealmSetConfig", name);
                return;
            },
        };

        for var in &vars {
            configure_realm(&realm, &var.0, &var.1);
        }
    }

    fn realm_exists(&self, name: &str) -> bool {
        Realm::is_valid_name(name) && self.manager.realm_by_name(name).is_some()
    }

    fn create_realm(&self, name: &str) -> bool {
        if let Err(err) = self.manager.new_realm(name) {
            warn!("Error creating realm ({}): {}", name, err);
            false
        } else {
            true
        }
    }

    fn list_realm_f_s(&self) -> Vec<String> {
        self.manager.realmfs_list()
            .into_iter()
            .map(|fs| fs.name().to_owned())
            .collect()
    }

    fn update_realm_f_s(&self, _name: &str) {

    }

    #[dbus_interface(signal)]
    pub fn realm_started(&self, realm: &str, namespace: &str, status: u8) -> zbus::Result<()> { Ok(()) }

    #[dbus_interface(signal)]
    pub fn realm_stopped(&self, realm: &str, status: u8) -> zbus::Result<()> { Ok(()) }

    #[dbus_interface(signal)]
    pub fn realm_new(&self, realm: &str, description: &str, status: u8) -> zbus::Result<()> { Ok(()) }

    #[dbus_interface(signal)]
    pub fn realm_removed(&self, realm: &str) -> zbus::Result<()> { Ok(()) }

    #[dbus_interface(signal)]
    pub fn realm_current(&self, realm: &str, status: u8) -> zbus::Result<()> { Ok(()) }

    #[dbus_interface(signal)]
    pub fn service_started(&self) -> zbus::Result<()> { Ok(()) }

}

const STATUS_REALM_RUNNING: u8 = 1;
const STATUS_REALM_CURRENT: u8 = 2;
const STATUS_REALM_SYSTEM_REALM: u8  = 4;

pub fn realm_status(realm: &Realm) -> u8 {
    let mut status = 0;
    if realm.is_active() {
        status |= STATUS_REALM_RUNNING;
    }
    if realm.is_current() {
        status |= STATUS_REALM_CURRENT;
    }
    if realm.is_system() {
        status |= STATUS_REALM_SYSTEM_REALM;
    }
    status
}

#[derive(Deserialize,Serialize,Type)]
struct RealmItem {
    name: String,
    description: String,
    realmfs: String,
    namespace: String,
    status: u8,
}

impl RealmItem {
    fn new_from_realm(realm: &Realm) -> Self {
        let name = realm.name().to_string();
        let description = realm.notes().unwrap_or(String::new());
        let realmfs = realm.config().realmfs().to_string();
        let namespace = realm.pid_namespace().unwrap_or(String::new());
        let status = realm_status(realm);
        RealmItem { name, description, realmfs, namespace, status }
    }
}

#[derive(Deserialize,Serialize,Type)]
struct RealmConfig {
    items: HashMap<String,String>,
}

impl RealmConfig {
    fn new() -> Self {
        RealmConfig { items: HashMap::new() }
    }

    fn new_from_realm(realm: &Realm) -> Self {
        let mut this = RealmConfig { items: HashMap::new() };
        let config = realm.config();
        this.add_bool("use-gpu", config.gpu());
        this.add_bool("use-wayland", config.wayland());
        this.add_bool("use-x11", config.x11());
        this.add_bool("use-sound", config.sound());
        this.add_bool("use-shared-dir", config.shared_dir());
        this.add_bool("use-network", config.network());
        this.add_bool("use-kvm", config.kvm());
        this.add_bool("use-ephemeral-home", config.ephemeral_home());

        let overlay = match config.overlay() {
            OverlayType::None => "none",
            OverlayType::TmpFS => "tmpfs",
            OverlayType::Storage => "storage",
        };
        this.add("overlay", overlay);

        let scheme = match config.terminal_scheme() {
            Some(name) => name.to_string(),
            None => String::new(),
        };
        this.add("terminal-scheme", scheme);
        this.add("realmfs", config.realmfs());
        this
    }

    fn add_bool(&mut self, name: &str, val: bool) {
        let valstr = if val { "true".to_string() } else { "false".to_string() };
        self.add(name, valstr);
    }

    fn add<S,T>(&mut self, k: S, v: T) where S: Into<String>, T: Into<String> {
        self.items.insert(k.into(), v.into());
    }
}
