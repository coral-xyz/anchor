use arrayref::array_ref;
use solana_program::hash;
use std::io::{Cursor, Write};

#[cfg(feature = "deprecated-layout")]
pub fn read_discriminator(data: &[u8]) -> &[u8; 8] {
    array_ref![data, 0, 8]
}

#[cfg(not(feature = "deprecated-layout"))]
pub fn read_discriminator(data: &[u8]) -> &[u8; 4] {
    array_ref![data, 2, 4]
}

#[cfg(feature = "deprecated-layout")]
pub fn create_discriminator(account_name: &str, namespace: Option<&str>) -> [u8; 8] {
    let discriminator_preimage = format!("{}:{}", namespace.unwrap_or("account"), account_name);
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash::hash(discriminator_preimage.as_bytes()).to_bytes()[..8]);
    discriminator
}

#[cfg(not(feature = "deprecated-layout"))]
pub fn create_discriminator(account_name: &str, namespace: Option<&str>) -> [u8; 4] {
    let discriminator_preimage = format!("{}:{}", namespace.unwrap_or("account"), account_name);
    let mut discriminator = [0u8; 4];
    discriminator.copy_from_slice(&hash::hash(discriminator_preimage.as_bytes()).to_bytes()[..4]);
    discriminator
}

// Header is 8 bytes regardless of layout.
pub fn read_data(account_data: &[u8]) -> &[u8] {
    &account_data[8..]
}

pub fn read_data_mut(account_data: &mut [u8]) -> &mut [u8] {
    &mut account_data[8..]
}

pub fn write_discriminator(account_data: &mut [u8], discriminator: &[u8]) {
    #[cfg(feature = "deprecated-layout")]
    {
        let mut cursor = Cursor::new(account_dst);
        cursor.write_all(discriminator).unwrap();
    }
    #[cfg(not(feature = "deprecated-layout"))]
    {
        let dst: &mut [u8] = &mut account_data[2..];
        let mut cursor = Cursor::new(dst);
        cursor.write_all(discriminator).unwrap();
    }
}

// Bit of a hack. We return the length as a string and then parse it into
// a token stream in the macro code generation, so that we can isolate all
// the feature flagging to this one module, here.
pub fn discriminator_len_str() -> &'static str {
    if cfg!(feature = "deprecated-layout") {
        "8"
    } else {
        "4"
    }
}
