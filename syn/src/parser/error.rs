use crate::{Error, ErrorCode};

// Removes any internal #[msg] attributes, as they are inert.
pub fn parse(error_enum: &mut syn::ItemEnum) -> Error {
    let ident = error_enum.ident.clone();
    let mut last_discriminant = 0;
    let codes: Vec<ErrorCode> = error_enum
        .variants
        .iter_mut()
        .map(|variant: &mut syn::Variant| {
            let msg = parse_error_attribute(variant);
            let ident = variant.ident.clone();
            let id = match &variant.discriminant {
                None => last_discriminant,
                Some((_, disc)) => match disc {
                    syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                        syn::Lit::Int(int) => {
                            int.base10_parse::<u32>().expect("Must be a base 10 number")
                        }
                        _ => panic!("Invalid error discriminant"),
                    },
                    _ => panic!("Invalid error discriminant"),
                },
            };
            last_discriminant = id + 1;

            // Remove any attributes on the error variant.
            variant.attrs = vec![];

            ErrorCode { id, ident, msg }
        })
        .collect();

    Error {
        name: error_enum.ident.to_string(),
        raw_enum: error_enum.clone(),
        ident,
        codes,
    }
}

fn parse_error_attribute(variant: &syn::Variant) -> Option<String> {
    let attrs = &variant.attrs;
    match attrs.len() {
        0 => None,
        1 => {
            let attr = &attrs[0];
            let attr_str = attr.path.segments[0].ident.to_string();
            if &attr_str != "msg" {
                panic!("Use msg to specify error strings");
            }

            let mut tts = attr.tokens.clone().into_iter();
            let g_stream = match tts.next().expect("Must have a token group") {
                proc_macro2::TokenTree::Group(g) => g.stream(),
                _ => panic!("Invalid syntax"),
            };

            let msg = match g_stream.into_iter().next() {
                None => panic!("Must specify a message string"),
                Some(msg) => msg.to_string().replace("\"", ""),
            };

            Some(msg)
        }
        _ => {
            panic!("Too many attributes found. Use `msg` to specify error strings");
        }
    }
}
