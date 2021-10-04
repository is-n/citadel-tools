use std::collections::HashMap;
use std::rc::Rc;

use zbus::dbus_proxy;
use zvariant::derive::Type;
use serde::{Serialize,Deserialize};

use crate::error::{Error, Result};

#[derive(Deserialize,Serialize,Type)]
pub struct RealmItem {
    name: String,
    description: String,
    realmfs: String,
    namespace: String,
    status: u8,
}

impl RealmItem {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug,Clone)]
pub struct RealmConfig {
    options: Rc<HashMap<String,String>>,
    realmfs_list: Rc<Vec<String>>,
}

impl RealmConfig {
    pub fn new_default(realmfs_list: Vec<String>) -> Self {
        let config = libcitadel::RealmConfig::default();
        let mut vars = HashMap::new();
        vars.insert("use-gpu".to_string(), config.gpu().to_string());
        vars.insert("use-wayland".to_string(), config.wayland().to_string());
        vars.insert("use-x11".to_string(), config.x11().to_string());
        vars.insert("use-sound".to_string(), config.sound().to_string());
        vars.insert("use-shared-dir".to_string(), config.shared_dir().to_string());
        vars.insert("use-network".to_string(), config.network().to_string());
        vars.insert("use-kvm".to_string(), config.kvm().to_string());
        vars.insert("use-ephemeral-home".to_string(), config.ephemeral_home().to_string());

        if realmfs_list.contains(&String::from("main")) {
            vars.insert("realmfs".to_string(), String::from("main"));
        } else if let Some(first) = realmfs_list.first() {
            vars.insert("realmfs".to_string(), first.clone());
        }
        Self::new(vars, realmfs_list)
    }

    fn new(options: HashMap<String, String>, realmfs_list: Vec<String>) -> Self {
        RealmConfig {
            options: Rc::new(options),
            realmfs_list: Rc::new(realmfs_list),
        }
    }

    pub fn get_string(&self, id: &str) -> Option<&str> {
        self.options.get(id).map(|s| s.as_str())
    }

    fn parse_bool(val: &str) -> bool {
        match val.parse::<bool>() {
            Ok(v) => v,
            _ => {
                warn!("Failed to parse value '{}' as bool", val);
                false
            }
        }
    }

    pub fn get_bool(&self, id: &str) -> bool {
        match self.get_string(id) {
            Some(val) => Self::parse_bool(val),
            None => {
                warn!("No value found for option '{}'", id);
                false
            }
        }
    }

    pub fn realmfs_list(&self) -> &[String] {
        &self.realmfs_list
    }
}

#[dbus_proxy(
default_service = "com.subgraph.realms",
interface = "com.subgraph.realms.Manager",
default_path = "/com/subgraph/realms"
)]
pub trait RealmsManager {
    fn get_current(&self) -> zbus::Result<String>;
    fn realm_set_config(&self, name: &str, vars: Vec<(String,String)>) -> zbus::Result<()>;
    fn list(&self) -> zbus::Result<Vec<RealmItem>>;
    fn realm_config(&self, name: &str) -> zbus::Result<HashMap<String,String>>;
    fn realm_exists(&self, name: &str) -> zbus::Result<bool>;
    fn list_realm_f_s(&self) -> zbus::Result<Vec<String>>;
    fn create_realm(&self, name: &str) -> zbus::Result<bool>;
}

impl RealmsManagerProxy<'_> {
    pub fn connect() -> Result<Self> {
        let connection = zbus::Connection::new_system()?;

        let proxy = RealmsManagerProxy::new(&connection)
            .map_err(|_| Error::ManagerConnect)?;

        // Test connection
        proxy.get_current().map_err(|_| Error::ManagerConnect)?;

        Ok(proxy)
    }

    pub fn realm_names(&self) -> Result<Vec<String>> {
        let realms = self.list()?;
        let names = realms.iter()
            .map(|r| r.name().to_string())
            .collect();
        Ok(names)
    }

    pub fn default_config(&self) -> Result<RealmConfig> {
        let realmfs_list = self.list_realm_f_s()?;
        Ok(RealmConfig::new_default(realmfs_list))
    }

    pub fn config(&self, realm: &str) -> Result<RealmConfig> {
        if !self.realm_exists(realm)? {
            return Err(Error::NoSuchRealm(realm.to_string()));
        }

        let options = self.realm_config(realm)?;
        let realmfs_list = self.list_realm_f_s()?;
        Ok(RealmConfig::new(options, realmfs_list))
    }

    pub fn configure_realm(&self, realm: &str, config: Vec<(String, String)>) -> Result<()> {
        self.realm_set_config(realm, config)?;
        Ok(())
    }

    pub fn create_new_realm(&self, realm: &str, config: Vec<(String, String)>) -> Result<()> {
        if self.create_realm(realm)? && !config.is_empty() {
            self.realm_set_config(realm, config)?;
        } else {
            return Err(Error::CreateRealmFailed);
        }

        Ok(())
    }
}
