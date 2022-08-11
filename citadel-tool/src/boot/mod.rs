use std::fs;
use std::process::exit;

use crate::boot::disks::DiskPartition;
use libcitadel::RealmManager;
use libcitadel::{util, CommandLine, KeyRing, LogLevel, Logger, ResourceImage, Result};
use std::path::Path;

mod disks;
mod live;
mod rootfs;

pub fn main(args: Vec<String>) {
    if CommandLine::debug() {
        Logger::set_log_level(LogLevel::Debug);
    } else if CommandLine::verbose() {
        Logger::set_log_level(LogLevel::Info);
    }

    let result = match args.get(1) {
        Some(s) if s == "rootfs" => do_rootfs(),
        Some(s) if s == "setup" => do_setup(),
        Some(s) if s == "boot-automount" => do_boot_automount(),
        Some(s) if s == "start-realms" => do_start_realms(),
        _ => Err(format_err!("Bad or missing argument").into()),
    };

    if let Err(ref e) = result {
        warn!("Failed: {}", e);
        exit(1);
    }
}

fn do_rootfs() -> Result<()> {
    if CommandLine::live_mode() || CommandLine::install_mode() {
        live::live_rootfs()
    } else {
        rootfs::setup_rootfs()
    }
}

fn setup_keyring() -> Result<()> {
    ResourceImage::ensure_storage_mounted()?;
    let keyring = KeyRing::load_with_cryptsetup_passphrase("/sysroot/storage/keyring")?;
    keyring.add_keys_to_kernel()?;
    Ok(())
}

fn do_setup() -> Result<()> {
    if CommandLine::live_mode() || CommandLine::install_mode() {
        live::live_setup()?;
    } else if let Err(err) = setup_keyring() {
        warn!("Failed to setup keyring: {}", err);
    }

    ResourceImage::mount_image_type("kernel")?;
    ResourceImage::mount_image_type("extra")?;

    if CommandLine::overlay() {
        mount_overlay()?;
    }

    Ok(())
}

fn mount_overlay() -> Result<()> {
    info!("Creating rootfs overlay");

    info!("Moving /sysroot mount to /rootfs.ro");
    util::create_dir("/rootfs.ro")?;
    cmd!("/usr/bin/mount", "--make-private /")?;
    cmd!("/usr/bin/mount", "--move /sysroot /rootfs.ro")?;
    info!("Mounting tmpfs on /rootfs.rw");
    util::create_dir("/rootfs.rw")?;
    cmd!(
        "/usr/bin/mount",
        "-t tmpfs -orw,noatime,mode=755 rootfs.rw /rootfs.rw"
    )?;
    info!("Creating /rootfs.rw/work /rootfs.rw/upperdir");
    util::create_dir("/rootfs.rw/upperdir")?;
    util::create_dir("/rootfs.rw/work")?;
    info!("Mounting overlay on /sysroot");
    cmd!("/usr/bin/mount", "-t overlay overlay -olowerdir=/rootfs.ro,upperdir=/rootfs.rw/upperdir,workdir=/rootfs.rw/work /sysroot")?;

    info!("Moving /rootfs.ro and /rootfs.rw to new root");
    util::create_dir("/sysroot/rootfs.ro")?;
    util::create_dir("/sysroot/rootfs.rw")?;
    cmd!("/usr/bin/mount", "--move /rootfs.ro /sysroot/rootfs.ro")?;
    cmd!("/usr/bin/mount", "--move /rootfs.rw /sysroot/rootfs.rw")?;
    Ok(())
}

fn do_start_realms() -> Result<()> {
    let manager = RealmManager::load()?;
    manager.start_boot_realms()
}

// Write automount unit for /boot partition
fn do_boot_automount() -> Result<()> {
    Logger::set_log_level(LogLevel::Info);

    if CommandLine::live_mode() || CommandLine::install_mode() {
        info!("Skipping creation of /boot automount units for live/install mode");
        return Ok(());
    }

    let boot_partition = find_boot_partition()?;
    info!(
        "Creating /boot automount units for boot partition {}",
        boot_partition
    );
    cmd!(
        "/usr/bin/systemd-mount",
        "-A --timeout-idle-sec=300 {} /boot",
        boot_partition
    )
}

fn find_boot_partition() -> Result<String> {
    let loader_dev = read_loader_dev_efi_var()?;
    let boot_partitions = DiskPartition::boot_partitions(true)?
        .into_iter()
        .filter(|part| matches_loader_dev(part, &loader_dev))
        .collect::<Vec<_>>();

    if boot_partitions.len() != 1 {
        return Err(format_err!("Cannot uniquely determine boot partition"));
    }

    Ok(boot_partitions[0].path().display().to_string())
}

// if the 'loader device' EFI variable is set, then dev will contain the UUID
// of the device to match. If it has not been set, then return true to match
// every partition.
fn matches_loader_dev(partition: &DiskPartition, dev: &Option<String>) -> bool {
    if let Some(ref dev) = dev {
        match partition.partition_uuid() {
            Err(err) => {
                warn!("error running lsblk {}", err);
                return true;
            }
            Ok(uuid) => return uuid == dev.as_str(),
        }
    }
    true
}

const LOADER_EFI_VAR_PATH: &str =
    "/sys/firmware/efi/efivars/LoaderDevicePartUUID-4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";

fn read_loader_dev_efi_var() -> Result<Option<String>> {
    let efi_var = Path::new(LOADER_EFI_VAR_PATH);
    if efi_var.exists() {
        let s = fs::read(efi_var)
            .map_err(context!("could not read {:?}", efi_var))?
            .into_iter()
            .skip(4) // u32 'attribute'
            .filter(|b| *b != 0) // string is utf16 ascii
            .map(|b| (b as char).to_ascii_lowercase())
            .collect::<String>();
        Ok(Some(s))
    } else {
        info!("efi path does not exist");
        Ok(None)
    }
}
