use anyhow::{bail, Result};
use std::io::BufRead;

use crate::credentials;
use crate::settings::Settings;

pub fn run(token: Option<&str>, clear: bool) -> Result<()> {
    if clear && token.is_some() {
        bail!("--clear and --token cannot be used together.");
    }

    if clear {
        credentials::clear_token()?;
        println!("Logged out.");
        return Ok(());
    }

    let settings = Settings::load()?;
    if settings.api_gateway_url.is_none() {
        eprintln!(
            "Warning: api_gateway_url is not configured. \
             Run: scaffold config set api_gateway_url <url>"
        );
    }

    let token = match token {
        Some(t) => {
            eprintln!(
                "Warning: passing tokens via --token may expose them in shell history. \
                 Consider piping instead: echo <token> | scaffold login"
            );
            t.trim().to_string()
        }
        None => {
            eprint!("Paste your access token: ");
            let stdin = std::io::stdin();
            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if token.is_empty() {
        bail!("Token cannot be empty.");
    }

    credentials::store_token(&token)?;
    println!("Token stored.");
    Ok(())
}
