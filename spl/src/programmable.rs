pub use mpl_token_auth_rules;

#[derive(Clone, Debug, PartialEq)]
pub struct TokenRecordAccount(mpl_token_metadata::state::TokenRecord);

impl TokenRecordAccount {
    pub const LEN: usize = mpl_token_metadata::state::TOKEN_RECORD_SIZE;
}
impl anchor_lang::AccountDeserialize for TokenRecordAccount {
    fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let tr = Self::try_deserialize_unchecked(buf)?;
        if tr.key != mpl_token_metadata::state::TokenRecord::key() {
            return Err(ErrorCode::AccountNotInitialized.into());
        }
        Ok(tr)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let tr = mpl_token_metadata::state::TokenRecord::safe_deserialize(buf)?;
        Ok(Self(tr))
    }
}

impl anchor_lang::AccountSerialize for TokenRecordAccount {}

impl anchor_lang::Owner for TokenRecordAccount {
    fn owner() -> Pubkey {
        ID
    }
}

impl Deref for TokenRecordAccount {
    type Target = mpl_token_metadata::state::TokenRecord;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct TokenAuthRules;

impl anchor_lang::Id for TokenAuthRules {
    fn id() -> Pubkey {
        mpl_token_auth_rules::ID
    }
}
