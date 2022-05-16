use crate::error;
use crate::error::Result;
use bytes::BufMut;
use snafu::{OptionExt, ResultExt};
use std::convert::TryInto;
use std::mem::size_of;

const BASE_INITRD_SIZE: usize = 0;
const BOOTCONFIG_MAGIC: &str = "#BOOTCONFIG\n";
// 1 << 2 = 4
const BOOTCONFIG_ALIGN: usize = 0b100;

/// This generates an initrd with just the bootconfig according to the format described in
/// https://www.kernel.org/doc/html/latest/admin-guide/bootconfig.html#boot-kernel-with-a-boot-config
pub(crate) fn generate_initrd(bootconfig: &[u8]) -> Result<Vec<u8>> {
    let mut bootconfig_size = bootconfig.len();
    // Calculate checksum of the boot config file so it can be included in the initrd image
    let mut checksum: u32 = 0;
    for byte in bootconfig {
        checksum = checksum
            .checked_add(u32::from(*byte))
            .context(error::AddU32OverflowSnafu)?;
    }
    // Bottlerocket does not use an initrd, so the base initrd image size is 0
    let total_size = BASE_INITRD_SIZE
        + bootconfig_size
        + size_of::<u32>() * 2
        + BOOTCONFIG_MAGIC.as_bytes().len();
    let mut initrd = bootconfig.to_owned();
    // initrd image file needs to be 4-byte aligned, so we add padding as necessary.
    let padding_size = (BOOTCONFIG_ALIGN - (total_size % BOOTCONFIG_ALIGN)) % BOOTCONFIG_ALIGN;
    trace!("Boot config size: {}", bootconfig_size);
    trace!("Initrd total size: {}", total_size);
    trace!("Padding size: {}", padding_size);
    initrd.put_bytes(b'\0', padding_size);
    bootconfig_size += padding_size;
    // Append boot config file size (file + padding bytes)
    initrd.put_u32_le(bootconfig_size.try_into().context(error::UsizeToU32Snafu)?);
    // Append boot config file checksum
    initrd.put_u32_le(checksum);
    // Append boot config magic value
    initrd.put(BOOTCONFIG_MAGIC.as_bytes());
    Ok(initrd)
}

#[cfg(test)]
mod tests {
    use crate::initrd::generate_initrd;

    static GOOD_BOOTCONFIG: &str = include_str!("../tests/data/ser_good_bootconfig");
    static INITRD_FROM_GOOD_BOOTCONFIG: &[u8] =
        include_bytes!("../tests/data/initrd_from_good_bootconfig");

    #[test]
    fn test_initrd_gen() {
        let bootconfig = GOOD_BOOTCONFIG;
        assert_eq!(
            INITRD_FROM_GOOD_BOOTCONFIG,
            generate_initrd(&bootconfig.as_bytes()).unwrap()
        );
    }
}
