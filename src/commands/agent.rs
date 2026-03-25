use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use clap::{Args, Subcommand};

use crate::auth;
use crate::client::exchange::ExchangeClient;
use crate::client::info::InfoClient;
use crate::config;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Authorize a new agent key (signs with master wallet)
    Approve {
        /// Agent's ETH address to authorize (if omitted, generates a new agent keypair)
        #[arg(long)]
        agent_address: Option<String>,
        /// Validity duration: "24h", "7d", "permanent" (default: permanent)
        #[arg(long, default_value = "permanent")]
        valid_until: String,
    },
    /// Revoke an agent key (signs with master wallet)
    Revoke {
        /// Agent's ETH address to revoke
        #[arg(long)]
        agent_address: String,
    },
    /// List authorized agents for current wallet
    List,
    /// Show current agent key info
    Show,
}

fn parse_valid_until(s: &str) -> Result<u64> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;

    match s {
        "permanent" | "0" => Ok(0),
        s if s.ends_with('h') => {
            let hours: u64 = s.trim_end_matches('h').parse()?;
            Ok(now_ms + hours * 3_600_000)
        }
        s if s.ends_with('d') => {
            let days: u64 = s.trim_end_matches('d').parse()?;
            Ok(now_ms + days * 86_400_000)
        }
        _ => anyhow::bail!("Invalid duration format. Use '24h', '7d', or 'permanent'."),
    }
}

fn eth_address_bytes(addr: &str) -> Result<[u8; 20]> {
    let hex_str = addr.strip_prefix("0x").unwrap_or(addr);
    let bytes = hex::decode(hex_str)?;
    anyhow::ensure!(bytes.len() == 20, "ETH address must be 20 bytes");
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

pub async fn execute(
    exchange: &ExchangeClient,
    _info: &InfoClient,
    args: AgentArgs,
    output: OutputFormat,
    identity: &auth::Identity,
) -> Result<()> {
    match args.command {
        AgentCommand::Approve {
            agent_address,
            valid_until,
        } => {
            let master_signer = auth::resolve_signer(identity)?;
            let valid_until_ms = parse_valid_until(&valid_until)?;

            // If no agent address provided, generate a new keypair
            let (agent_addr_bytes, agent_private_key) = match agent_address {
                Some(addr) => (eth_address_bytes(&addr)?, None),
                None => {
                    let agent_signer = PrivateKeySigner::random();
                    let addr = agent_signer.address();
                    let key_hex = format!("0x{}", hex::encode(agent_signer.to_bytes()));
                    let mut addr_bytes = [0u8; 20];
                    addr_bytes.copy_from_slice(addr.as_slice());
                    (addr_bytes, Some(key_hex))
                }
            };

            let resp = exchange
                .approve_agent(&master_signer, agent_addr_bytes, 0, valid_until_ms)
                .await?;

            // Save agent key to config if we generated it
            if let Some(ref key) = agent_private_key {
                let mut cfg = config::load_config()?;
                cfg.agent_key = Some(key.clone());
                cfg.agent_valid_until = Some(valid_until_ms);
                config::save_config(&cfg)?;
            }

            let agent_addr_hex = format!("0x{}", hex::encode(agent_addr_bytes));

            match output {
                OutputFormat::Json => {
                    let mut result = serde_json::json!({
                        "status": resp.status,
                        "master_address": format!("{}", master_signer.address()),
                        "agent_address": agent_addr_hex,
                        "valid_until_ms": valid_until_ms,
                    });
                    if let Some(ref key) = agent_private_key {
                        result["agent_private_key"] = serde_json::json!(key);
                        result["saved_to_config"] = serde_json::json!(true);
                    }
                    if let Some(ref r) = resp.response {
                        result["response"] = r.clone();
                    }
                    crate::output::print_json(&result);
                }
                OutputFormat::Table => {
                    println!("✓ Agent approved");
                    println!("  Master:  {}", master_signer.address());
                    println!("  Agent:   {agent_addr_hex}");
                    if valid_until_ms == 0 {
                        println!("  Expires: permanent");
                    } else {
                        println!("  Expires: {valid_until_ms}");
                    }
                    if agent_private_key.is_some() {
                        println!("  Agent key saved to config.");
                        println!();
                        println!("  You can now trade with: dex --agent-key order place ...");
                        println!("  Or configure in config.json and trade normally.");
                    }
                }
            }
        }
        AgentCommand::Revoke { agent_address } => {
            let master_signer = auth::resolve_signer(identity)?;
            let agent_addr_bytes = eth_address_bytes(&agent_address)?;

            let resp = exchange
                .revoke_agent(&master_signer, agent_addr_bytes)
                .await?;

            // Clear agent key from config if it matches
            let cfg = config::load_config()?;
            if let Some(ref stored_key) = cfg.agent_key {
                if let Ok(stored_signer) = auth::create_signer(stored_key) {
                    let stored_addr = stored_signer.address();
                    if stored_addr.as_slice() == &agent_addr_bytes {
                        let mut cfg = cfg;
                        cfg.agent_key = None;
                        cfg.agent_valid_until = None;
                        config::save_config(&cfg)?;
                    }
                }
            }

            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({
                        "status": resp.status,
                        "revoked_agent": agent_address,
                    }));
                }
                OutputFormat::Table => {
                    println!("✓ Agent revoked: {agent_address}");
                }
            }
        }
        AgentCommand::List => {
            let address = auth::resolve_address(identity, &crate::client::gateway::GatewayClient::new("")).await
                .or_else(|_| {
                    let signer = auth::resolve_signer(identity)?;
                    Ok::<String, anyhow::Error>(format!("{}", signer.address()))
                })?;

            // TODO: Query from dex-api when userAgents endpoint is available
            // For now, show locally configured agent
            let cfg = config::load_config()?;

            match output {
                OutputFormat::Json => {
                    if let Some(ref key) = cfg.agent_key {
                        if let Ok(signer) = auth::create_signer(key) {
                            crate::output::print_json(&serde_json::json!([{
                                "agent_address": format!("{}", signer.address()),
                                "valid_until_ms": cfg.agent_valid_until.unwrap_or(0),
                                "source": "config",
                            }]));
                        } else {
                            crate::output::print_json(&serde_json::json!([]));
                        }
                    } else {
                        crate::output::print_json(&serde_json::json!([]));
                    }
                }
                OutputFormat::Table => {
                    println!("Master: {address}");
                    if let Some(ref key) = cfg.agent_key {
                        if let Ok(signer) = auth::create_signer(key) {
                            let exp = cfg.agent_valid_until.unwrap_or(0);
                            let exp_str = if exp == 0 { "permanent".to_string() } else { exp.to_string() };
                            println!("Agent:  {} (expires: {exp_str})", signer.address());
                        }
                    } else {
                        println!("No agent configured. Run 'dex agent approve' to create one.");
                    }
                }
            }
        }
        AgentCommand::Show => {
            let cfg = config::load_config()?;

            match &cfg.agent_key {
                Some(key) => {
                    let signer = auth::create_signer(key)?;
                    let exp = cfg.agent_valid_until.unwrap_or(0);

                    match output {
                        OutputFormat::Json => {
                            crate::output::print_json(&serde_json::json!({
                                "agent_address": format!("{}", signer.address()),
                                "valid_until_ms": exp,
                                "master_address": cfg.address,
                            }));
                        }
                        OutputFormat::Table => {
                            crate::output::print_detail_table(&[
                                ("Agent Address", format!("{}", signer.address())),
                                ("Master Address", cfg.address.unwrap_or_else(|| "unknown".to_string())),
                                ("Valid Until", if exp == 0 { "permanent".to_string() } else { exp.to_string() }),
                                ("Config Path", config::config_path()?.display().to_string()),
                            ]);
                        }
                    }
                }
                None => {
                    match output {
                        OutputFormat::Json => {
                            crate::output::print_json(&serde_json::json!({"agent": null}));
                        }
                        OutputFormat::Table => {
                            println!("No agent key configured. Run 'dex agent approve' to create one.");
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
