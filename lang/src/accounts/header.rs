use arrayref::array_ref;

#[cfg(feature = "deprecated-layout")]
pub fn read_discriminator(data: &[u8]) -> &[u8; 8] {
    array_ref![data, 0, 8]
}

#[cfg(not(feature = "deprecated-layout"))]
pub fn read_discriminator<'a>(data: &[u8]) -> &[u8; 4] {
    array_ref![data, 2, 4]
}

#[cfg(feature = "deprecated-layout")]
pub fn create_discriminator(account_name: &str, namespace: Option<&str>) -> [u8; 8] {
    let discriminator_preimage = format!("{}:{}", namespace.unwrap_or("account"), account_name);
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(
        &crate::solana_program::hash::hash(discriminator_preimage.as_bytes()).to_bytes()[..8],
    );
    discriminator
}

#[cfg(not(feature = "deprecated-layout"))]
pub fn create_discriminator(account_name: &str, namespace: Option<&str>) -> [u8; 4] {
    let discriminator_preimage = format!("{}:{}", namespace.unwrap_or("account"), account_name);
    let mut discriminator = [0u8; 4];
    discriminator.copy_from_slice(
        &crate::solana_program::hash::hash(discriminator_preimage.as_bytes()).to_bytes()[..4],
    );
    discriminator
}

// Header is 8 bytes regardless of layout.
pub fn read_data(account_data: &[u8]) -> &[u8] {
    &account_data[8..]
}

pub fn read_data_mut(account_data: &mut [u8]) -> &mut [u8] {
    &mut account_data[8..]
}
