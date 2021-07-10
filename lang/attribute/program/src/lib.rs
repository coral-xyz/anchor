extern crate proc_macro;

use quote::ToTokens;
use syn::parse_macro_input;

/// The `#[program]` attribute defines the module containing all instruction
/// handlers defining all entries into a Solana program.
///
/// # Arguments
/// | Name | Example | Description | Default/Required|
/// | :-- | :-- | :-- | :-- |
/// | `no_entrypoint_feature` | `#[program(no_entrypoint_feature = "not_us")]` | Sets the feature that will be used to declare no entrypoint | Default: `no-entrypoint` |
/// | `no_idl_feature` | `#[program(no_idl_feature = "dont_idl_me")]` | Sets the feature that will be used to remove idl code | Default: `no-idl` |
#[proc_macro_attribute]
pub fn program(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    parse_macro_input!(input as anchor_syn::Program)
        .to_token_stream()
        .into()
}
