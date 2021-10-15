use quote::quote;

use crate::Program;

pub fn generate(program: &Program) -> proc_macro2::TokenStream {
    eprintln!("Hello from the program_id macro!!!!");

    let program_name = &program.name.to_string();

    // Let's first look for the program's program_id in Anchor.toml.
    let anchor_toml = std::fs::read_to_string("./Anchor.toml")
        .expect("Expected to find an Anchor.toml file in the root of your project.")
        .parse::<toml::Value>()
        .expect("Expected your Anchor.toml file to be valid TOML.");

    // If you (or anchor build) have set the ANCHOR_CLUSTER env var, use it;
    // otherwise use Anchor.toml's provider > cluster field.
    let cluster = std::env::var("ANCHOR_CLUSTER")
        .ok()
        .or_else(|| {
            let anchor_toml_cluster = anchor_toml
                .get("provider")?
                .get("cluster")?
                .as_str()
                .expect("Your Anchor.toml [provider]'s cluster field should be a string.");
            Some(anchor_toml_cluster.to_string())
        })
        .unwrap_or("localnet".to_string());

    // Now that we know which cluster to use, go look for the corresponding
    // program_id in Anchor.toml.
    let anchor_toml_program_id = anchor_toml
        .get("programs")
        .and_then(|programs| programs.get(&cluster))
        .and_then(|cluster| cluster.get(program_name))
        .and_then(|program_id| {
            let anchor_toml_program_id = program_id.as_str().unwrap_or_else(|| {
                panic!(
                    "Expected your Anchor.toml's [programs.{}] {} field to be a string.",
                    cluster, program_name
                );
            });
            Some(anchor_toml_program_id.to_string())
        });

    // If we were able to find a suitable program_id in Anchor.toml, use it;
    // otherwise fall back to pubkey in the ./target/deploy directory.
    let program_id = anchor_toml_program_id
        .or_else(|| {
            let path = format!("./target/deploy/{}-keypair.json", program_name);
            // Hack: ideally we would use solana-sdk's
            // Keypair::read_keypair_file, but I can't seem to get it to work as
            // a dependency for this crate :/ So just parse the pubkey manually.
            let bytes: Vec<u8> = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
            let pubkey_bytes = &bytes[bytes.len() - 32..];
            let pubkey = bs58::encode(pubkey_bytes).into_string();
            Some(pubkey)
        })
        .unwrap_or_else(|| {
            panic!("Expected to find {}'s program_id in your Anchor.toml file's [program.{}] section or in ./target/deploy/{}-keypair.json", program_name, cluster, program_name);
        });

    eprintln!("program_id = {}", program_id);
    quote! {
        ::anchor_lang::declare_id!(#program_id);
    }
}
