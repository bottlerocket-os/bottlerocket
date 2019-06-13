#![allow(clippy::module_name_repetitions)]

pub const fn uuid_to_guid(uuid: [u8; 16]) -> [u8; 16] {
    [
        uuid[3], uuid[2], uuid[1], uuid[0], uuid[5], uuid[4], uuid[7], uuid[6], uuid[8], uuid[9],
        uuid[10], uuid[11], uuid[12], uuid[13], uuid[14], uuid[15],
    ]
}

#[cfg(test)]
mod tests {
    use crate::guid::uuid_to_guid;
    use hex_literal::hex;

    #[test]
    fn test() {
        assert_eq!(
            uuid_to_guid(hex!("21686148 6449 6e6f 744e 656564454649")),
            *b"Hah!IdontNeedEFI"
        );
    }
}
