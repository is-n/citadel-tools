use std::result;
use std::fmt;
use crate::error::Error::Zbus;
use std::fmt::Formatter;
use gtk::prelude::*;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Zbus(zbus::Error),
    ManagerConnect,
    NoSuchRealm(String),
    CreateRealmFailed,
}

impl Error {
    fn create_dialog(&self) -> gtk::MessageDialog {
        let title = "Error";
        let message = self.to_string();

        gtk::MessageDialog::builder()
            .message_type(gtk::MessageType::Error)
            .title(title)
            .text(&message)
            .buttons(gtk::ButtonsType::Close)
            .build()
    }

    pub fn error_dialog<P: IsA<gtk::Window>>(&self, parent: Option<&P>) {
        let dialog = self.create_dialog();
        dialog.set_transient_for(parent);
        dialog.run();
        dialog.close();
    }

    pub fn app_error_dialog(&self, app: &gtk::Application) {
        let dialog = self.create_dialog();
        app.add_window(&dialog);
        dialog.run();
        dialog.close();
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Zbus(e) => write!(f, "ZBus error: {}", e),
            Error::ManagerConnect => write!(f, "Unable to connect to Realms Manager"),
            Error::NoSuchRealm(name) => write!(f, "Realm '{}' does not exist", name),
            Error::CreateRealmFailed => write!(f, "Failed to create new realm"),
        }
    }
}

impl From<zbus::Error> for Error {
    fn from(e: zbus::Error) -> Self {
        Zbus(e)
    }
}