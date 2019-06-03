# signpost

si**g**n**p**os**t** is Thar's utility for modifying GPT priority bits on its OS disk.

```plain
USAGE:
    signpost <SUBCOMMAND>

SUBCOMMANDS:
    status                  Show partition sets and priority status
    mark-successful-boot    Mark the active partition as successfully booted
    upgrade-to-inactive     Sets the inactive partition as a new upgrade partition
    rollback-to-inactive    Deprioritizes the inactive partition
    rewrite-table           Rewrite the partition table with no changes to disk (used for testing this code)
```

Thar uses the same GPT partition attribute flags as Chrome OS, which are [used by GRUB to select the partition to read a grub.cfg from](../../packages/grub/gpt.patch).

| Bits | Content |
|-|-|
| 63-57 | Unused |
| 56 | Successful boot flag |
| 55-52 | Tries remaining |
| 51-48 | Priority |
| 47-0 | Reserved by GPT specification |
