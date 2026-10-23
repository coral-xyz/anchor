use {
    super::insert,
    crate::{
        config::{Config, Validator, WithPath},
        resolve_workspace_path, Pubkey,
    },
    anyhow::{anyhow, bail, Context, Result},
    base64::{engine::general_purpose::STANDARD, Engine},
    serde_json::{json, Map, Value},
    std::{fs, path::Path},
};

fn read(path: &Path) -> Result<Value> {
    let file =
        fs::File::open(path).with_context(|| format!("Open account fixture {}", path.display()))?;
    serde_json::from_reader(file)
        .with_context(|| format!("Parse account fixture {}", path.display()))
}

pub(super) fn image(value: &Value) -> Result<Value> {
    let account = value.get("account").unwrap_or(value);
    let owner = account["owner"]
        .as_str()
        .ok_or_else(|| anyhow!("missing owner"))?;
    let _: Pubkey = owner.parse().context("invalid account owner")?;
    let lamports = account["lamports"]
        .as_u64()
        .ok_or_else(|| anyhow!("missing/invalid lamports"))?;
    let executable = account["executable"]
        .as_bool()
        .ok_or_else(|| anyhow!("missing executable"))?;
    let rent_epoch = account["rentEpoch"]
        .as_u64()
        .ok_or_else(|| anyhow!("missing/invalid rentEpoch"))?;
    let data = &account["data"];
    let bytes = match data {
        Value::String(text) => STANDARD.decode(text).context("invalid base64 data")?,
        Value::Array(parts) if parts.len() == 2 => {
            let text = parts[0]
                .as_str()
                .ok_or_else(|| anyhow!("invalid account data"))?;
            match parts[1].as_str() {
                Some("base64") => STANDARD.decode(text).context("invalid base64 data")?,
                Some("base58") => bs58::decode(text)
                    .into_vec()
                    .context("invalid base58 data")?,
                _ => bail!("account fixture data must use base64 or base58 encoding"),
            }
        }
        _ => bail!("account fixture needs binary data, not jsonParsed data"),
    };
    if bytes.len() > 10 * 1024 * 1024 {
        bail!("account fixture data exceeds Solana's 10 MiB account limit");
    }
    Ok(
        json!({"owner":owner, "lamports":lamports, "executable":executable,
        "rentEpoch":rent_epoch, "data":STANDARD.encode(bytes), "parsedData":null}),
    )
}

pub(super) fn load(
    cfg: &WithPath<Config>,
    validator: &Validator,
    images: &mut Map<String, Value>,
) -> Result<()> {
    for account in validator.account.iter().flatten() {
        let path = resolve_workspace_path(cfg, &account.filename)?;
        let value = read(&path)?;
        let address = if account.address == "-" {
            value["pubkey"]
                .as_str()
                .ok_or_else(|| anyhow!("Account fixture {} has no pubkey", path.display()))?
        } else {
            &account.address
        };
        let address: Pubkey = address
            .parse()
            .context("invalid configured account address")?;
        let loaded =
            image(&value).with_context(|| format!("Account fixture {}", path.display()))?;
        // As with --account ADDRESS FILE, the explicit address is authoritative.
        insert(images, address, loaded)?;
    }
    for entry in validator.account_dir.iter().flatten() {
        let path = resolve_workspace_path(cfg, &entry.directory)?;
        let mut files = fs::read_dir(&path)
            .with_context(|| format!("Read account directory {}", path.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        files.sort();
        for file in files {
            if file.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let value = read(&file)?;
            let address: Pubkey = value["pubkey"]
                .as_str()
                .ok_or_else(|| {
                    anyhow!("Account directory fixture {} has no pubkey", file.display())
                })?
                .parse()
                .with_context(|| format!("Invalid pubkey in {}", file.display()))?;
            let loaded =
                image(&value).with_context(|| format!("Account fixture {}", file.display()))?;
            insert(images, address, loaded)?;
        }
    }
    Ok(())
}
