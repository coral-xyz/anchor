use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input,
    punctuated::{IntoIter, Punctuated},
    Attribute, DeriveInput, Fields, GenericArgument, LitInt, PathArguments, Token, Type, TypeArray,
};

#[proc_macro_derive(InitSpace, attributes(max_len))]
pub fn derive_anchor_deserialize(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;

    let expanded: TokenStream2 = match input.data {
        syn::Data::Struct(strct) => match strct.fields {
            Fields::Named(named) => {
                let recurse = named.named.into_iter().map(|f| {
                    let mut max_len_args = get_max_len_args(&f.attrs);
                    len_from_type(f.ty, &mut max_len_args)
                });

                quote! {
                    #[automatically_derived]
                    impl anchor_lang::Space for #name {
                        const INIT_SPACE: u64 = 0 #(+ #recurse)*;
                    }
                }
            }
            _ => panic!("Please use named fields in account structure"),
        },
        syn::Data::Enum(enm) => {
            let variants = enm.variants.into_iter().map(|v| {
                let len = v.fields.into_iter().map(|f| {
                    let mut max_len_args = get_max_len_args(&f.attrs);
                    len_from_type(f.ty, &mut max_len_args)
                });

                quote! {
                    0 #(+ #len)*
                }
            });

            let max = gen_max(variants);

            quote! {
                #[automatically_derived]
                impl anchor_lang::Space for #name {
                    const INIT_SPACE: u64 = 1 + #max;
                }
            }
        }
        _ => unimplemented!(),
    };

    TokenStream::from(expanded)
}

fn gen_max<T: Iterator<Item = TokenStream2>>(mut iter: T) -> TokenStream2 {
    if let Some(item) = iter.next() {
        let next_item = gen_max(iter);
        quote!(anchor_lang::__private::max(#item, #next_item))
    } else {
        quote!(0)
    }
}

fn len_from_type(ty: Type, attrs: &mut Option<IntoIter<LitInt>>) -> TokenStream2 {
    match ty {
        Type::Array(TypeArray { elem, len, .. }) => {
            let array_len = len.to_token_stream();
            let type_len = len_from_type(*elem, attrs);
            quote!((#array_len * #type_len))
        }
        Type::Path(path) => {
            let path_segment = path.path.segments.last().unwrap();
            let ident = &path_segment.ident;
            let type_name = ident.to_string();

            match type_name.as_str() {
                "i8" | "u8" | "bool" => quote!(1),
                "i16" | "u16" => quote!(2),
                "i32" | "u32" | "f32" => quote!(4),
                "i64" | "u64" | "f64" => quote!(8),
                "i128" | "u128" => quote!(16),
                "String" => {
                    let max_len = get_next_arg(ident, attrs);
                    quote!((4 + #max_len))
                }
                "Pubkey" => quote!(32),
                "Vec" => match &path_segment.arguments {
                    PathArguments::AngleBracketed(args) => {
                        let ty = args
                            .args
                            .iter()
                            .find_map(|el| match el {
                                GenericArgument::Type(ty) => Some(ty.to_owned()),
                                _ => None,
                            })
                            .unwrap();

                        let max_len = get_next_arg(ident, attrs);
                        let type_len = len_from_type(ty, attrs);

                        quote!((4 + #type_len * #max_len))
                    }
                    _ => panic!("Invalid argument in Vec"),
                },
                _ => {
                    let ty = &path_segment.ident;
                    quote!(<#ty as anchor_lang::Space>::INIT_SPACE)
                }
            }
        }
        _ => panic!("Type {:?} is not supported", ty),
    }
}

fn get_max_len_args(attributes: &[Attribute]) -> Option<IntoIter<LitInt>> {
    attributes
        .iter()
        .find(|a| a.path.is_ident("max_len"))
        .and_then(|a| {
            a.parse_args_with(Punctuated::<LitInt, Token![,]>::parse_terminated)
                .ok()
        })
        .map(|p| p.into_iter())
}

fn get_next_arg(ident: &Ident, args: &mut Option<IntoIter<LitInt>>) -> TokenStream2 {
    if let Some(arg_list) = args {
        if let Some(arg) = arg_list.next() {
            quote!(#arg)
        } else {
            quote_spanned!(ident.span() => compile_error!("The number of lengths are invalid."))
        }
    } else {
        quote_spanned!(ident.span() => compile_error!("Expected max_len attribute."))
    }
}
