use std::fs;
use std::path::{Path, PathBuf};

use libcitadel::{util, Result};

#[derive(Debug, Clone)]
pub struct Disk {
    path: PathBuf,
    size: usize,
    size_str: String,
    model: String,
}

impl Disk {
    pub fn probe_all() -> Result<Vec<Disk>> {
        let mut v = Vec::new();
        util::read_directory("/sys/block", |dent| {
            let path = dent.path();
            if Disk::is_disk_device(&path) {
                let disk = Disk::read_device(&path)?;
                v.push(disk);
            }
            Ok(())
        })?;

        Ok(v)
    }

    fn is_disk_device(device: &Path) -> bool {
        device.join("device/model").exists()
    }

    fn read_device(device: &Path) -> Result<Disk> {
        let path = Path::new("/dev/").join(device.file_name().unwrap());

        let size = fs::read_to_string(device.join("size"))
            .map_err(context!("failed to read device size for {:?}", device))?
            .trim()
            .parse::<usize>()
            .map_err(context!("error parsing device size for {:?}", device))?;

        let size_str = format!("{}G", size >> 21);

        let model = fs::read_to_string(device.join("device/model"))
            .map_err(context!("failed to read device/model for {:?}", device))?
            .trim()
            .to_string();

        Ok(Disk {
            path,
            size,
            size_str,
            model,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size_str(&self) -> &str {
        &self.size_str
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}
