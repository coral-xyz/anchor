use proc_macro2::TokenStream;
use quote::quote;

use super::common::gen_print_section;

pub fn gen_idl_print_fn_address(address: String) -> TokenStream {
    let fn_body = gen_print_section("address", quote! { #address });

    quote! {
        #[test]
        pub fn __anchor_private_print_idl_address() {
            #fn_body
        }
    }
}
