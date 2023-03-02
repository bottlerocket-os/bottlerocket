use crate::error::{self, Error};
use crate::gptprio::GptPrio;
use crate::guid::uuid_to_guid;
use crate::set::{PartitionSet, SetSelect};
use block_party::BlockDevice;
use gptman::GPT;
use hex_literal::hex;
use snafu::{ensure, OptionExt, ResultExt};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

const BOTTLEROCKET_BOOT: [u8; 16] = uuid_to_guid(hex!("6b636168 7420 6568 2070 6c616e657421"));
const BOTTLEROCKET_ROOT: [u8; 16] = uuid_to_guid(hex!("5526016a 1a97 4ea4 b39a b7c8c6ca4502"));
const BOTTLEROCKET_HASH: [u8; 16] = uuid_to_guid(hex!("598f10af c955 4456 6a99 7720068a6cea"));
const BOTTLEROCKET_PRIVATE: [u8; 16] = uuid_to_guid(hex!("440408bb eb0b 4328 a6e5 a29038fad706"));

#[derive(Debug, Clone)]
pub struct State {
    os_disk: PathBuf,
    // BOTTLEROCKET_PRIVATE partition number
    private_partition_num: u32,
    sets: [PartitionSet; 2],
    /// The partition numbers that correspond to the boot partitions in each partition set,
    /// respectively.
    ///
    /// This is used to load the correct partition flags from `table`.
    boot_partition_nums: [u32; 2],
    table: GPT,
    active: SetSelect,
}

impl State {
    /// Finds the partition sets available on disk, and determines which is active under the root
    /// filesystem.
    ///
    /// * Finds the device corresponding to the root filesystem mount (`/`), which is assumed to be
    ///   a dm-verity device.
    /// * Gets the first lower device, which will either be the root or hash partition of the
    ///   active partition set.
    /// * Find the first and second partitions matching each of the boot, root, and hash partition
    ///   type GUIDs. The first partitions are set A and the second partitions are set B.
    /// * Determine which partition set is active by finding which one contains the partition we
    ///   found from our root filesystem earlier.
    pub fn load() -> Result<Self, Error> {
        // The root filesystem is a dm-verity device. We want to determine what disk and partition
        // the backing data is part of. Look up the device major and minor via stat(2):
        let root_fs = BlockDevice::from_device_path("/")
            .context(error::BlockDeviceFromPathSnafu { device: "/" })?;
        // Get the first lower device from this one, and determine what disk it belongs to.
        let active_partition = root_fs
            .lower_devices()
            .and_then(|mut iter| iter.next().transpose())
            .context(error::RootLowerDevicesSnafu {
                root: root_fs.path(),
            })?
            .context(error::RootHasNoLowerDevicesSnafu {
                root: root_fs.path(),
            })?;
        let os_disk = active_partition
            .disk()
            .context(error::DiskFromPartitionSnafu {
                device: root_fs.path(),
            })?
            .context(error::RootNotPartitionSnafu {
                device: root_fs.path(),
            })?;
        let active_partition = active_partition.path();

        // Parse the partition table on the disk.
        let table = GPT::find_from(&mut File::open(os_disk.path()).context(error::OpenSnafu {
            path: os_disk.path(),
            what: "reading",
        })?)
        .map_err(error::GPTError)
        .context(error::GPTFindSnafu {
            device: os_disk.path(),
        })?;

        // Finds the nth partition on `table` matching the partition type GUID `guid`.
        let nth_guid = |guid, n| -> Result<u32, Error> {
            Ok(table
                .iter()
                .filter(|(_, p)| p.partition_type_guid == guid)
                .nth(n)
                .context(error::PartitionMissingFromSetSnafu {
                    part_type: stringify!(guid),
                    set: if n == 0 { "A" } else { "B" },
                })?
                .0)
        };
        // Loads the path to partition number `num` on the OS disk.
        let device_from_part_num = |num| -> Result<PathBuf, Error> {
            Ok(os_disk
                .partition(num)
                .context(error::PartitionFromDiskSnafu {
                    device: os_disk.path(),
                })?
                .context(error::PartitionNotFoundOnDeviceSnafu {
                    num,
                    device: os_disk.path(),
                })?
                .path())
        };

        let boot_partition_nums = [
            nth_guid(BOTTLEROCKET_BOOT, 0)?,
            nth_guid(BOTTLEROCKET_BOOT, 1)?,
        ];
        let sets = [
            PartitionSet {
                boot: device_from_part_num(boot_partition_nums[0])?,
                root: device_from_part_num(nth_guid(BOTTLEROCKET_ROOT, 0)?)?,
                hash: device_from_part_num(nth_guid(BOTTLEROCKET_HASH, 0)?)?,
            },
            PartitionSet {
                boot: device_from_part_num(boot_partition_nums[1])?,
                root: device_from_part_num(nth_guid(BOTTLEROCKET_ROOT, 1)?)?,
                hash: device_from_part_num(nth_guid(BOTTLEROCKET_HASH, 1)?)?,
            },
        ];

        // Determine which set is active by seeing which set contains the current running root or
        // hash partition.
        let active = if sets[0].contains(&active_partition) {
            SetSelect::A
        } else if sets[1].contains(&active_partition) {
            SetSelect::B
        } else {
            return error::ActiveNotInSetSnafu {
                active_partition,
                sets,
            }
            .fail();
        };

        Ok(Self {
            os_disk: os_disk.path(),
            private_partition_num: nth_guid(BOTTLEROCKET_PRIVATE, 0)?,
            sets,
            boot_partition_nums,
            table,
            active,
        })
    }

    pub(crate) fn os_disk(&self) -> &Path {
        &self.os_disk
    }

    fn gpt_attributes(&self, num_part: u32) -> u64 {
        self.table[num_part].attribute_bits
    }

    fn gptprio(&self, select: SetSelect) -> GptPrio {
        GptPrio::from(self.gpt_attributes(self.boot_partition_nums[select.idx()]))
    }

    fn set_gpt_attributes(&mut self, num_part: u32, flags: GptPrio) {
        self.table[num_part].attribute_bits = flags.into();
    }

    fn set_gptprio(&mut self, select: SetSelect, flags: GptPrio) {
        self.set_gpt_attributes(self.boot_partition_nums[select.idx()], flags);
    }

    pub fn active(&self) -> SetSelect {
        self.active
    }

    pub fn inactive(&self) -> SetSelect {
        // resolve opposing set member
        !self.active
    }

    pub fn active_set(&self) -> &PartitionSet {
        &self.sets[self.active().idx()]
    }

    pub fn inactive_set(&self) -> &PartitionSet {
        &self.sets[self.inactive().idx()]
    }

    pub fn next(&self) -> Option<SetSelect> {
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
    /// Marks the BOTTLEROCKET_PRIVATE partition table to indicate that boot has succeeded at least once
    pub fn mark_successful_boot(&mut self) {
        let mut flags = self.gptprio(self.active());
        flags.set_successful(true);
        self.set_gptprio(self.active(), flags);

        let mut private_flags = GptPrio::from(self.gpt_attributes(self.private_partition_num));
        private_flags.boot_has_succeeded();
        self.set_gpt_attributes(self.private_partition_num, private_flags);
    }

    /// Clears priority bits of the inactive partition in preparation to write new images, but
    /// **does not write to the disk**.
    pub fn clear_inactive(&mut self) {
        let mut inactive_flags = self.gptprio(self.inactive());
        inactive_flags.set_priority(0);
        inactive_flags.set_tries_left(0);
        inactive_flags.set_successful(false);
        self.set_gptprio(self.inactive(), inactive_flags);
    }

    /// Sets 'tries left' to 1 on the inactive partition to represent a
    /// potentially valid image, but does not change the priority.
    /// **does not write to the disk**.
    pub fn mark_inactive_valid(&mut self) {
        let mut inactive_flags = self.gptprio(self.inactive());
        inactive_flags.set_tries_left(1);
        self.set_gptprio(self.inactive(), inactive_flags);
    }

    /// Sets the inactive partition as a new upgrade partition, but **does not write to the disk**.
    /// Ensures that the inactive partition is marked as valid beforehand
    ///
    /// * Sets the inactive partition's priority to 2 and the active partition's priority to 1.
    /// * Sets the inactive partition as not successfully booted.
    /// * Returns an error if the partition has not been marked as potentially
    ///   valid or if it has already been marked for upgrade.
    pub fn upgrade_to_inactive(&mut self) -> Result<(), Error> {
        let mut inactive_flags = self.gptprio(self.inactive());
        ensure!(
            inactive_flags.priority() == 0 && !inactive_flags.successful(),
            error::InactiveAlreadyMarkedSnafu {
                inactive: &self.os_disk
            }
        );
        ensure!(
            inactive_flags.tries_left() > 0,
            error::InactiveNotValidSnafu {
                inactive: &self.os_disk
            }
        );

        inactive_flags.set_priority(2);
        inactive_flags.set_successful(false);
        self.set_gptprio(self.inactive(), inactive_flags);

        let mut active_flags = self.gptprio(self.active());
        active_flags.set_priority(1);
        self.set_gptprio(self.active(), active_flags);
        Ok(())
    }

    /// Reverts upgrade_to_inactive(), but **does not write to the disk**.
    ///
    /// * Sets the inactive partition's priority to 0
    /// * Restores the active partition's priority to 2
    pub fn cancel_upgrade(&mut self) {
        let mut inactive_flags = self.gptprio(self.inactive());
        inactive_flags.set_priority(0);
        self.set_gptprio(self.inactive(), inactive_flags);

        let mut active_flags = self.gptprio(self.active());
        active_flags.set_priority(2);
        self.set_gptprio(self.active(), active_flags);
    }

    /// Prioritizes the inactive partition, but **does not write to the disk**.
    ///
    /// Returns an error if the inactive partition is not bootable (it doesn't have a prior
    /// successful boot and doesn't have the priority/tries_left that would make it safe to try).
    pub fn rollback_to_inactive(&mut self) -> Result<(), Error> {
        let mut inactive_flags = self.gptprio(self.inactive());
        if !inactive_flags.will_boot() {
            return error::InactiveInvalidRollbackSnafu {
                priority: inactive_flags.priority(),
                tries_left: inactive_flags.tries_left(),
                successful: inactive_flags.successful(),
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

    /// Returns whether boot has ever succeeded or not
    pub fn has_boot_succeeded(&mut self) -> bool {
        let private_flags = GptPrio::from(self.gpt_attributes(self.private_partition_num));
        private_flags.has_boot_succeeded()
    }

    /// Writes the partition table to the OS disk.
    pub fn write(&mut self) -> Result<(), Error> {
        self.table
            .write_into(
                &mut OpenOptions::new()
                    .write(true)
                    .open(self.os_disk())
                    .context(error::OpenSnafu {
                        path: &self.os_disk,
                        what: "writing",
                    })?,
            )
            .map_err(error::GPTError)
            .context(error::GPTWriteSnafu {
                device: &self.os_disk,
            })?;
        Ok(())
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "OS disk: {}", self.os_disk.display())?;
        writeln!(
            f,
            "Set A:   {} {}",
            self.sets[SetSelect::A.idx()],
            self.gptprio(SetSelect::A)
        )?;
        writeln!(
            f,
            "Set B:   {} {}",
            self.sets[SetSelect::B.idx()],
            self.gptprio(SetSelect::B)
        )?;
        writeln!(f, "Active:  Set {}", self.active())?;
        match self.next() {
            Some(next) => write!(f, "Next:    Set {}", next),
            None => write!(f, "Next:    None"),
        }
    }
}
