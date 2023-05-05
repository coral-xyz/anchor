extern crate proc_macro;

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Token,
};

/// The event attribute allows a struct to be used with
/// [emit!](./macro.emit.html) so that programs can log significant events in
/// their programs that clients can subscribe to. Currently, this macro is for
/// structs only.
///
/// See the [`emit!` macro](emit!) for an example.
#[proc_macro_attribute]
pub fn event(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let event_strct = parse_macro_input!(input as syn::ItemStruct);

    let event_name = &event_strct.ident;

    let discriminator: proc_macro2::TokenStream = {
        let discriminator_preimage = format!("event:{event_name}");
        let mut discriminator = [0u8; 8];
        discriminator.copy_from_slice(
            &anchor_syn::hash::hash(discriminator_preimage.as_bytes()).to_bytes()[..8],
        );
        format!("{discriminator:?}").parse().unwrap()
    };

    proc_macro::TokenStream::from(quote! {
        #[derive(anchor_lang::__private::EventIndex, AnchorSerialize, AnchorDeserialize)]
        #event_strct

        impl anchor_lang::Event for #event_name {
            fn data(&self) -> Vec<u8> {
                let mut d = #discriminator.to_vec();
                d.append(&mut self.try_to_vec().unwrap());
                d
            }
        }

        impl anchor_lang::Discriminator for #event_name {
            const DISCRIMINATOR: [u8; 8] = #discriminator;
        }
    })
}

/// Logs an event that can be subscribed to by clients.
/// Uses the [`sol_log_data`](https://docs.rs/solana-program/latest/solana_program/log/fn.sol_log_data.html)
/// syscall which results in the following log:
/// ```ignore
/// Program data: <Base64EncodedEvent>
/// ```
/// # Example
///
/// ```rust,ignore
/// use anchor_lang::prelude::*;
///
/// // handler function inside #[program]
/// pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
///     emit!(MyEvent {
///         data: 5,
///         label: [1,2,3,4,5],
///     });
///     Ok(())
/// }
///
/// #[event]
/// pub struct MyEvent {
///     pub data: u64,
///     pub label: [u8; 5],
/// }
/// ```
#[proc_macro]
pub fn emit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let data: proc_macro2::TokenStream = input.into();
    proc_macro::TokenStream::from(quote! {
        {
            anchor_lang::solana_program::log::sol_log_data(&[&anchor_lang::Event::data(&#data)]);
        }
    })
}

// Custom wrapper struct (thanks ChatGPT!)
struct EmitCpiArgs {
    self_program_info: Expr,
    event_authority_info: Expr,
    event_authority_bump: Expr,
    event_struct: Expr,
}

// Implement the `Parse` trait for the `TwoArgs` struct
impl Parse for EmitCpiArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let self_program_info: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let event_authority_info: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let event_authority_bump: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let event_struct: Expr = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("Expected exactly 3 arguments"));
        }

        Ok(EmitCpiArgs {
            self_program_info,
            event_authority_info,
            event_authority_bump,
            event_struct,
        })
    }
}

/// Logs an event that can be subscribed to by clients. More stable than `emit!` because
/// RPCs are less likely to truncate CPI information than program logs. Generated code for this feature
/// can be disabled by adding `no-cpi-events` to the `defaults = []` section of your program's Cargo.toml.
///
/// Uses a [`invoke_signed`](https://docs.rs/solana-program/latest/solana_program/program/fn.invoke_signed.html)
/// syscall to store data in the ledger, which results in data being stored in the transaction metadata.
///
/// This also requires the usage of an additional PDA, derived from &[b"__event_authority"], to guarantee that
/// the self-CPI is truly being invoked by the same program. Requiring this PDA to be a signer during `invoke_signed` syscall
/// ensures that the program is the one doing the logging.
///
/// # Example
///
/// ```rust,ignore
/// use anchor_lang::prelude::*;
///
/// // handler function inside #[program]
/// pub fn do_something(ctx: Context<DoSomething>) -> Result<()> {
///     emit_cpi!(
///         ctx.accounts.program.to_account_info(),
///         ctx.accounts.event_authority.clone(),
///         *ctx.bumps.get("event_authority").unwrap(),
///         MyEvent {
///             data: 5,
///             label: [1,2,3,4,5],
///         }
///     );
///     Ok(())
/// }
///
/// #[derive(Accounts)]
/// pub struct DoSomething<'info> {
///     /// CHECK: this account is needed to guarantee that your program is the one doing the logging
///     #[account(seeds=[b"__event_authority"], bump)]
///     pub event_authority: AccountInfo<'info>,
///     pub program: Program<'info, crate::program::MyProgramName>,
/// }
///
/// #[event]
/// pub struct MyEvent {
///     pub data: u64,
///     pub label: [u8; 5],
/// }
/// ```
#[proc_macro]
pub fn emit_cpi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(input as EmitCpiArgs);

    let self_program_info = &args.self_program_info;
    let event_authority_info = &args.event_authority_info;
    let event_authority_bump = &args.event_authority_bump;
    let event_struct = &args.event_struct;

    proc_macro::TokenStream::from(quote! {
        let __program_info: anchor_lang::solana_program::account_info::AccountInfo = #self_program_info;
        let __event_authority_info: anchor_lang::solana_program::account_info::AccountInfo = #event_authority_info;
        let __event_authority_bump: u8 = #event_authority_bump;

        let __disc = crate::event::EVENT_IX_TAG_LE;
        let __inner_data: Vec<u8> = anchor_lang::Event::data(&#event_struct);
        let __ix_data: Vec<u8> = __disc.into_iter().chain(__inner_data.into_iter()).collect();

        let __ix = anchor_lang::solana_program::instruction::Instruction::new_with_bytes(
            *__program_info.key,
            __ix_data.as_ref(),
            vec![
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*__event_authority_info.key, true)
            ]
        );
        anchor_lang::solana_program::program::invoke_signed(&__ix, &[__program_info, __event_authority_info], &[&[b"__event_authority", &[__event_authority_bump]]])
            .map_err(anchor_lang::error::Error::from)?;
    })
}

// EventIndex is a marker macro. It functionally does nothing other than
// allow one to mark fields with the `#[index]` inert attribute, which is
// used to add metadata to IDLs.
#[proc_macro_derive(EventIndex, attributes(index))]
pub fn derive_event(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! {})
}
