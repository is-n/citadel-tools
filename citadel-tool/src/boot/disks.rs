use std::path::{Path, PathBuf};

use libcitadel::{Result, util};

///
/// Represents a disk partition device on the system
///
/// A wrapper around the fields from a line in /proc/partitions
///
#[derive(Debug)]
pub struct DiskPartition {
    path: PathBuf,
    major: u16,
    minor: u16,
    blocks: usize,
}

impl DiskPartition {
    const ESP_GUID: &'static str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";

    /// Return list of all vfat partitions on the system as a `Vec<DiskPartition>`
    pub fn boot_partitions(check_guid: bool) -> Result<Vec<DiskPartition>> {
        let pp = util::read_to_string("/proc/partitions")?;
        let mut v = Vec::new();
        for line in pp.lines().skip(2)
        {
            let part = DiskPartition::from_proc_line(&line)
                .map_err(context!("failed to parse line '{}'", line))?;

            if part.is_boot_partition(check_guid)? {
                v.push(part);
            }
        }
        Ok(v)
    }

    fn is_boot_partition(&self, check_guid: bool) -> Result<bool> {
        let is_boot = if check_guid {
            self.is_vfat()? && self.is_esp_guid()?
        } else {
            self.is_vfat()?
        };
        Ok(is_boot)
    }

    // Parse a single line from /proc/partitions
    //
    // Example line:
    //
    //    8        1     523264 sda1
    //
    fn from_proc_line(line: &str) -> Result<DiskPartition> {
        let v = line.split_whitespace().collect::<Vec<_>>();
        if v.len() != 4 {
            bail!("could not parse");
        }
        Ok(DiskPartition::from_line_components(
            v[0].parse::<u16>()?,    // Major
            v[1].parse::<u16>()?,    // Minor
            v[2].parse::<usize>()?, // number of blocks
            v[3],
        )) // device name
    }

    // create a new `DiskPartion` from parsed components of line from /proc/partitions
    fn from_line_components(major: u16, minor: u16, blocks: usize, name: &str) -> DiskPartition {
        DiskPartition {
            path: PathBuf::from("/dev").join(name),
            major,
            minor,
            blocks,
        }
    }

    // return `true` if partition is VFAT type
    fn is_vfat(&self) -> Result<bool> {
        let ok = self.partition_fstype()? == "vfat";
        Ok(ok)
    }

    fn is_esp_guid(&self) -> Result<bool> {
        let ok = self.partition_guid_type()? == Self::ESP_GUID;
        Ok(ok)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn mount<P: AsRef<Path>>(&self, target: P) -> Result<()> {
        cmd!("/usr/bin/mount", "{} {}", self.path.display(), target.as_ref().display())
    }

    pub fn umount(&self) -> Result<()> {
        cmd!("/usr/bin/umount", "{}", self.path().display())
    }

    fn partition_fstype(&self) -> Result<String> {
        self.lsblk_var("FSTYPE")
    }

    fn partition_guid_type(&self) -> Result<String> {
        self.lsblk_var("PARTTYPE")
    }

    pub fn partition_uuid(&self) -> Result<String> {
        self.lsblk_var("PARTUUID")
    }

    /// Execute lsblk to query for a single output column variable on this partition device
    fn lsblk_var(&self, var: &str) -> Result<String> {
        cmd_with_output!("/usr/bin/lsblk", "-dno {} {}", var, self.path().display())
    }
}
