use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use clap::{Args, Subcommand};
use std::io::{self, Write};

use crate::auth::{self, Identity};
use crate::client::gateway::GatewayClient;
use crate::config;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct WalletArgs {
    #[command(subcommand)]
    pub command: WalletCommand,
}

#[derive(Subcommand)]
pub enum WalletCommand {
    /// Create a new random secp256k1 keypair
    Create {
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
    },
    /// Import an existing private key (hex, with or without 0x prefix)
    Import {
        /// Hex-encoded secp256k1 private key
        key: String,
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
    },
    /// Show current address
    Address,
    /// Show full wallet info (does not reveal private key)
    Show,
    /// Delete wallet configuration
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Request test SUI from faucet (sender-index mode only)
    Faucet,
}

pub async fn execute(
    gateway: &GatewayClient,
    args: WalletArgs,
    output: OutputFormat,
    identity: &Identity,
) -> Result<()> {
    match args.command {
        WalletCommand::Create { force } => {
            let cfg = config::load_config()?;
            if cfg.private_key.is_some() && !force {
                anyhow::bail!(
                    "Wallet already exists. Use '--force' to overwrite.\n\
                     Current address: {}",
                    cfg.address.as_deref().unwrap_or("unknown")
                );
            }

            let signer = PrivateKeySigner::random();
            let address = signer.address();
            let key_hex = format!("0x{}", hex::encode(signer.to_bytes()));

            let mut cfg = config::load_config().unwrap_or_default();
            cfg.private_key = Some(key_hex.clone());
            cfg.address = Some(format!("{address}"));
            cfg.sender_index = None; // private key takes precedence
            config::save_config(&cfg)?;

            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({
                        "address": format!("{address}"),
                        "config_path": config::config_path()?.display().to_string(),
                    }));
                }
                OutputFormat::Table => {
                    println!("Wallet created successfully.");
                    println!("  Address: {address}");
                    println!("  Config:  {}", config::config_path()?.display());
                    println!();
                    println!("  IMPORTANT: Back up your private key from the config file.");
                    println!("  It will not be shown again.");
                }
            }
        }
        WalletCommand::Import { key, force } => {
            let cfg = config::load_config()?;
            if cfg.private_key.is_some() && !force {
                anyhow::bail!("Wallet already exists. Use '--force' to overwrite.");
            }

            let signer = auth::create_signer(&key)?;
            let address = signer.address();
            // Normalize to 0x-prefixed hex
            let normalized = if key.starts_with("0x") {
                key
            } else {
                format!("0x{key}")
            };

            let mut cfg = config::load_config().unwrap_or_default();
            cfg.private_key = Some(normalized);
            cfg.address = Some(format!("{address}"));
            cfg.sender_index = None;
            config::save_config(&cfg)?;

            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({
                        "address": format!("{address}"),
                    }));
                }
                OutputFormat::Table => {
                    println!("Wallet imported successfully.");
                    println!("  Address: {address}");
                }
            }
        }
        WalletCommand::Address => {
            let addr = auth::resolve_address(identity, gateway).await?;
            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({"address": addr}));
                }
                OutputFormat::Table => {
                    println!("{addr}");
                }
            }
        }
        WalletCommand::Show => {
            let cfg = config::load_config()?;
            let key_source = match identity {
                Identity::SenderIndex(idx) => format!("sender_index: {idx}"),
                Identity::PrivateKey(_) => "private_key (secp256k1)".to_string(),
                Identity::None => "none".to_string(),
            };
            let address = match auth::resolve_address(identity, gateway).await {
                Ok(addr) => addr,
                Err(_) => cfg.address.clone().unwrap_or_else(|| "not configured".to_string()),
            };
            let signing_mode = match identity {
                Identity::PrivateKey(_) => "EIP-712 → POST /exchange",
                Identity::SenderIndex(_) => "tx-gateway → POST /tx/*",
                Identity::None => "none",
            };

            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({
                        "address": address,
                        "key_source": key_source,
                        "signing_mode": signing_mode,
                        "config_path": config::config_path()?.display().to_string(),
                        "api_url": cfg.api_url,
                        "gateway_url": cfg.gateway_url,
                    }));
                }
                OutputFormat::Table => {
                    crate::output::print_detail_table(&[
                        ("Address", address),
                        ("Key Source", key_source),
                        ("Signing Mode", signing_mode.to_string()),
                        ("Config Path", config::config_path()?.display().to_string()),
                        ("API URL", cfg.api_url.unwrap_or_else(|| "(default)".to_string())),
                        ("Gateway URL", cfg.gateway_url.unwrap_or_else(|| "(default)".to_string())),
                    ]);
                }
            }
        }
        WalletCommand::Reset { force } => {
            if !force {
                eprint!("Delete wallet configuration? This cannot be undone. [y/N] ");
                io::stderr().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            config::delete_config()?;
            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({"status": "deleted"}));
                }
                OutputFormat::Table => {
                    println!("Wallet configuration deleted.");
                }
            }
        }
        WalletCommand::Faucet => {
            auth::require_identity(identity)?;
            let mut body = serde_json::json!({});
            if let Identity::SenderIndex(idx) = identity {
                body["sender_index"] = serde_json::json!(idx);
            }
            let resp = gateway.faucet(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
    }
    Ok(())
}
