extern crate proc_macro;

use borsh_derive_internal::*;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Ident, Item};

#[proc_macro_derive(AnchorSerialize, attributes(borsh_skip))]
pub fn anchor_serialize(input: TokenStream) -> TokenStream {
    let cratename = Ident::new("borsh", Span::call_site());

    let item: Item = syn::parse(input).unwrap();
    let res = match item {
        Item::Struct(item) => struct_ser(&item, cratename),
        Item::Enum(item) => enum_ser(&item, cratename),
        Item::Union(item) => union_ser(&item, cratename),
        // Derive macros can only be defined on structs, enums, and unions.
        _ => unreachable!(),
    };

    TokenStream::from(match res {
        Ok(res) => res,
        Err(err) => err.to_compile_error(),
    })
}

#[proc_macro_derive(AnchorDeserialize, attributes(borsh_skip, borsh_init))]
pub fn borsh_deserialize(input: TokenStream) -> TokenStream {
    let cratename = Ident::new("borsh", Span::call_site());

    let item: Item = syn::parse(input).unwrap();
    let res = match item {
        Item::Struct(item) => struct_de(&item, cratename),
        Item::Enum(item) => enum_de(&item, cratename),
        Item::Union(item) => union_de(&item, cratename),
        // Derive macros can only be defined on structs, enums, and unions.
        _ => unreachable!(),
    };

    TokenStream::from(match res {
        Ok(res) => res,
        Err(err) => err.to_compile_error(),
    })
}
