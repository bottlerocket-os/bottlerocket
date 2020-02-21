/*!
This module handles the filesystem and block device interactions necessary to
load, resize, and save a partition table back to disk.
*/

pub(crate) mod error;
use error::Result;

use block_party::BlockDevice;
use gptman::{GPTPartitionEntry, GPT};
use snafu::{ensure, OptionExt, ResultExt};
use std::fs;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub struct DiskPart {
    gpt: GPT,
    device: PathBuf,
}

impl DiskPart {
    /// Given a path to a partition, find the underlying disk and load the GPT label.
    pub(crate) fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let disk = Self::find_disk(path)?;
        let device = disk.path();
        let gpt = Self::load_gpt(&device)?;
        let used_partitions = gpt.iter().filter(|(_num, part)| part.is_used()).count();
        ensure!(
            used_partitions == 1,
            error::MultiplePartitions {
                path,
                count: used_partitions
            }
        );
        Ok(Self { device, gpt })
    }

    /// Grow a single partition to fill the available capacity on the device.
    pub(crate) fn grow(&mut self) -> Result<()> {
        let gpt = &mut self.gpt;
        let part = 1;
        let current = &gpt[part];
        let partition_name = current.partition_name.clone();
        let partition_type_guid = current.partition_type_guid;
        let unique_parition_guid = current.unique_parition_guid;

        let path = &self.device;

        // Remove all existing partitions so that the space shows up as free.
        gpt.remove(part)
            .context(error::RemovePartition { part, path })?;

        // First usable LBA is just after the primary label. We want partitions aligned on 1 MB
        // boundaries, so the first one occurs at 2048 sectors.
        let starting_lba = 2048;

        // Max size gives us the sector count between starting and ending LBA, but doesn't give
        // us the last LBA, which we must solve for next.
        let max_size: u64 = gpt
            .get_maximum_partition_size()
            .context(error::FindMaxSize { path })?;

        // We know the first LBA, and we know the sector count, so we can calculate the last LBA.
        let ending_lba = starting_lba + max_size - 1;

        gpt[part] = GPTPartitionEntry {
            starting_lba,
            ending_lba,
            attribute_bits: 0,
            partition_name,
            partition_type_guid,
            unique_parition_guid,
        };

        Ok(())
    }

    /// Write the GPT label back to the device. If this is a block device, tell
    /// the kernel to reload the partition table.
    pub(crate) fn write(&mut self) -> Result<()> {
        let path = &self.device;

        let mut f = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .context(error::DeviceOpen { path })?;

        self.gpt
            .write_into(&mut f)
            .context(error::WritePartitionTable { path })?;

        let metadata = fs::metadata(path).context(error::DeviceStat { path })?;
        let filetype = metadata.st_mode() & libc::S_IFMT;
        if filetype == libc::S_IFBLK {
            gptman::linux::reread_partition_table(&mut f)
                .context(error::ReloadPartitionTable { path })?;
        }

        Ok(())
    }

    /// Find the block device that holds the specified partition.
    fn find_disk<P>(path: P) -> Result<BlockDevice>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let partition_path = fs::canonicalize(path).context(error::CanonicalizeLink { path })?;

        let partition_device =
            BlockDevice::from_device_node(&partition_path).context(error::FindBlockDevice {
                path: &partition_path,
            })?;

        let disk = partition_device
            .disk()
            .context(error::FindDisk {
                path: &partition_path,
            })?
            .context(error::NotPartition {
                path: &partition_path,
            })?;

        Ok(disk)
    }

    /// Load the GPT disk label from the device.
    fn load_gpt<P>(path: P) -> Result<GPT>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mut f = fs::File::open(path).context(error::DeviceOpen { path })?;
        let gpt = GPT::find_from(&mut f).context(error::ReadPartitionTable { path })?;
        Ok(gpt)
    }
}
