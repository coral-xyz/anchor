extern crate proc_macro;

#[cfg(feature = "event-cpi")]
use anchor_syn::parser::accounts::event_cpi::{add_event_cpi_accounts, EventAuthority};
use quote::quote;
use syn::parse_macro_input;

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

// EventIndex is a marker macro. It functionally does nothing other than
// allow one to mark fields with the `#[index]` inert attribute, which is
// used to add metadata to IDLs.
#[proc_macro_derive(EventIndex, attributes(index))]
pub fn derive_event(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! {})
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

/// Log an event by making a self-CPI that can be subscribed to by clients.
///
/// This way of logging events is more reliable than [`emit!`](emit!) because RPCs are less likely
/// to truncate CPI information than program logs.
///
/// Uses a [`invoke_signed`](https://docs.rs/solana-program/latest/solana_program/program/fn.invoke_signed.html)
/// syscall to store the event data in the ledger, which results in the data being stored in the
/// transaction metadata.
///
/// This method requires the usage of an additional PDA to guarantee that the self-CPI is truly
/// being invoked by the same program. Requiring this PDA to be a signer during `invoke_signed`
/// syscall ensures that the program is the one doing the logging.
///
/// The necessary accounts are added to the accounts struct via [`#[event_cpi]`](event_cpi)
/// attribute macro.
///
/// # Example
///
/// ```ignore
/// use anchor_lang::prelude::*;
///
/// #[program]
/// pub mod my_program {
///     use super::*;
///
///     pub fn my_instruction(ctx: Context<MyInstruction>) -> Result<()> {
///         emit_cpi!(MyEvent { data: 42 });
///         Ok(())
///     }
/// }
///
/// #[event_cpi]
/// #[derive(Accounts)]
/// pub struct MyInstruction {}
///
/// #[event]
/// pub struct MyEvent {
///     pub data: u64,
/// }
/// ```
///
/// **NOTE:** This macro requires `ctx` to be in scope.
///
/// *Only available with `event-cpi` feature enabled.*
#[cfg(feature = "event-cpi")]
#[proc_macro]
pub fn emit_cpi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let event_struct = parse_macro_input!(input as syn::Expr);

    let authority = EventAuthority::get();
    let authority_name = authority.name_token_stream();
    let authority_name_str = authority.name;
    let authority_seeds = authority.seeds;

    proc_macro::TokenStream::from(quote! {
        {
            let authority_info = ctx.accounts.#authority_name.to_account_info();
            let authority_bump = *ctx.bumps.get(#authority_name_str).unwrap();

            let disc = anchor_lang::event::EVENT_IX_TAG_LE;
            let inner_data = anchor_lang::Event::data(&#event_struct);
            let ix_data: Vec<u8> = disc.into_iter().chain(inner_data.into_iter()).collect();

            let ix = anchor_lang::solana_program::instruction::Instruction::new_with_bytes(
                crate::ID,
                &ix_data,
                vec![
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        *authority_info.key,
                        true,
                    ),
                ],
            );
            anchor_lang::solana_program::program::invoke_signed(
                &ix,
                &[authority_info],
                &[&[#authority_seeds, &[authority_bump]]],
            )
            .map_err(anchor_lang::error::Error::from)?;
        }
    })
}

/// An attribute macro to add necessary event CPI accounts to the given accounts struct.
///
/// Two accounts named `event_authority` and `program` will be appended to the list of accounts.
///
/// # Example
///
/// ```ignore
/// #[event_cpi]
/// #[derive(Accounts)]
/// pub struct MyInstruction<'info> {
///    pub signer: Signer<'info>,
/// }
/// ```
///
/// The code above will be expanded to:
///
/// ```ignore
/// #[derive(Accounts)]
/// pub struct MyInstruction<'info> {
///    pub signer: Signer<'info>,
///    /// CHECK: Only the event authority can invoke self-CPI
///    #[account(seeds = [b"__event_authority"], bump)]
///    pub event_authority: AccountInfo<'info>,
///    /// CHECK: Self-CPI will fail if the program is not the current program
///    pub program: AccountInfo<'info>,
/// }
/// ```
///
/// See [`emit_cpi!`](emit_cpi!) for a full example.
///
/// *Only available with `event-cpi` feature enabled.*
#[cfg(feature = "event-cpi")]
#[proc_macro_attribute]
pub fn event_cpi(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let accounts_struct = parse_macro_input!(input as syn::ItemStruct);
    let accounts_struct = add_event_cpi_accounts(&accounts_struct).unwrap();
    proc_macro::TokenStream::from(quote! {#accounts_struct})
}
