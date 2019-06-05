mod block;

use crate::error::{self, Error};
use crate::gptprio::GptPrio;
use crate::guid::uuid_to_guid;
use crate::set::{PartitionSet, SetSelect};
use crate::state::block::BlockDevice;
use gptman::GPT;
use hex_literal::hex;
use snafu::{OptionExt, ResultExt};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

const THAR_BOOT: [u8; 16] = uuid_to_guid(hex!("6b636168 7420 6568 2070 6c616e657421"));
const THAR_ROOT: [u8; 16] = uuid_to_guid(hex!("5526016a 1a97 4ea4 b39a b7c8c6ca4502"));
const THAR_HASH: [u8; 16] = uuid_to_guid(hex!("598f10af c955 4456 6a99 7720068a6cea"));

#[derive(Debug, Clone)]
pub(crate) struct State {
    os_disk: PathBuf,
    sets: [PartitionSet; 2],
    boot_nums: [u32; 2],
    table: GPT,
    active: SetSelect,
}

impl State {
    pub(crate) fn load() -> Result<Self, Error> {
        // The root filesystem is a dm-verity device. We want to determine what disk and partition
        // the backing data is part of. Look up the device major and minor via stat(2):
        let root_fs = BlockDevice::from_resident("/")?;
        // Get the first lower device from this one, and determine what disk it belongs to.
        let active_partition =
            root_fs
                .lower_devices()
                .next()
                .context(error::RootHasNoLowerDevices {
                    root_major_minor: root_fs.to_string(),
                })??;
        let os_disk = active_partition.disk()?;
        let active_partition = active_partition.path();

        // Parse the partition table on the disk and detect the partition sets.
        let table = GPT::find_from(&mut File::open(os_disk.path()).context(error::Open {
            path: os_disk.path(),
            what: "reading",
        })?)
        .map_err(error::GPTError)
        .context(error::GPTFind {
            device: os_disk.path(),
        })?;

        let nth_guid = |guid, n| -> Result<u32, Error> {
            Ok(table
                .iter()
                .filter(|(_, p)| p.partition_type_guid == guid)
                .nth(n)
                .context(error::PartitionMissingFromSet {
                    part_type: stringify!(guid),
                    set: if n == 0 { "A" } else { "B" },
                })?
                .0)
        };
        let disk = |num| -> Result<PathBuf, Error> {
            Ok(os_disk
                .partition(num)?
                .context(error::PartitionNotFoundOnDevice {
                    num,
                    device: os_disk.path(),
                })?
                .path())
        };

        let boot_nums = [nth_guid(THAR_BOOT, 0)?, nth_guid(THAR_BOOT, 1)?];
        let sets = [
            PartitionSet {
                boot: disk(boot_nums[0])?,
                root: disk(nth_guid(THAR_ROOT, 0)?)?,
                hash: disk(nth_guid(THAR_HASH, 0)?)?,
            },
            PartitionSet {
                boot: disk(boot_nums[1])?,
                root: disk(nth_guid(THAR_ROOT, 1)?)?,
                hash: disk(nth_guid(THAR_HASH, 1)?)?,
            },
        ];

        let active = if sets[0].contains(&active_partition) {
            SetSelect::A
        } else if sets[1].contains(&active_partition) {
            SetSelect::B
        } else {
            return error::ActiveNotInSet {
                active_partition,
                sets,
            }
            .fail();
        };

        Ok(Self {
            os_disk: os_disk.path(),
            sets,
            boot_nums,
            table,
            active,
        })
    }

    pub(crate) fn os_disk(&self) -> &Path {
        &self.os_disk
    }

    pub(crate) fn set(&self, select: SetSelect) -> &PartitionSet {
        &self.sets[select.idx()]
    }

    fn gptprio(&self, select: SetSelect) -> GptPrio {
        GptPrio::from(self.table[self.boot_nums[select.idx()]].attribute_bits)
    }

    fn set_gptprio(&mut self, select: SetSelect, flags: GptPrio) {
        self.table[self.boot_nums[select.idx()]].attribute_bits = flags.into();
    }

    pub(crate) fn active(&self) -> SetSelect {
        self.active
    }

    pub(crate) fn inactive(&self) -> SetSelect {
        // resolve opposing set member
        !self.active
    }

    pub(crate) fn next(&self) -> Option<SetSelect> {
        let gptprio_a = self.gptprio(SetSelect::A);
        let gptprio_b = self.gptprio(SetSelect::B);
        match (gptprio_a.will_boot(), gptprio_b.will_boot()) {
            (true, true) => {
                if gptprio_a.priority() >= gptprio_b.priority() {
                    Some(SetSelect::A)
                } else {
                    Some(SetSelect::B)
                }
            }
            (true, false) => Some(SetSelect::A),
            (false, true) => Some(SetSelect::B),
            (false, false) => None,
        }
    }

    /// Sets the active partition as successfully booted, but **does not write to the disk**.
    pub(crate) fn mark_successful_boot(&mut self) {
        let mut flags = self.gptprio(self.active());
        flags.set_successful(true);
        self.set_gptprio(self.active(), flags);
    }

    /// Clears priority bits of the inactive partition in preparation to write new images, but
    /// **does not write to the disk**.
    pub(crate) fn clear_inactive(&mut self) {
        let mut inactive_flags = self.gptprio(self.inactive());
        inactive_flags.set_priority(0);
        inactive_flags.set_tries_left(0);
        inactive_flags.set_successful(false);
        self.set_gptprio(self.inactive(), inactive_flags);
    }

    /// Sets the inactive partition as a new upgrade partition, but **does not write to the disk**.
    ///
    /// * Sets the inactive partition's priority to 2 and the active partition's priority to 1.
    /// * Sets the inactive partition's tries left to 1.
    /// * Sets the inactive partition as not successfully booted.
    pub(crate) fn upgrade_to_inactive(&mut self) {
        let mut inactive_flags = self.gptprio(self.inactive());
        inactive_flags.set_priority(2);
        inactive_flags.set_tries_left(1);
        inactive_flags.set_successful(false);
        self.set_gptprio(self.inactive(), inactive_flags);

        let mut active_flags = self.gptprio(self.active());
        active_flags.set_priority(1);
        self.set_gptprio(self.active(), active_flags);
    }

    /// Deprioritizes the active partition, but **does not write to the disk**.
    ///
    /// * Sets the inactive partition's priority to 2 and the active partition's priority to 1.
    /// * Does not modify the inactive partition's tries left.
    /// * Does not modify whether the inactive partition successfully booted.
    pub(crate) fn rollback_to_inactive(&mut self) -> Result<(), Error> {
        let mut inactive_flags = self.gptprio(self.inactive());
        if inactive_flags.priority() == 0 {
            return error::InactiveInvalidRollback {
                flags: inactive_flags,
            }
            .fail();
        }
        inactive_flags.set_priority(2);
        self.set_gptprio(self.inactive(), inactive_flags);

        let mut active_flags = self.gptprio(self.active());
        active_flags.set_priority(1);
        self.set_gptprio(self.active(), active_flags);

        Ok(())
    }

    /// Writes the partition table to the OS disk.
    pub(crate) fn write(&mut self) -> Result<(), Error> {
        self.table
            .write_into(
                &mut OpenOptions::new()
                    .write(true)
                    .open(self.os_disk())
                    .context(error::Open {
                        path: &self.os_disk,
                        what: "writing",
                    })?,
            )
            .map_err(error::GPTError)
            .context(error::GPTWrite {
                device: &self.os_disk,
            })?;
        Ok(())
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "OS disk: {}", self.os_disk.display())?;
        writeln!(
            f,
            "Set A:   {} {}",
            self.set(SetSelect::A),
            self.gptprio(SetSelect::A)
        )?;
        writeln!(
            f,
            "Set B:   {} {}",
            self.set(SetSelect::B),
            self.gptprio(SetSelect::B)
        )?;
        writeln!(f, "Active:  Set {}", self.active())?;
        match self.next() {
            Some(next) => write!(f, "Next:    Set {}", next),
            None => write!(f, "Next:    None"),
        }
    }
}
