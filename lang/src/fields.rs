use solana_program::pubkey::Pubkey;

pub struct Field {
    pub name: &'static str,
    pub address: Pubkey,
    pub is_mutable: bool,
}

// A data structure that has fields with the following attributes:
// (name, address, Option<name and address the field may be a duplicate of>, is_writable)
pub trait Fields {
    fn fields(&self) -> Vec<Field>;
}
