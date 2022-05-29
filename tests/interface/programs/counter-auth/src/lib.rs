//! counter-auth is an example of a program *implementing* an external program
//! interface. Here the `counter::Auth` trait, where we only allow a count
//! to be incremented if it changes the counter from odd -> even or even -> odd.
//! Creative, I know. :P.

use anchor_lang::prelude::*;
use counter::Auth;

declare_id!("Aws2XRVHjNqCUbMmaU245ojT2DBJFYX58KVo2YySEeeP");

#[program]
pub mod counter_auth {
    use super::*;

    #[state]
    pub struct CounterAuth;

    impl<'info> Auth<'info, Empty> for CounterAuth {
        fn is_authorized(_ctx: Context<Empty>, current: u64, new: u64) -> Result<bool> {
            if current % 2 == 0 {
                Ok(new % 2 == 1)
            } else {
                Ok(new % 2 == 0)
            }
        }

        fn authorize(_ctx: Context<Empty>, force: bool) -> Result<()> {
            if !force {
                Err(ProgramError::Custom(15000).into())
            } else {
                Ok(())
            }
        }
    }
}

#[derive(Accounts)]
pub struct Empty {}
