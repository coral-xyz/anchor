use anchor_lang::prelude::*;

#[error_code]
pub enum OptionalErrors {
    #[msg("Failed realloc")]
    ReallocFailed,
}
