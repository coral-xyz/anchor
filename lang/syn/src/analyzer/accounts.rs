//! Perform syntax analysis on an accounts struct, providing additional compile
//! time guarantees for Anchor programs.

use crate::{AccountField, AccountsStruct};
use thiserror::Error;

pub fn analyze(accs: AccountsStruct) -> Result<AccountsStruct, AnalyzeError> {
    CpiOwner::analyze(&accs)?;
    OwnerRoot::analyze(&accs)?;
    Ok(accs)
}

#[derive(Error, Debug)]
pub enum AnalyzeError {
    #[error("Owner not specified on field: {0}")]
    OwnerNotSpecified(String),
}

trait SyntaxAnalyzer {
    fn analyze(accs: &AccountsStruct) -> Result<(), AnalyzeError>;
}

// Asserts all cpi accounts have an owner specified.
struct CpiOwner;
impl SyntaxAnalyzer for CpiOwner {
    fn analyze(accs: &AccountsStruct) -> Result<(), AnalyzeError> {
        // TODO
        Ok(())
    }
}

// Asserts all owners have explicit program ids, or are marked unsafe.
struct OwnerRoot;
impl SyntaxAnalyzer for OwnerRoot {
    fn analyze(accs: &AccountsStruct) -> Result<(), AnalyzeError> {
        // TODO
        Ok(())
    }
}
