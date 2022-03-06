use syn::AttrStyle::Inner;
use syn::{Meta::NameValue, MetaNameValue, Lit::Str};

// Returns first match or None
pub fn parse_inner(attrs: &Vec<syn::Attribute>) -> Option<String> {
    for attr in attrs {
        if let syn::Attribute {
            style: Inner(..),
            ..
        } = attr {
            for segment in &attr.path.segments {
                if segment.ident.to_string() == "doc" {
                    let meta = attr.parse_meta();
                    if let Ok(NameValue(
                        MetaNameValue {
                            lit: Str(lit_str),
                            ..
                    })) = meta {
                        return Some(lit_str.value().trim().to_string());
                    }
                }
            }
        }
    }
    None
}

// Returns first match or None
pub fn parse_any(attrs: &Vec<syn::Attribute>) -> Option<String> {
    for attr in attrs {
        for segment in &attr.path.segments {
            if segment.ident.to_string() == "doc" {
                let meta = attr.parse_meta();
                if let Ok(NameValue(
                    MetaNameValue {
                        lit: Str(lit_str),
                        ..
                })) = meta {
                    return Some(lit_str.value().trim().to_string());
                }
            }
        }
    }
    None
}
