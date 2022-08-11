use crate::rowdata::RowData;
use crate::zbus_client;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use std::error;

use futures_util::stream::StreamExt;

const WELCOME_UI: &str = include_str!("../data/welcome_page.ui");
const CITADEL_PASSWORD_UI: &str = include_str!("../data/citadel_password_page.ui");
const LUKS_PASSWORD_UI: &str = include_str!("../data/luks_password_page.ui");
const INSTALL_DESTINATION_UI: &str = include_str!("../data/install_destination_page.ui");
const CONFIRM_INSTALL_UI: &str = include_str!("../data/confirm_install_page.ui");
const INSTALL_UI: &str = include_str!("../data/install_page.ui");

#[derive(Clone)]
pub struct Ui {
    pub assistant: gtk::Assistant,
    pub disks_listbox: gtk::ListBox,
    pub disks_model: gio::ListStore,
    pub disk_rows: Vec<RowData>,
    pub citadel_password_page: gtk::Box,
    pub citadel_password_entry: gtk::Entry,
    pub citadel_password_confirm_entry: gtk::Entry,
    pub citadel_password_status_label: gtk::Label,
    pub luks_password_page: gtk::Box,
    pub luks_password_entry: gtk::Entry,
    pub luks_password_confirm_entry: gtk::Entry,
    pub luks_password_status_label: gtk::Label,
    pub confirm_install_label: gtk::Label,
    pub install_page: gtk::Box,
    pub install_progress: gtk::ProgressBar,
    pub install_scrolled_window: gtk::ScrolledWindow,
    pub install_textview: gtk::TextView,
    application: gtk::Application,
}

impl Ui {
    pub fn build(application: &gtk::Application) -> Result<Self, Box<dyn error::Error>> {
        let assistant = gtk::Assistant::new();
        assistant.set_default_size(800, 600);

        assistant.set_application(Some(application));
        // assistant.connect_delete_event(glib::clone!(@strong application => move |_, _| {
        //    application.quit();
        //    gtk::Inhibit(false)
        // }));
        assistant.connect_cancel(glib::clone!(@strong application => move |_| {
            application.quit();
        }));

        let welcome_builder = gtk::Builder::from_string(WELCOME_UI);
        let welcome_page: gtk::Box = welcome_builder
            .object("welcome_page")
            .expect("Failed to create welcome_page");

        // show disks page
        let install_destination_builder = gtk::Builder::from_string(INSTALL_DESTINATION_UI);
        let install_destination_page: gtk::Box = install_destination_builder
            .object("install_destination_page")
            .expect("Failed to create install_destination_page");
        let disks_listbox: gtk::ListBox = install_destination_builder
            .object("install_destination_listbox")
            .expect("Failed to create disks_listbox");

        disks_listbox.set_margin_start(15);
        disks_listbox.set_margin_end(15);

        let disks_model = gio::ListStore::new(RowData::static_type());
        disks_listbox.bind_model(Some(&disks_model), move |item| {
            let row = gtk::ListBoxRow::new();
            let item = item
                .downcast_ref::<RowData>()
                .expect("Row data is of wrong type");
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            hbox.set_homogeneous(true);
            let removable = item.property::<bool>("removable");
            let icon_name = Self::get_disk_icon(removable);
            let disk_icon = gtk::Image::from_icon_name(&icon_name);
            disk_icon.set_halign(gtk::Align::Start);
            disk_icon.set_margin_start(20);

            let model_label = gtk::Label::new(None);
            model_label.set_halign(gtk::Align::Start);
            model_label.set_justify(gtk::Justification::Left);
            item.bind_property("model", &model_label, "label")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();
            let path_label = gtk::Label::new(None);
            path_label.set_halign(gtk::Align::Start);
            path_label.set_justify(gtk::Justification::Left);
            item.bind_property("path", &path_label, "label")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();
            let size_label = gtk::Label::new(None);
            size_label.set_halign(gtk::Align::End);
            size_label.set_justify(gtk::Justification::Right);
            item.bind_property("size", &size_label, "label")
                .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                .build();
            size_label.set_margin_end(20);

            hbox.append(&disk_icon);
            hbox.append(&path_label);
            hbox.append(&model_label);
            hbox.append(&size_label);

            row.set_child(Some(&hbox));

            row.upcast::<gtk::Widget>()
        });

        disks_listbox.connect_row_selected(glib::clone!(@strong assistant, @strong install_destination_page => move |_, listbox_row | {
                if let Some(_) = listbox_row {
                    assistant.set_page_complete(&install_destination_page, true);
                }
            }));

        // show citadel user password page
        let citadel_password_builder = gtk::Builder::from_string(CITADEL_PASSWORD_UI);
        let citadel_password_page: gtk::Box = citadel_password_builder
            .object("citadel_password_page")
            .expect("Failed to create citadel_password_page");
        let citadel_password_entry: gtk::Entry = citadel_password_builder
            .object("citadel_password_entry")
            .expect("Failed to create citadel_password_entry");
        let citadel_password_confirm_entry: gtk::Entry = citadel_password_builder
            .object("citadel_password_confirm_entry")
            .expect("Failed to create citadel_password_confirm_entry");
        let citadel_password_status_label: gtk::Label = citadel_password_builder
            .object("citadel_password_status_label")
            .expect("Failed to create citadel_password_status_label");

        // show hdd encryption password page
        let luks_password_builder = gtk::Builder::from_string(LUKS_PASSWORD_UI);
        let luks_password_page: gtk::Box = luks_password_builder
            .object("luks_password_page")
            .expect("Failed to create luks_password_page");
        let luks_password_entry: gtk::Entry = luks_password_builder
            .object("luks_password_entry")
            .expect("Failed to create luks_password_entry");
        let luks_password_confirm_entry: gtk::Entry = luks_password_builder
            .object("luks_password_confirm_entry")
            .expect("Failed to create luks_password_confirm_entry");
        let luks_password_status_label: gtk::Label = luks_password_builder
            .object("luks_password_status_label")
            .expect("Failed to create luks_password_status_label");

        // show install confirmation page
        let confirm_install_builder = gtk::Builder::from_string(CONFIRM_INSTALL_UI);
        let confirm_install_page: gtk::Box = confirm_install_builder
            .object("confirm_install_page")
            .expect("Failed to create confirm_install_page");
        let confirm_install_label: gtk::Label = confirm_install_builder
            .object("confirm_install_label_3")
            .expect("Failed to create confirm_install_label_3");

        let install_builder = gtk::Builder::from_string(INSTALL_UI);
        let install_page: gtk::Box = install_builder
            .object("install_page")
            .expect("Failed to create install_page");
        let install_progress: gtk::ProgressBar = install_builder
            .object("install_progress")
            .expect("Failed to get install_progress");
        let install_scrolled_window: gtk::ScrolledWindow = install_builder
            .object("install_scrolled_window")
            .expect("Failed to get install_scrolled_window");
        let install_textview: gtk::TextView = install_builder
            .object("install_textview")
            .expect("Failed to get install_textview");

        assistant.append_page(&welcome_page);
        assistant.set_page_type(&welcome_page, gtk::AssistantPageType::Intro);
        assistant.set_page_complete(&welcome_page, true);
        assistant.append_page(&install_destination_page);
        assistant.append_page(&citadel_password_page);
        assistant.append_page(&luks_password_page);
        assistant.append_page(&confirm_install_page);
        assistant.set_page_type(&confirm_install_page, gtk::AssistantPageType::Confirm);
        assistant.set_page_complete(&confirm_install_page, true);
        assistant.append_page(&install_page);
        assistant.set_page_type(&install_page, gtk::AssistantPageType::Progress);

        let disks_model_clone = disks_model.clone();

        let mut ui = Self {
            assistant,
            citadel_password_page,
            citadel_password_entry,
            citadel_password_confirm_entry,
            citadel_password_status_label,
            luks_password_page,
            luks_password_entry,
            luks_password_confirm_entry,
            luks_password_status_label,
            disks_listbox,
            disks_model,
            disk_rows: vec![],
            confirm_install_label,
            install_page,
            install_progress,
            install_scrolled_window,
            install_textview,
            application: application.to_owned(),
        };

        let disks = ui.get_disks().expect("Failed to get disks");

        ui.disk_rows = disks.clone();

        ui.setup_style(); // we use default styles for the moment
        ui.setup_signals();
        for disk in disks {
            disks_model_clone.append(&disk);
        }
        Ok(ui)
    }

    fn get_disks(&self) -> Result<Vec<RowData>, Box<dyn error::Error>> {
        let conn = zbus::blocking::Connection::system().unwrap();
        let manager = zbus_client::ManagerProxyBlocking::new(&conn).unwrap();
        let devices = manager.get_disks().unwrap();

        let mut disks: Vec<RowData> = Vec::new();

        for device in devices {
            log::debug!("The following disk was dicovered: {:?}", device);
            let disk = RowData::new(
                &device.1[0].clone(),
                &device.0,
                &device.1[1].clone(),
                device.1[2].parse().unwrap(),
            );
            disks.push(disk);
        }
        log::debug!("The following disks were dicovered: {:?}", disks);

        Ok(disks)
    }

    fn get_disk_icon(removable: bool) -> String {
        if removable {
            return "drive-harddisk-usb-symbolic".to_string();
        }
        "drive-harddisk-system-symbolic".to_string()
    }

    pub fn setup_entry_signals(
        &self,
        page: &gtk::Box,
        first_entry: &gtk::Entry,
        second_entry: &gtk::Entry,
        status_label: &gtk::Label,
    ) {
        let ui = self.clone();
        let assistant = ui.assistant.clone();
        first_entry.connect_changed(glib::clone!(@weak assistant, @weak page, @weak second_entry, @weak status_label => move |entry| {
            let password = entry.text();
            let confirm = second_entry.text();
            if password != "" && confirm != "" {
                let matches = password == confirm;
                if !matches {
                    status_label.set_text("Passwords do not match");
                } else {
                    status_label.set_text("");
                }
                assistant.set_page_complete(&page, matches);
            }
        }));
        first_entry.connect_activate(glib::clone!(@weak second_entry => move |_| {
            second_entry.grab_focus();
        }));
    }

    pub fn setup_prepare_signal(&self) {
        let ui = self.clone();
        ui.assistant
            .connect_prepare(glib::clone!(@strong ui => move |assistant, page| {
                let page_type = assistant.page_type(page);
                if page_type == gtk::AssistantPageType::Confirm {
                    let path = ui.get_install_destination();
                    let text = format!("<i>{}</i>", path);
                    ui.confirm_install_label.set_markup(&text);
                }
            }));
    }

    pub fn setup_apply_signal(&self) {
        let ui = self.clone();
        ui.assistant
            .connect_apply(glib::clone!(@strong ui => move |_| {
                let conn = zbus::blocking::Connection::system().unwrap();
                let manager = zbus_client::ManagerProxyBlocking::new(&conn).unwrap();
                let result = manager.run_install(&ui.get_install_destination(), &ui.get_citadel_password(), &ui.get_luks_password());

                log::debug!("Started the install with message: {:?}", result);
            }));
    }

    fn _setup_autoscroll_signal(&self) {
        // let ui = self.clone();
        // let scrolled_window = ui.install_scrolled_window;
        // ui.install_textview
        //     .connect_size_allocate(glib::clone!(@weak scrolled_window => move |_| {
        //         let adjustment = scrolled_window.vadjustment();
        //         adjustment.set_value(adjustment.upper() - adjustment.page_size());
        //     }));
    }

    pub fn setup_signals(&self) {
        let ui = self.clone();
        self.setup_entry_signals(
            &ui.citadel_password_page,
            &ui.citadel_password_entry,
            &ui.citadel_password_confirm_entry,
            &ui.citadel_password_status_label,
        );
        self.setup_entry_signals(
            &ui.citadel_password_page,
            &ui.citadel_password_confirm_entry,
            &ui.citadel_password_entry,
            &ui.citadel_password_status_label,
        );
        self.setup_entry_signals(
            &ui.luks_password_page,
            &ui.luks_password_entry,
            &ui.luks_password_confirm_entry,
            &ui.luks_password_status_label,
        );
        self.setup_entry_signals(
            &ui.luks_password_page,
            &ui.luks_password_confirm_entry,
            &ui.luks_password_entry,
            &ui.luks_password_status_label,
        );
        self.setup_prepare_signal();
        self.setup_apply_signal();
        //self.setup_autoscroll_signal();
    }

    fn setup_style(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_bytes!("../data/style.css"));

        gtk::StyleContext::add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    pub fn get_citadel_password(&self) -> String {
        let ui = self.clone();
        let password = ui.citadel_password_entry.text();
        password.to_string()
    }

    pub fn get_luks_password(&self) -> String {
        let ui = self.clone();
        let password = ui.luks_password_entry.text();
        password.to_string()
    }

    pub fn get_install_destination(&self) -> String {
        let ui = self.clone();
        if let Some(row) = ui.disks_listbox.selected_row() {
            let index = row.index() as usize;
            if ui.disk_rows.len() > index {
                let data = &ui.disk_rows[index];
                let path: String = data.property::<String>("path");
                return path.to_string();
            }
        }
        String::new()
    }

    /// This function takes care of setting up all of the signals the that the UI is listening to.
    /// Thse signals are only used to display the progress of the install after the last user confirmation is clicked
    async fn setup_signal_matchers(&self) -> zbus::Result<()> {
        //    fn setup_signal_matchers(&self) {
        let conn = zbus::Connection::system().await.unwrap();
        let manager = zbus_client::ManagerProxy::new(&conn).await.unwrap();

        let mut install_install_started_stream = manager.receive_install_started().await?;
        let mut partion_disk_stream = manager.receive_partition_disk().await?;
        let mut luks_setup_stream = manager.receive_luks_setup().await?;
        let mut lvm_setup_stream = manager.receive_lvm_setup().await?;
        let mut boot_setup_stream = manager.receive_boot_setup().await?;
        let mut storage_created_stream = manager.receive_storage_created().await?;
        let mut rootfs_installed_stream = manager.receive_rootfs_installed().await?;

        let mut install_completed_stream = manager.receive_install_completed().await?;
        let mut install_failed_stream = manager.receive_install_failed().await?;

        futures_util::try_join!(
            async {
                // install started
                while let Some(_) = install_install_started_stream.next().await {
                    let text = format!(
                            "Installing Citadel to {}. \nFor a full log, consult the systemd journal by running the following command:\n sudo journalctl -u citadel-installer-backend.service\n",
                            self.get_install_destination());

                    self.process_signal_reception("Msg::InstallStarted", &text, 1.0)?;
                }
                Ok::<(), zbus::Error>(())
            },
            async {
                // partition disk
                while let Some(signal) = partion_disk_stream.next().await {
                    let progress_message = format!("{:?}", signal.args()?);

                    self.process_signal_reception("Msg::PartitionDisk", &progress_message, 2.0)?;
                }
                Ok(())
            },
            async {
                // luks setup
                while let Some(signal) = luks_setup_stream.next().await {
                    let progress_message = format!("{:?}", signal.args()?);

                    self.process_signal_reception("Msg::LuksSetup", &progress_message, 3.0)?;
                }
                Ok(())
            },
            async {
                // lvm setup
                while let Some(signal) = lvm_setup_stream.next().await {
                    let progress_message = format!("{:?}", signal.args()?);

                    self.process_signal_reception("Msg::LvmSetup", &progress_message, 4.0)?;
                }
                Ok(())
            },
            async {
                // boot setup
                while let Some(signal) = boot_setup_stream.next().await {
                    let progress_message = format!("{:?}", signal.args()?);

                    self.process_signal_reception("Msg::BootSetup", &progress_message, 5.0)?;
                }
                Ok(())
            },
            async {
                // storage created
                while let Some(signal) = storage_created_stream.next().await {
                    let progress_message = format!("{:?}", signal.args()?);

                    self.process_signal_reception("Msg::StorageCreated", &progress_message, 6.0)?;
                }
                Ok(())
            },
            async {
                // rootfs installed
                while let Some(signal) = rootfs_installed_stream.next().await {
                    let progress_message = format!("{:?}", signal.args()?);

                    self.process_signal_reception("Msg::RootfsInstalled", &progress_message, 7.0)?;
                }
                Ok(())
            },
            async {
                // install completed
                while let Some(_) = install_completed_stream.next().await {
                    let progress_message = "Completed the installation successfully\n";

                    self.process_signal_reception(
                        "Msg::InstallCompleted",
                        &progress_message,
                        10.0,
                    )?;
                    self.add_quit_button_upon_end_of_install();
                }
                Ok(())
            },
            async {
                // install failed
                while let Some(signal) = install_failed_stream.next().await {
                    let progress_message = signal.args()?;

                    self.install_progress.set_fraction(1.0);
                    let buffer = self.install_textview.buffer();
                    let mut iter = self.install_textview.buffer().end_iter();
                    let text = format!(
                        "+ Install failed with error:\n<i>{:?}</i>\n",
                        progress_message
                    );
                    buffer.insert_markup(&mut iter, &text);

                    self.add_quit_button_upon_end_of_install();

                    log::debug!(
                        "Signal matchers log message at install failed: {:?}",
                        progress_message
                    );
                }
                Ok(())
            },
        )?;

        Ok::<(), zbus::Error>(())
    }

    /// Helper function to print the message received from the signal to the user at install_page
    fn process_signal_reception(
        &self,
        step: &str,
        progress_message: &str,
        index: f64,
    ) -> zbus::Result<()> {
        if index > 7.0 {
            self.install_progress.set_fraction(1.0);
        } else {
            self.install_progress.set_fraction(0.125 * index);
        }
        let buffer = self.install_textview.buffer();
        let mut iter = buffer.end_iter();

        buffer.insert(&mut iter, &progress_message);

        log::debug!(
            "Signal matchers log message at {} {:?}",
            step,
            progress_message
        );
        Ok(())
    }

    fn add_quit_button_upon_end_of_install(&self) {
        let quit_button = gtk::Button::with_label("Quit");
        let appli = &self.application;

        quit_button.connect_clicked(glib::clone!(@strong appli => move |_| {
            appli.quit();
        }));

        quit_button.set_sensitive(true);
        self.assistant.add_action_widget(&quit_button);
    }

    pub async fn start(&self) {
        // let result = manager
        //     .run_install(
        //         &self.get_install_destination(),
        //         &self.citadel_password_entry.text(),
        //         &self.luks_password_entry.text(),
        //     )
        //     .unwrap();

        let _ = self.setup_signal_matchers().await;
        //self.setup_signal_matchers();

        // thread::spawn(move || loop {
        //     conn.process(Duration::from_millis(1000)).unwrap();
        // });
    }
}
