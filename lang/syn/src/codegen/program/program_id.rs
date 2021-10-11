use quote::quote;

use crate::Program;

pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    let anchor_toml = std::fs::read_to_string("./Anchor.toml")
        .map(|s| s.parse::<toml::Value>())
        .unwrap()
        .unwrap();
    let cluster = std::env::var("ANCHOR_CLUSTER").unwrap_or("localnet".to_string());
    let program_id = anchor_toml["programs"][cluster][&program.name.to_string()].to_string();
    // strip quotes, urgh
    let program_id = &program_id[1..program_id.len() - 1];
    quote! {
        ::anchor_lang::declare_id!(#program_id);
    }
}
