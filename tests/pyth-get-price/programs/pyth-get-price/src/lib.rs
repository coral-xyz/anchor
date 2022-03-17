use anchor_lang::prelude::*;

declare_id!("EVFzuVuE2DGP7VVAFafVq4knkQ4dqF3mmCtdMcpNLoSA");

#[program]
pub mod pyth_get_price {
    use pyth_client::{load_price, Price};
    use super::*;

    pub fn get_price(ctx: Context<PythGetPrice>) -> Result<()> {
        let pyth_price_info = &ctx.accounts.pyth_price;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let price_account: Price = *load_price(pyth_price_data).unwrap();

        msg!("price: {}", price_account.agg.price);
        msg!("price: {:?}", price_account.get_current_price().unwrap());
        msg!("status: {:?}", price_account.get_current_price_status());

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