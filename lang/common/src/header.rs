use arrayref::array_ref;
use solana_program::hash;
use std::io::{Cursor, Write};

#[cfg(feature = "deprecated-layout")]
pub const DISCRIMINATOR_LEN: usize = 8;
#[cfg(not(feature = "deprecated-layout"))]
pub const DISCRIMINATOR_LEN: usize = 4;
pub const HEADER_LEN: usize = 8;

const LAYOUT_VERSION: u8 = 1;
const DEFAULT_ACCOUNT_VERSION: u8 = 1;

const HEADER_LAYOUT_VERSION_INDEX: usize = 0;
const HEADER_TYPE_VERSION_INDEX: usize = 1;
#[cfg(feature = "deprecated-layout")]
const HEADER_DISCRIMINATOR_INDEX: usize = 0;
#[cfg(not(feature = "deprecated-layout"))]
const HEADER_DISCRIMINATOR_INDEX: usize = 2;

// Initializes the header. Should only be run once.
pub fn init(account_data: &mut [u8], discriminator: &[u8]) {
    if !cfg!(feature = "deprecated-layout") {
        write_layout_version(account_data, LAYOUT_VERSION);
        write_account_version(account_data, DEFAULT_ACCOUNT_VERSION);
    }
    write_discriminator(account_data, discriminator);
}

fn write_layout_version(account_data: &mut [u8], version: u8) {
    account_data[HEADER_LAYOUT_VERSION_INDEX] = version;
}

fn write_account_version(account_data: &mut [u8], version: u8) {
    account_data[HEADER_TYPE_VERSION_INDEX] = version;
}

fn write_discriminator(account_data: &mut [u8], discriminator: &[u8]) {
    let dst: &mut [u8] = &mut account_data[HEADER_DISCRIMINATOR_INDEX..];
    let mut cursor = Cursor::new(dst);
    cursor.write_all(discriminator).unwrap();
}

pub fn read_discriminator(data: &[u8]) -> &[u8; DISCRIMINATOR_LEN] {
    array_ref![data, HEADER_DISCRIMINATOR_INDEX, DISCRIMINATOR_LEN]
}

pub fn create_discriminator(
    account_name: &str,
    namespace: Option<&str>,
) -> [u8; DISCRIMINATOR_LEN] {
    let discriminator_preimage = format!("{}:{}", namespace.unwrap_or("account"), account_name);
    let mut discriminator = [0u8; DISCRIMINATOR_LEN];
    discriminator.copy_from_slice(
        &hash::hash(discriminator_preimage.as_bytes()).to_bytes()[..DISCRIMINATOR_LEN],
    );
    discriminator
}

pub fn read_data(account_data: &[u8]) -> &[u8] {
    &account_data[HEADER_LEN..]
}

pub fn read_data_mut(account_data: &mut [u8]) -> &mut [u8] {
    &mut account_data[HEADER_LEN..]
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
