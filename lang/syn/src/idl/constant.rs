use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{
    common::{gen_print_section, get_idl_module_path, get_no_docs},
    defined::gen_idl_type,
};
use crate::parser::docs;

pub fn gen_idl_print_fn_constant(item: &syn::ItemConst) -> TokenStream {
    let idl = get_idl_module_path();
    let no_docs = get_no_docs();

    let name = item.ident.to_string();
    let expr = &item.expr;
    let fn_name = format_ident!("__anchor_private_print_idl_const_{}", name.to_snake_case());

    let docs = match docs::parse(&item.attrs) {
        Some(docs) if !no_docs => quote! { vec![#(#docs.into()),*] },
        _ => quote! { vec![] },
    };

    let fn_body = match gen_idl_type(&item.ty, &[]) {
        Ok((ty, _)) => {
            // Get the type as a string for comparison
            let type_str = ty.to_string();
            
            // Use different formatting based on the type
            let value_format = if type_str == "string" {
                // For string types, use Display formatting to avoid extra quotes
                quote! { format!("{}", #expr) }
            } else {
                // For other types, keep using Debug formatting
                quote! { format!("{:?}", #expr) }
            };
            
            gen_print_section(
                "const",
                quote! {
                    #idl::IdlConst {
                        name: #name.into(),
                        docs: #docs,
                        ty: #ty,
                        value: #value_format,
                    }
                },
            )
        },
        _ => quote! {},
    };

    quote! {
        #[test]
        pub fn #fn_name() {
            #fn_body
        }
    }
}
