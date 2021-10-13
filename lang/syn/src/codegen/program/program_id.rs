use quote::quote;

use crate::Program;

pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    let program_name = &program.name.to_string();
    let program_id = program_id_from_anchor_toml(program_name)
        .or_else(|| program_id_from_target_slash_deploy_dir(program_name))
        .unwrap_or_else(|| {
            panic!("Expected to find {}'s program_id in your Anchor.toml file's program section or in ./target/deploy/{}-keypair.json", program_name, program_name);
        });
    quote! {
        ::anchor_lang::declare_id!(#program_id);
    }
}

fn program_id_from_anchor_toml(program_name: &str) -> Option<String> {
    let anchor_toml = std::fs::read_to_string("./Anchor.toml")
        .expect("Expected to find an Anchor.toml file in the root of your project.")
        .parse::<toml::Value>()
        .expect("Expected your Anchor.toml file to be valid TOML.");

    let provider = anchor_toml.get("provider")?;
    let cluster = provider
        .get("cluster")?
        .as_str()
        .expect("Your Anchor.toml [provider]'s cluster field should be a string.");

    let program_id = anchor_toml
        .get("programs")?
        .get(cluster)?
        .get(program_name)?
        .as_str()
        .unwrap_or_else(|| {
            panic!(
                "Expected your Anchor.toml's [programs.{}] {} field to be a string.",
                cluster, program_name
            )
        });

    Some(program_id.to_string())
}

fn program_id_from_target_slash_deploy_dir(program_name: &str) -> Option<String> {
    let path = format!("./target/deploy/{}-keypair.json", program_name);
    let bytes: Vec<u8> = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let pubkey_bytes = &bytes[bytes.len() - 32..];
    let pubkey = bs58::encode(pubkey_bytes).into_string();
    Some(pubkey)
}
