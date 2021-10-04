#[macro_use] extern crate libcitadel;

use zbus::{Connection, fdo};

use libcitadel::{Logger, LogLevel, Result};

use crate::realms_manager::RealmsManagerServer;

mod realms_manager;
mod events;


fn main() {
    if let Err(e) = run_realm_manager() {
        warn!("Error: {}", e);
    }
}

fn create_system_connection() -> zbus::Result<Connection> {
    let connection = zbus::Connection::new_system()?;
    fdo::DBusProxy::new(&connection)?.request_name("com.subgraph.realms", fdo::RequestNameFlags::AllowReplacement.into())?;
    Ok(connection)
}

fn run_realm_manager() -> Result<()> {
    Logger::set_log_level(LogLevel::Verbose);

    let connection = create_system_connection()
        .map_err(context!("ZBus Connection error"))?;

    let mut object_server = RealmsManagerServer::register(&connection)?;

    loop {
        if let Err(err) = object_server.try_handle_next() {
            warn!("Error handling DBus message: {}", err);
        }
    }

}
