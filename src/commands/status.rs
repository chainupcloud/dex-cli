use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::gateway::GatewayClient;
use crate::client::info::InfoClient;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct StatusArgs {
    #[command(subcommand)]
    pub command: StatusCommand,
}

#[derive(Subcommand)]
pub enum StatusCommand {
    /// Check dex-api connectivity
    Api,
    /// Check tx-gateway connectivity and info
    Gateway,
}

pub async fn execute(
    info: &InfoClient,
    gateway: &GatewayClient,
    args: StatusArgs,
    output: OutputFormat,
) -> Result<()> {
    match args.command {
        StatusCommand::Api => {
            match info.health_check().await {
                Ok(()) => {
                    match output {
                        OutputFormat::Json => {
                            println!("{}", serde_json::json!({"status": "connected"}));
                        }
                        OutputFormat::Table => {
                            println!("dex-api: Connected ✓");
                        }
                    }
                }
                Err(e) => {
                    match output {
                        OutputFormat::Json => {
                            println!("{}", serde_json::json!({"status": "unreachable", "error": format!("{e:#}")}));
                        }
                        OutputFormat::Table => {
                            println!("dex-api: Unreachable ✗");
                            eprintln!("  {e:#}");
                        }
                    }
                }
            }
            Ok(())
        }
        StatusCommand::Gateway => {
            match gateway.status().await {
                Ok(status) => {
                    match output {
                        OutputFormat::Json => {
                            crate::output::print_json(&status);
                        }
                        OutputFormat::Table => {
                            println!("tx-gateway: Connected ✓");
                            if let Some(ref addr) = status.address {
                                println!("  Address: {addr}");
                            }
                        }
                    }
                }
                Err(e) => {
                    match output {
                        OutputFormat::Json => {
                            println!("{}", serde_json::json!({"status": "unreachable", "error": format!("{e:#}")}));
                        }
                        OutputFormat::Table => {
                            println!("tx-gateway: Unreachable ✗");
                            eprintln!("  {e:#}");
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
