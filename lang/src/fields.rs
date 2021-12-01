use solana_program::pubkey::Pubkey;

use crate::Key;

pub struct Field {
    pub name: &'static str,
    pub key: Pubkey,
    pub is_mutable: bool,
    pub dup_target: Option<&'static str>,
    pub path: Vec<&'static str>,
}

impl Field {
    pub fn build_path(&self) -> String {
        let mut t = self.path.clone();
        t.reverse();
        t.join(".")
    }
}

impl Key for Field {
    fn key(&self) -> Pubkey {
        self.key
    }
}

// A data structure that has fields with the following attributes:
// (name, address, Option<name and address the field may be a duplicate of>, is_writable)
pub trait Fields {
    fn fields(&self) -> Vec<Field>;
}
