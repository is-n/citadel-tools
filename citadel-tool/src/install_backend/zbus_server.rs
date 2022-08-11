use crate::install::installer;
use crate::install_backend::disk;

use zbus::{dbus_interface, Result, SignalContext};

// #[derive(DBusError, Debug)]
// #[dbus_error(prefix = "com.subgraph.installer.Manager")]
// enum ZBusServerError {
//     #[dbus_error(zbus_error)]
//     ZBus(zbus::Error),
//     FailedToInstall(String),
// }

pub struct ServerManager {
    pub done: event_listener::Event,
}

#[dbus_interface(name = "com.subgraph.installer.Manager")]
impl ServerManager {
    fn get_disks(&self) -> std::collections::HashMap<String, Vec<String>> {
        // TODO: improve error handling
        log::debug!("The dbus server ran command \"do_get_disks\"");

        let disks = disk::Disk::probe_all().unwrap();

        let mut disk_map = std::collections::HashMap::new();
        for d in disks {
            let mut fields = vec![];
            fields.push(d.model().to_string());
            fields.push(d.size_str().to_string());
            fields.push(d.removable().to_string());
            disk_map.insert(d.path().to_string_lossy().to_string(), fields);
        }
        disk_map
    }

    /// Until we fix this function with better error handling, it returns true on success and false otherwise
    pub async fn run_install(
        &self,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        path: String,
        citadel_passphrase: String,
        luks_passphrase: String,
    ) -> bool {
        let failed_install_step = |msg: &str| -> bool {
            log::error!("Zbus server failed to run at {}", msg);
            let msg = format!(
                " Install failed with installation daemon stoping at: {}",
                msg
            );
            let future = Self::install_failed(&ctxt, &msg);
            let _ = futures::executor::block_on(future);

            false
        };

        // TODO: improve error handling

        let mut install = installer::Installer::new(path, &citadel_passphrase, &luks_passphrase);
        let _ = Self::install_started(&ctxt, "Successfully started installer").await;

        install.set_install_syslinux(true);

        let _ = install.verify();
        if install.partition_disk().is_err() {
            return failed_install_step("install.partition_disk()");
        }
        let _ = Self::disk_partitioned(&ctxt, "Successfully partitioned disk").await;

        if install.setup_luks().is_err() {
            return failed_install_step("install.setup_luks()");
        }
        let _ = Self::luks_setup(&ctxt, "Successfully setup LUKS").await;

        if install.setup_lvm().is_err() {
            return failed_install_step("install.setup_lvm()");
        }
        let _ = Self::lvm_setup(&ctxt, "Successfully setup LVM").await;

        if install.setup_boot().is_err() {
            return failed_install_step("install.setup_boot()");
        }
        let _ = Self::boot_setup(&ctxt, "Successfully setup boot").await;

        if install.create_storage().is_err() {
            return failed_install_step("install.create_storage()");
        }
        let _ = Self::storage_created(&ctxt, "Successfully created storage").await;

        if install.install_rootfs_partitions().is_err() {
            return failed_install_step("install.install_rootfs_partitions()");
        }
        let _ = Self::rootfs_installed(&ctxt, "Successfully installed rootfs").await;

        if install.finish_install().is_err() {
            return failed_install_step("install.finish_install()");
        }
        let _ = Self::install_completed(&ctxt).await;

        true
    }

    #[dbus_interface(signal)]
    async fn install_started(ctxt: &SignalContext<'_>, progress_message: &str) -> Result<()>;

    #[dbus_interface(signal)]
    async fn disk_partitioned(ctxt: &SignalContext<'_>, progress_message: &str) -> Result<()>;

    #[dbus_interface(signal)]
    async fn lvm_setup(ctxt: &SignalContext<'_>, progress_message: &str) -> Result<()>;

    #[dbus_interface(signal)]
    async fn luks_setup(ctxt: &SignalContext<'_>, progress_message: &str) -> Result<()>;

    #[dbus_interface(signal)]
    async fn boot_setup(ctxt: &SignalContext<'_>, progress_message: &str) -> Result<()>;

    #[dbus_interface(signal)]
    async fn storage_created(ctxt: &SignalContext<'_>, progress_message: &str) -> Result<()>;

    #[dbus_interface(signal)]
    async fn rootfs_installed(ctxt: &SignalContext<'_>, progress_message: &str) -> Result<()>;

    #[dbus_interface(signal)]
    async fn install_completed(ctxt: &SignalContext<'_>) -> Result<()>;

    #[dbus_interface(signal)]
    async fn install_failed(ctxt: &SignalContext<'_>, error_text: &str) -> Result<()>;
}
