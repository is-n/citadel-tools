use zbus::{Connection, ObjectServer};
use crate::realms_manager::{RealmsManagerServer, REALMS_SERVER_OBJECT_PATH, realm_status};
use libcitadel::{RealmEvent, Realm};

pub struct EventHandler {
    connection: Connection,
    realms_server: RealmsManagerServer,
}

impl EventHandler {
    pub fn new(connection: Connection, realms_server: RealmsManagerServer) -> Self {
        EventHandler { connection, realms_server }
    }

    pub fn handle_event(&self, ev: &RealmEvent)  {
        if let Err(err) = self.dispatch_event(ev) {
            warn!("Error emitting signal for realm event {}: {}", ev, err);
        }
    }

    fn dispatch_event(&self, ev: &RealmEvent) -> zbus::Result<()> {
        match ev {
            RealmEvent::Started(realm) => self.on_started(realm),
            RealmEvent::Stopped(realm) => self.on_stopped(realm),
            RealmEvent::New(realm) => self.on_new(realm),
            RealmEvent::Removed(realm) => self.on_removed(realm),
            RealmEvent::Current(realm) => self.on_current(realm.as_ref()),
        }
    }

    fn with_server<F>(&self, func: F) -> zbus::Result<()>
        where
            F: Fn(&RealmsManagerServer) -> zbus::Result<()>,
    {
        let mut object_server = ObjectServer::new(&self.connection);
        object_server.at(REALMS_SERVER_OBJECT_PATH, self.realms_server.clone())?;
        object_server.with(REALMS_SERVER_OBJECT_PATH, |iface: &RealmsManagerServer| func(iface))
    }

    fn on_started(&self, realm: &Realm) -> zbus::Result<()> {
        let namespace = realm.pid_namespace().unwrap_or(String::new());
        let status = realm_status(realm);
        self.with_server(|server| server.realm_started(realm.name(), namespace.as_str(), status))
    }

    fn on_stopped(&self, realm: &Realm) -> zbus::Result<()> {
        let status = realm_status(realm);
        self.with_server(|server| server.realm_stopped(realm.name(), status))
    }

    fn on_new(&self, realm: &Realm) -> zbus::Result<()> {
        let status = realm_status(realm);
        let description = realm.notes().unwrap_or(String::new());
        self.with_server(|server| server.realm_new(realm.name(), &description, status))
    }

    fn on_removed(&self, realm: &Realm) -> zbus::Result<()> {
        self.with_server(|server| server.realm_removed(realm.name()))
    }

    fn on_current(&self, realm: Option<&Realm>) -> zbus::Result<()> {
        self.with_server(|server| {
            match realm {
                Some(realm) => server.realm_current(realm.name(), realm_status(realm)),
                None => server.realm_current("", 0),
            }
        })
    }
}
