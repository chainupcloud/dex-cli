use anyhow::Result;
use std::io::{self, BufRead, Write};

use crate::config;

pub fn execute() -> Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("DEX CLI Setup Wizard");
    println!("====================\n");

    let mut cfg = config::load_config().unwrap_or_default();

    eprint!("dex-api URL [{}]: ", cfg.api_url.as_deref().unwrap_or("http://127.0.0.1:9100"));
    io::stderr().flush()?;
    if let Some(Ok(line)) = lines.next() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            cfg.api_url = Some(trimmed.to_string());
        }
    }

    eprint!("tx-gateway URL [{}]: ", cfg.gateway_url.as_deref().unwrap_or("http://127.0.0.1:3200"));
    io::stderr().flush()?;
    if let Some(Ok(line)) = lines.next() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            cfg.gateway_url = Some(trimmed.to_string());
        }
    }

    eprint!("Identity mode (sender_index / private_key) [sender_index]: ");
    io::stderr().flush()?;
    if let Some(Ok(line)) = lines.next() {
        let trimmed = line.trim();
        if trimmed == "private_key" {
            eprint!("Private key (base64): ");
            io::stderr().flush()?;
            if let Some(Ok(key_line)) = lines.next() {
                let key = key_line.trim();
                if !key.is_empty() {
                    cfg.private_key = Some(key.to_string());
                    cfg.sender_index = None;
                }
            }
        } else {
            eprint!("Sender index [0]: ");
            io::stderr().flush()?;
            if let Some(Ok(idx_line)) = lines.next() {
                let idx = idx_line.trim();
                let idx: u32 = if idx.is_empty() { 0 } else { idx.parse()? };
                cfg.sender_index = Some(idx);
                cfg.private_key = None;
            }
        }
    }

    eprint!("Default subaccount [0]: ");
    io::stderr().flush()?;
    if let Some(Ok(line)) = lines.next() {
        let trimmed = line.trim();
        let sub: u32 = if trimmed.is_empty() { 0 } else { trimmed.parse()? };
        cfg.default_subaccount = Some(sub);
    }

    config::save_config(&cfg)?;

    println!("\nConfiguration saved to {}", config::config_path()?.display());
    println!("Run 'dex status api' to verify connectivity.");

    Ok(())
}
