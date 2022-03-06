use syn::AttrStyle::Inner;
use syn::{Meta::NameValue, Lit::Str};

// Returns first match or None
pub fn parse_inner(attrs: &Vec<syn::Attribute>) -> Option<String> {
    for attr in attrs {
        if let syn::Attribute {
            style: Inner(..),
            ..
        } = attr {
            let meta_result = attr.parse_meta();
            if let Ok(NameValue(meta)) = meta_result {
                if meta.path.is_ident("doc") {
                    if let Str(doc) = meta.lit {
                        return Some(doc.value().trim().to_string());
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
        let meta_result = attr.parse_meta();
        if let Ok(NameValue(meta)) = meta_result {
            if meta.path.is_ident("doc") {
                if let Str(doc) = meta.lit {
                    return Some(doc.value().trim().to_string());
                }
            }
        }
    }
    None
}
