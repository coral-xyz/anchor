extern crate proc_macro;

use anchor_syn::idl::gen::gen_idl_print_function_for_constant;
use quote::quote;

/// A marker attribute used to mark const values that should be included in the
/// generated IDL but functionally does nothing.
#[proc_macro_attribute]
pub fn constant(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ts = match syn::parse(input).unwrap() {
        syn::Item::Const(item) => {
            let idl_print = gen_idl_print_function_for_constant(&item);
            quote! {
                #item
                #idl_print
            }
        }
        item => quote! {#item},
    };

    proc_macro::TokenStream::from(quote! {
        #ts
    })
}
