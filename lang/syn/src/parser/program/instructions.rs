use crate::parser::program::ctx_accounts_ident;
use crate::{FallbackFn, Ix, IxArg};
use syn::parse::{Error as ParseError, Result as ParseResult};
use syn::spanned::Spanned;

// Parse all non-state ix handlers from the program mod definition.
pub fn parse(program_mod: &syn::ItemMod) -> ParseResult<(Vec<Ix>, Option<FallbackFn>)> {
    let mod_content = &program_mod
        .content
        .as_ref()
        .ok_or_else(|| ParseError::new(program_mod.span(), "program content not provided"))?
        .1;

    let ixs = mod_content
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(item_fn) => {
                let (ctx, _) = parse_args(item_fn).ok()?;
                ctx_accounts_ident(&ctx.raw_arg).ok()?;
                Some(item_fn)
            }
            _ => None,
        })
        .map(|method: &syn::ItemFn| {
            let (ctx, args) = parse_args(method)?;
            let anchor_ident = ctx_accounts_ident(&ctx.raw_arg)?;
            let description = parse_description(method);
            Ok(Ix {
                raw_method: method.clone(),
                ident: method.sig.ident.clone(),
                description,
                args,
                anchor_ident,
            })
        })
        .collect::<ParseResult<Vec<Ix>>>()?;

    let fallback_fn = {
        let fallback_fns = mod_content
            .iter()
            .filter_map(|item| match item {
                syn::Item::Fn(item_fn) => {
                    let (ctx, _args) = parse_args(item_fn).ok()?;
                    if ctx_accounts_ident(&ctx.raw_arg).is_ok() {
                        return None;
                    }
                    Some(item_fn)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if fallback_fns.len() > 1 {
            return Err(ParseError::new(
                fallback_fns[0].span(),
                "More than one fallback function found",
            ));
        }
        fallback_fns
            .first()
            .map(|method: &&syn::ItemFn| FallbackFn {
                raw_method: (*method).clone(),
            })
    };

    Ok((ixs, fallback_fn))
}

pub fn parse_args(method: &syn::ItemFn) -> ParseResult<(IxArg, Vec<IxArg>)> {
    let mut args: Vec<IxArg> = method
        .sig
        .inputs
        .iter()
        .map(|arg: &syn::FnArg| match arg {
            syn::FnArg::Typed(arg) => {
                let ident = match &*arg.pat {
                    syn::Pat::Ident(ident) => &ident.ident,
                    _ => return Err(ParseError::new(arg.pat.span(), "expected argument name")),
                };
                Ok(IxArg {
                    name: ident.clone(),
                    raw_arg: arg.clone(),
                })
            }
            syn::FnArg::Receiver(_) => Err(ParseError::new(
                arg.span(),
                "expected a typed argument not self",
            )),
        })
        .collect::<ParseResult<_>>()?;

    // Remove the Context argument
    let ctx = args.remove(0);

    Ok((ctx, args))
}

pub fn parse_description(method: &syn::ItemFn) -> Option<String> {
    let description: Vec<String> = method
        .attrs
        .iter()
        .filter_map(|attr| match attr.parse_meta() {
            Ok(syn::Meta::NameValue(meta)) => {
                if meta.path.is_ident("doc") {
                    match meta.lit {
                        syn::Lit::Str(lit) => Some(lit.value().trim().to_string()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();

    if description.is_empty() {
        None
    } else {
        Some(description.join(" "))
    }
}
