use crate::dex;
use crate::dex::middleware::{Context, ErrorCode, MarketMiddleware};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use serum_dex::instruction::*;

/// MarketProxy provides an abstraction for implementing proxy programs to the
/// Serum orderbook, allowing one to implement a middleware for the purposes
/// of intercepting and modifying requests before being relayed to the
/// orderbook.
///
/// The only requirement for a middleware is that, when all are done processing,
/// a valid DEX instruction--accounts and instruction data--must be left to
/// forward to the orderbook program.
pub struct MarketProxy<'a> {
    middlewares: Vec<&'a mut dyn MarketMiddleware>,
}

impl<'a> MarketProxy<'a> {
    /// Constructs a new `MarketProxy`.
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Builder method for adding a middleware to the proxy.
    pub fn middleware(mut self, mw: &'a mut dyn MarketMiddleware) -> Self {
        self.middlewares.push(mw);
        self
    }

    /// Entrypoint to the program.
    pub fn run(
        mut self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8],
    ) -> ProgramResult {
        let mut ix_data = &data[..];

        // First account is the Serum DEX executable--used for CPI.
        let dex = &accounts[0];
        let acc_infos = (&accounts[1..]).to_vec();

        // Process the instruction data.
        for mw in &mut self.middlewares {
            mw.instruction(&mut ix_data)?;
        }

        // Request context.
        let mut ctx = Context::new(program_id, dex.key, acc_infos);

        // Decode instruction.
        let ix = MarketInstruction::unpack(ix_data);

        // Method dispatch.
        match ix {
            Some(MarketInstruction::InitOpenOrders) => {
                require!(ctx.accounts.len() >= 4, ErrorCode::NotEnoughAccounts);
                for mw in &self.middlewares {
                    mw.init_open_orders(&mut ctx)?;
                }
            }
            Some(MarketInstruction::NewOrderV3(ix)) => {
                require!(ctx.accounts.len() >= 12, ErrorCode::NotEnoughAccounts);
                for mw in &self.middlewares {
                    mw.new_order_v3(&mut ctx, &ix)?;
                }
            }
            Some(MarketInstruction::CancelOrderV2(ix)) => {
                require!(ctx.accounts.len() >= 6, ErrorCode::NotEnoughAccounts);
                for mw in &self.middlewares {
                    mw.cancel_order_v2(&mut ctx, &ix)?;
                }
            }
            Some(MarketInstruction::CancelOrderByClientIdV2(ix)) => {
                require!(ctx.accounts.len() >= 6, ErrorCode::NotEnoughAccounts);
                for mw in &self.middlewares {
                    mw.cancel_order_by_client_id_v2(&mut ctx, ix)?;
                }
            }
            Some(MarketInstruction::SettleFunds) => {
                require!(ctx.accounts.len() >= 10, ErrorCode::NotEnoughAccounts);
                for mw in &self.middlewares {
                    mw.settle_funds(&mut ctx)?;
                }
            }
            Some(MarketInstruction::CloseOpenOrders) => {
                require!(ctx.accounts.len() >= 4, ErrorCode::NotEnoughAccounts);
                for mw in &self.middlewares {
                    mw.close_open_orders(&mut ctx)?;
                }
            }
            _ => {
                for mw in &self.middlewares {
                    mw.fallback(&mut ctx)?;
                }
                return Ok(());
            }
        };

        // Extract the middleware adjusted context.
        let Context {
            seeds, accounts, ..
        } = ctx;

        // Convert the signer seeds into a slice.
        //
        // This is wasteful but is there a better way to have signer seeds
        // appendable by middleware handlers?
        let tmp_signers: Vec<Vec<&[u8]>> = seeds
            .iter()
            .map(|seeds| {
                let seeds: Vec<&[u8]> = seeds.iter().map(|seed| &seed[..]).collect();
                seeds
            })
            .collect();
        let signers: Vec<&[&[u8]]> = tmp_signers.iter().map(|seeds| &seeds[..]).collect();

        // CPI to the DEX.
        let dex_accounts = accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect();
        let ix = anchor_lang::solana_program::instruction::Instruction {
            data: ix_data.to_vec(),
            accounts: dex_accounts,
            program_id: dex::ID,
        };
        anchor_lang::solana_program::program::invoke_signed(&ix, &accounts, &signers)
    }
}
