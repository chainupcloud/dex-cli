use anyhow::Result;
use clap::{Args, Subcommand};

use crate::auth::{self, Identity};
use crate::client::gateway::GatewayClient;
use crate::client::info::InfoClient;
use crate::config;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub command: AccountCommand,
}

#[derive(Subcommand)]
pub enum AccountCommand {
    /// Show account overview (balance, margin, positions)
    Info,
    /// Show fill/trade history
    Fills {
        #[arg(long)]
        perpetual_id: Option<i32>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Show balance change history
    Balances,
    /// Show transfer history
    Transfers,
    /// Deposit USDC via cross-chain bridge (private-key) or gateway (sender-index)
    Deposit {
        /// Amount of USDC to deposit (6 decimals, e.g. 1000 = 1000 USDC)
        #[arg(long)]
        amount: f64,
    },
    /// Withdraw USDC from subaccount
    Withdraw {
        #[arg(long)]
        amount: f64,
    },
    /// Mint test USDC (sender-index mode, devnet only)
    MintUsdc {
        #[arg(long)]
        amount: f64,
    },
    /// Check USDC balance on EVM chain (private-key mode)
    EvmBalance,
}

fn build_identity_fields(identity: &Identity, subaccount: u32) -> serde_json::Value {
    match identity {
        Identity::SenderIndex(idx) => serde_json::json!({
            "sender_index": idx,
            "subaccount_number": subaccount,
        }),
        Identity::PrivateKey(_) => {
            let cfg = config::load_config().unwrap_or_default();
            serde_json::json!({
                "owner": cfg.address.unwrap_or_default(),
                "subaccount_number": subaccount,
            })
        }
        Identity::None => serde_json::json!({}),
    }
}

pub async fn execute(
    info: &InfoClient,
    gateway: &GatewayClient,
    args: AccountArgs,
    output: OutputFormat,
    identity: &Identity,
    subaccount: u32,
) -> Result<()> {
    auth::require_identity(identity)?;

    match args.command {
        AccountCommand::Info => {
            let address = auth::resolve_address(identity, gateway).await?;
            let state = info.clearinghouse_state(&address, subaccount).await?;
            crate::output::account::print_account_info(&address, subaccount, &state, output);
        }
        AccountCommand::Fills {
            perpetual_id,
            limit,
        } => {
            let address = auth::resolve_address(identity, gateway).await?;
            let fills = info.user_fills(&address, perpetual_id, limit).await?;
            crate::output::account::print_fills(&fills, output);
        }
        AccountCommand::Balances => {
            let address = auth::resolve_address(identity, gateway).await?;
            let balances = info.user_balances(&address).await?;
            crate::output::account::print_balances(&balances, output);
        }
        AccountCommand::Transfers => {
            let address = auth::resolve_address(identity, gateway).await?;
            let transfers = info.user_transfers(&address).await?;
            crate::output::account::print_transfers(&transfers, output);
        }
        AccountCommand::Deposit { amount } => {
            match identity {
                Identity::PrivateKey(_) => {
                    // Cross-chain bridge deposit
                    let signer = auth::resolve_signer(identity)?;
                    let sui_addr = auth::derive_sui_address(&signer);
                    let sui_addr_hex = auth::derive_sui_address_hex(&signer);

                    let cfg = config::load_config()?;
                    let eth_rpc = cfg
                        .eth_rpc_url
                        .as_deref()
                        .unwrap_or(config::DEFAULT_ETH_RPC_URL);
                    let bridge_addr = cfg
                        .bridge_address
                        .as_deref()
                        .unwrap_or(config::DEFAULT_BRIDGE_ADDRESS_DEV);
                    let usdc_addr = cfg
                        .usdc_address
                        .as_deref()
                        .unwrap_or(config::DEFAULT_USDC_ADDRESS);

                    let bridge =
                        crate::client::bridge::BridgeClient::new(eth_rpc, bridge_addr, usdc_addr)?;

                    // USDC has 6 decimals
                    let amount_raw = (amount * 1_000_000.0) as u64;

                    let result = bridge
                        .deposit(&signer, sui_addr, subaccount, amount_raw)
                        .await?;

                    match output {
                        OutputFormat::Json => {
                            crate::output::print_json(&serde_json::json!({
                                "success": true,
                                "tx_hash": result.tx_hash,
                                "amount_usdc": amount,
                                "sui_address": result.sui_address,
                                "subaccount": result.subaccount,
                                "message": "Bridge deposit submitted. Funds will arrive in 2-10 minutes.",
                            }));
                        }
                        OutputFormat::Table => {
                            println!("✓ Bridge deposit submitted");
                            println!("  Amount:       {amount} USDC");
                            println!("  EVM tx:       {}", result.tx_hash);
                            println!("  Sui address:  {sui_addr_hex}");
                            println!("  Subaccount:   {subaccount}");
                            println!();
                            println!("  Funds will arrive in ~2-10 minutes after bridge node confirmation.");
                        }
                    }
                }
                Identity::SenderIndex(_) => {
                    // Gateway test deposit (mint + deposit directly)
                    let mut body = build_identity_fields(identity, subaccount);
                    body["amount"] = serde_json::json!(amount);
                    let resp = gateway.deposit(body).await?;
                    crate::output::print_gateway_result(&resp, output);
                }
                Identity::None => unreachable!(),
            }
        }
        AccountCommand::Withdraw { amount } => {
            let mut body = build_identity_fields(identity, subaccount);
            body["amount"] = serde_json::json!(amount);
            let resp = gateway.withdraw(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
        AccountCommand::MintUsdc { amount } => {
            let mut body = build_identity_fields(identity, subaccount);
            body["amount"] = serde_json::json!(amount);
            let resp = gateway.mint_usdc(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
        AccountCommand::EvmBalance => {
            let signer = auth::resolve_signer(identity)?;
            let cfg = config::load_config()?;
            let eth_rpc = cfg
                .eth_rpc_url
                .as_deref()
                .unwrap_or(config::DEFAULT_ETH_RPC_URL);
            let bridge_addr = cfg
                .bridge_address
                .as_deref()
                .unwrap_or(config::DEFAULT_BRIDGE_ADDRESS_DEV);
            let usdc_addr = cfg
                .usdc_address
                .as_deref()
                .unwrap_or(config::DEFAULT_USDC_ADDRESS);

            let bridge =
                crate::client::bridge::BridgeClient::new(eth_rpc, bridge_addr, usdc_addr)?;
            let balance = bridge.usdc_balance(&signer).await?;
            let balance_human = balance as f64 / 1_000_000.0;

            match output {
                OutputFormat::Json => {
                    crate::output::print_json(&serde_json::json!({
                        "address": format!("{}", signer.address()),
                        "usdc_balance": balance_human,
                        "usdc_raw": balance,
                    }));
                }
                OutputFormat::Table => {
                    println!("EVM Address: {}", signer.address());
                    println!("USDC Balance: {balance_human} USDC ({balance} raw)");
                }
            }
        }
    }
    Ok(())
}
