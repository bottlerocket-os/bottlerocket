# signpost

si**g**n**p**os**t** is a utility for modifying Chrome OS-style GPT priority bits on an OS disk.

```plain
USAGE:
    signpost <SUBCOMMAND>

SUBCOMMANDS:
    status                  Show partition sets and priority status
    mark-successful-boot    Mark the active partitions as successfully booted
    clear-inactive          Clears inactive priority information to prepare writing images to disk
    mark-inactive-valid     Marks the inactive partition as having a valid image
    upgrade-to-inactive     Sets the inactive partitions as new upgrade partitions if marked valid
    cancel-upgrade          Reverse upgrade-to-inactive
    rollback-to-inactive    Deprioritizes the inactive partitions
    has-boot-ever-succeeded Checks whether boot has ever succeeded
    rewrite-table           Rewrite the partition table with no changes to disk (used for testing this code)
```

## Background

The Bottlerocket OS disk has two partition sets, each containing three partitions:

* the *boot* partition, containing the `vmlinuz` Linux kernel image and the GRUB configuration.
* the *root* partition, containing the read-only `/` filesystem.
* the *hash* partition, containing the full dm-verity hash tree for the root partition.

The Bottlerocket boot partition uses the same GPT partition attribute flags as Chrome OS, which are used by GRUB to select the partition from which to read a `grub.cfg`:

| Bits  | Content                         |
|-------|---------------------------------|
| 63-56 | Unused                          |
| 57    | Have successfully booted before |
| 56    | Successful boot flag            |
| 55-52 | Tries remaining                 |
| 51-48 | Priority                        |
| 47-0  | Reserved by GPT specification   |

The boot partition GRUB selects contains a grub.cfg which references the root and hash partitions by offset, thus selecting all three partitions of a set.

## Upgrade procedure

1. Run `signpost clear-inactive` to clear the priority and successful bits before making any changes to the inactive partitions.
2. Copy the downloaded images to the inactive partitions on disk, then validate data was written correctly.
3. Run `signpost upgrade-to-inactive` to prioritize the inactive partitions and allow it one boot attempt before automatically rolling back.

## Rollback procedure

1. Run `signpost rollback-to-inactive` to prioritize the inactive partitions without modifying whether the active partitions were successful.
