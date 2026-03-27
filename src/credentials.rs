use anyhow::{bail, Result};
use keyring::Entry;

const SERVICE: &str = "scaffold";
const ACCOUNT: &str = "api-token";

pub fn load_token() -> Result<String> {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| anyhow::anyhow!("Failed to access credential store: {}", e))?;
    match entry.get_password() {
        Ok(token) => Ok(token),
        Err(keyring::Error::NoEntry) => {
            bail!("No token stored. Run: scaffold login")
        }
        Err(e) => bail!("Failed to read token from credential store: {}", e),
    }
}

pub fn store_token(token: &str) -> Result<()> {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| anyhow::anyhow!("Failed to access credential store: {}", e))?;
    entry
        .set_password(token)
        .map_err(|e| anyhow::anyhow!("Failed to store token: {}", e))
}

pub fn clear_token() -> Result<()> {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| anyhow::anyhow!("Failed to access credential store: {}", e))?;
    match entry.delete_password() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => bail!("Failed to clear token: {}", e),
    }
}
