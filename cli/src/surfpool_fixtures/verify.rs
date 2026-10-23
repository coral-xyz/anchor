use {
    crate::{CommitmentConfig, Pubkey, RpcClient},
    anyhow::{bail, Context, Result},
    base64::{engine::general_purpose::STANDARD, Engine},
    serde_json::{Map, Value},
    std::{fs, path::Path},
};

/// Surfpool logs snapshot failures instead of aborting. Never run tests with a
/// fixture that was skipped or replaced by a remote account.
pub(crate) fn verify(client: &RpcClient, flags: &[String]) -> Result<()> {
    // surfpool_flags puts Anchor's compiled snapshot first, before ordinary
    // options and extra_args. User snapshots can include remote/null entries.
    if flags.first().map(String::as_str) != Some("--snapshot") {
        return Ok(());
    }
    {
        let path = Path::new(flags.get(1).context("compiled fixture snapshot path")?);
        let images: Map<String, Value> = serde_json::from_slice(&fs::read(path)?)
            .with_context(|| format!("Read Surfpool fixture snapshot {}", path.display()))?;
        for (address, image) in images {
            let key: Pubkey = address.parse()?;
            let actual = client
                .get_account_with_commitment(&key, CommitmentConfig::processed())?
                .value;
            let expected = STANDARD.decode(image["data"].as_str().context("fixture data")?)?;
            let matches = actual.is_some_and(|account| {
                image["lamports"].as_u64() == Some(account.lamports)
                    && image["owner"].as_str() == Some(account.owner.to_string().as_str())
                    && image["executable"].as_bool() == Some(account.executable)
                    && account.data == expected
            });
            if !matches {
                bail!(
                    "FIXTURE_MISMATCH: {address} differs from its declared image after startup: {}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}
