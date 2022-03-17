use anchor_lang::prelude::*;

declare_id!("D46LzWxNfmtAL6H2Dcn9uZVxjauKGhYVm9kCd3UPje2h");

/// Converted https://github.com/pyth-network/pyth-examples/blob/main/program/src/lib.rs to anchor
#[program]
pub mod pyth_get_price {
    use super::*;

    pub fn get_price(ctx: Context<PythGetPrice>) -> Result<()> {
        // setup accounts
        let pyth_product_info = &ctx.accounts.pyth_product;
        let pyth_product_data = &pyth_product_info.try_borrow_data()?;
        let product_account = *pyth_client::load_product(pyth_product_data).unwrap();

        // security checks
        if product_account.magic != pyth_client::MAGIC {
            msg!("Pyth product account provided is not a valid Pyth account");
            return Err(ProgramError::InvalidArgument.into());
        }

        if product_account.atype != pyth_client::AccountType::Product as u32 {
            msg!("Pyth product account provided is not a valid Pyth product account");
            return Err(ProgramError::InvalidArgument.into());
        }

        if product_account.ver != pyth_client::VERSION_2 {
            msg!("Pyth product account provided has a different version than the Pyth client");
            return Err(ProgramError::InvalidArgument.into());
        }

        if !product_account.px_acc.is_valid() {
            msg!("Pyth product price account is invalid");
            return Err(ProgramError::InvalidArgument.into());
        }

        let pyth_price_pubkey = Pubkey::new(&product_account.px_acc.val);
        if &pyth_price_pubkey != &ctx.accounts.pyth_price.key() {
            msg!("Pyth product price account does not match the Pyth price provided");
            return Err(ProgramError::InvalidArgument.into());
        }

        let pyth_price_info = &ctx.accounts.pyth_price;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let price_account = *pyth_client::load_price(pyth_price_data).unwrap();


        msg!("  price_account .. {:?}", pyth_price_info.key);
        msg!("    price_type ... {:?}", price_account.ptype);
        msg!("    price ........ {}", price_account.agg.price);

        // TODO: These are all thowing onchain errors:
        // Program failed to complete: Access violation in stack frame 5 at address 0x200005fd0 of size 8 by instruction #3018
        // msg!("    exponent ..... {}", price_account.expo);
        // msg!("    status ....... {:?}", price_account.agg.status);
        // msg!("    corp_act ..... {:?}",price_account.agg.corp_act);
        // msg!("    conf ......... {}", price_account.agg.conf);
        // msg!("    valid_slot ... {}", price_account.valid_slot);
        // msg!("    publish_slot . {}", price_account.agg.pub_slot);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct PythGetPrice<'info> {
    /// CHECK:
    pub pyth_product: AccountInfo<'info>,
    /// CHECK:
    pub pyth_price: AccountInfo<'info>,
}