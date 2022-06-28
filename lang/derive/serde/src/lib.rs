extern crate proc_macro;

use anchor_syn::idl::gen::*;
use borsh_derive_internal::*;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, Item};

fn gen_borsh_serialize(input: TokenStream) -> TokenStream2 {
    let cratename = Ident::new("borsh", Span::call_site());

    let item: Item = syn::parse(input).unwrap();
    let res = match item {
        Item::Struct(item) => struct_ser(&item, cratename),
        Item::Enum(item) => enum_ser(&item, cratename),
        Item::Union(item) => union_ser(&item, cratename),
        // Derive macros can only be defined on structs, enums, and unions.
        _ => unreachable!(),
    };

    match res {
        Ok(res) => res,
        Err(err) => err.to_compile_error(),
    }
}

#[proc_macro_derive(AnchorSerialize, attributes(borsh_skip))]
pub fn anchor_serialize(input: TokenStream) -> TokenStream {
    let borsh = gen_borsh_serialize(input.clone());

    let no_docs = false; // TODO

    let idl_gen_impl = match syn::parse(input).unwrap() {
        Item::Struct(item) => gen_idl_gen_impl_for_struct(&item, no_docs),
        Item::Enum(item) => gen_idl_gen_impl_for_enum(item, no_docs),
        Item::Union(item) => {
            // unions are not included in the IDL - TODO print a warning
            idl_gen_impl_skeleton(quote! {None}, quote! {}, &item.ident, &item.generics)
        }
        // Derive macros can only be defined on structs, enums, and unions.
        _ => unreachable!(),
    };

    TokenStream::from(quote! {
        #borsh
        #idl_gen_impl
    })
}

fn gen_borsh_deserialize(input: TokenStream) -> TokenStream2 {
    let cratename = Ident::new("borsh", Span::call_site());

    let item: Item = syn::parse(input).unwrap();
    let res = match item {
        Item::Struct(item) => struct_de(&item, cratename),
        Item::Enum(item) => enum_de(&item, cratename),
        Item::Union(item) => union_de(&item, cratename),
        // Derive macros can only be defined on structs, enums, and unions.
        _ => unreachable!(),
    };

    match res {
        Ok(res) => res,
        Err(err) => err.to_compile_error(),
    }
}

#[proc_macro_derive(AnchorDeserialize, attributes(borsh_skip, borsh_init))]
pub fn borsh_deserialize(input: TokenStream) -> TokenStream {
    TokenStream::from(gen_borsh_deserialize(input))
}
