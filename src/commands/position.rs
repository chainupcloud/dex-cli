use anyhow::Result;
use clap::{Args, Subcommand};

use crate::auth::{self, Identity};
use crate::client::exchange::ExchangeClient;
use crate::client::gateway::GatewayClient;
use crate::client::info::InfoClient;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct PositionArgs {
    #[command(subcommand)]
    pub command: PositionCommand,
}

#[derive(Subcommand)]
pub enum PositionCommand {
    /// List all open positions
    List,
    /// Close a position
    Close {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        size: Option<f64>,
        #[arg(long)]
        worst_price: f64,
    },
    /// Set position leverage
    Leverage {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        leverage: u32,
    },
}

pub async fn execute(
    info: &InfoClient,
    gateway: &GatewayClient,
    args: PositionArgs,
    output: OutputFormat,
    identity: &Identity,
    subaccount: u32,
    exchange: &ExchangeClient,
) -> Result<()> {
    auth::require_identity(identity)?;

    match args.command {
        PositionCommand::List => {
            let address = auth::resolve_address(identity, gateway).await?;
            let state = info.clearinghouse_state(&address, subaccount).await?;
            crate::output::position::print_positions(&state.asset_positions, output);
        }
        PositionCommand::Close {
            perpetual_id,
            worst_price,
            ..
        } => {
            match identity {
                Identity::PrivateKey(_) => {
                    let signer = auth::resolve_signer(identity)?;
                    let resp = exchange
                        .close_position(
                            &signer,
                            perpetual_id as u32,
                            &worst_price.to_string(),
                            subaccount,
                        )
                        .await?;
                    crate::output::print_json(&resp);
                }
                Identity::SenderIndex(idx) => {
                    let body = serde_json::json!({
                        "sender_index": idx,
                        "perpetual_id": perpetual_id,
                        "worst_price": worst_price,
                        "subaccount_number": subaccount,
                    });
                    let resp = gateway.close_position(body).await?;
                    crate::output::print_gateway_result(&resp, output);
                }
                Identity::None => unreachable!(),
            }
        }
        PositionCommand::Leverage {
            perpetual_id,
            leverage,
        } => {
            match identity {
                Identity::PrivateKey(_) => {
                    let signer = auth::resolve_signer(identity)?;
                    let resp = exchange
                        .update_leverage(&signer, perpetual_id as u32, true, leverage, subaccount)
                        .await?;
                    crate::output::print_json(&resp);
                }
                Identity::SenderIndex(idx) => {
                    let body = serde_json::json!({
                        "sender_index": idx,
                        "perpetual_id": perpetual_id,
                        "leverage": leverage,
                        "subaccount_number": subaccount,
                    });
                    let resp = gateway.set_leverage(body).await?;
                    crate::output::print_gateway_result(&resp, output);
                }
                Identity::None => unreachable!(),
            }
        }
    }
    Ok(())
}
