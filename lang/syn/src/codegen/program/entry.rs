use crate::program_codegen::dispatch;
use crate::Program;
use quote::quote;

pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    let fallback_maybe = dispatch::gen_fallback(program).unwrap_or(quote! {
        Err(anchor_lang::__private::ErrorCode::InstructionMissing.into());
    });
    quote! {
        #[cfg(not(feature = "no-entrypoint"))]
        anchor_lang::solana_program::entrypoint!(entry);
        /// The Anchor codegen exposes a programming model where a user defines
        /// a set of methods inside of a `#[program]` module in a way similar
        /// to writing RPC request handlers. The macro then generates a bunch of
        /// code wrapping these user defined methods into something that can be
        /// executed on Solana.
        ///
        /// These methods fall into one of three categories, each of which
        /// can be considered a different "namespace" of the program.
        ///
        /// 1) Global methods - regular methods inside of the `#[program]`.
        /// 2) State methods - associated methods inside a `#[state]` struct.
        /// 3) Interface methods - methods inside a strait struct's
        ///    implementation of an `#[interface]` trait.
        ///
        /// Care must be taken by the codegen to prevent collisions between
        /// methods in these different namespaces. For this reason, Anchor uses
        /// a variant of sighash to perform method dispatch, rather than
        /// something like a simple enum variant discriminator.
        ///
        /// The execution flow of the generated code can be roughly outlined:
        ///
        /// * Start program via the entrypoint.
        /// * Strip method identifier off the first 8 bytes of the instruction
        ///   data and invoke the identified method. The method identifier
        ///   is a variant of sighash. See docs.rs for `anchor_lang` for details.
        /// * If the method identifier is an IDL identifier, execute the IDL
        ///   instructions, which are a special set of hardcoded instructions
        ///   baked into every Anchor program. Then exit.
        /// * Otherwise, the method identifier is for a user defined
        ///   instruction, i.e., one of the methods in the user defined
        ///   `#[program]` module. Perform method dispatch, i.e., execute the
        ///   big match statement mapping method identifier to method handler
        ///   wrapper.
        /// * Run the method handler wrapper. This wraps the code the user
        ///   actually wrote, deserializing the accounts, constructing the
        ///   context, invoking the user's code, and finally running the exit
        ///   routine, which typically persists account changes.
        ///
        /// The `entry` function here, defines the standard entry to a Solana
        /// program, where execution begins.
        #[cfg(not(feature = "no-entrypoint"))]
        pub fn entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
            #[cfg(feature = "anchor-debug")]
            {
                msg!("anchor-debug is active");
            }
            if data.len() < 8 {
                return #fallback_maybe
            }

            dispatch(program_id, accounts, data)
                .map_err(|e| {
                    anchor_lang::solana_program::msg!(&e.to_string());
                    e
                })
        }
    }
}
