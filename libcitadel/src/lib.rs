#[macro_use] extern crate failure;
#[macro_use] extern crate nix;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

#[macro_export]
macro_rules! info {
    ($e:expr) => { if $crate::verbose() { println!("[+] {}", $e);} };
    ($fmt:expr, $($arg:tt)+) => { if $crate::verbose() { println!("[+] {}", format!($fmt, $($arg)+));} };
}
#[macro_export]
macro_rules! warn {
    ($e:expr) => { println!("WARNING: {}", $e); };
    ($fmt:expr, $($arg:tt)+) => { println!("WARNING: {}", format!($fmt, $($arg)+)); };
}

#[macro_export]
macro_rules! notify {
    ($e:expr) => { println!("[+] {}", $e); };
    ($fmt:expr, $($arg:tt)+) => { println!("[+] {}", format!($fmt, $($arg)+)); };
}

use std::cell::RefCell;
use std::result;

use failure::Error;

thread_local! {
    pub static VERBOSE: RefCell<bool> = RefCell::new(false);
}

pub fn verbose() -> bool {
    VERBOSE.with(|f| { *f.borrow() })
}

pub fn set_verbose(val: bool) {
    VERBOSE.with(|f| { *f.borrow_mut() = val });
}

pub fn format_error(err: &Error) -> String {
    let mut output = err.to_string();
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        output.push_str(": ");
        output.push_str(&next.to_string());
        prev = next;
    }
    output
}

mod blockdev;
mod config;
mod keys;
mod cmdline;
mod header;
mod partition;
mod resource;
pub mod util;
pub mod verity;
mod mount;
mod realmfs;
mod keyring;

pub use crate::config::OsRelease;
pub use crate::blockdev::BlockDev;
pub use crate::cmdline::CommandLine;
pub use crate::header::{ImageHeader,MetaInfo};
pub use crate::partition::Partition;
pub use crate::resource::ResourceImage;
pub use crate::keys::{KeyPair,PublicKey};
pub use crate::mount::Mount;
pub use crate::realmfs::RealmFS;
pub use crate::keyring::KeyRing;

const DEVKEYS_HEX: &str = "bc02a3a4fd4a0471a8cb2f96d8be0a0a2d060798c024e60d7a98482f23197fc0";

pub fn devkeys() -> KeyPair {
    KeyPair::from_hex(&DEVKEYS_HEX)
        .expect("Error parsing built in dev channel keys")
}

pub fn public_key_for_channel(channel: &str) -> Result<Option<PublicKey>> {
    if channel == "dev" {
        return Ok(Some(devkeys().public_key()));
    }

    // Look in /etc/os-release
    if Some(channel) == OsRelease::citadel_channel() {
        if let Some(hex) = OsRelease::citadel_image_pubkey() {
            let pubkey = PublicKey::from_hex(hex)?;
            return Ok(Some(pubkey));
        }
    }

    // Does kernel command line have citadel.channel=name:[hex encoded pubkey]
    if Some(channel) == CommandLine::channel_name() {
        if let Some(hex) = CommandLine::channel_pubkey() {
            let pubkey = PublicKey::from_hex(hex)?;
            return Ok(Some(pubkey))
        }
    }

    Ok(None)
}

pub type Result<T> = result::Result<T,Error>;

pub const BLOCK_SIZE: usize = 4096;
