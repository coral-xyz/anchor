use syn::{Lit::Str, Meta::NameValue};


// Returns space separated doc comments
pub fn parse(attrs: &[syn::Attribute]) -> Option<String> {
    let doc_strings: Vec<String> = attrs
        .iter()
        .filter_map(|attr| match attr.parse_meta() {
            Ok(NameValue(meta)) => {
                if meta.path.is_ident("doc") {
                    if let Str(doc) = meta.lit {
                        return Some(doc.value().trim().to_string());
                    }
                }
                return None;
            },
            _ => None,
        })
        .collect();
    if doc_strings.is_empty() {
        None
    } else {
        Some(doc_strings.join(" "))
    }
}
