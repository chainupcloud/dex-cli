use anyhow::Result;
use clap::{Args, Subcommand};

use crate::auth::{self, Identity};
use crate::client::gateway::GatewayClient;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: AdminCommand,
}

#[derive(Subcommand)]
pub enum AdminCommand {
    /// Create a new perpetual contract
    Setup {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        ticker: String,
        #[arg(long)]
        atomic_resolution: i32,
        #[arg(long, default_value = "50000")]
        initial_margin_ppm: u32,
        #[arg(long, default_value = "-8")]
        quantum_conversion_exponent: i32,
        #[arg(long, default_value = "1000")]
        subticks_per_tick: u64,
        #[arg(long, default_value = "10000000")]
        step_base_quantums: u64,
    },
    /// Update oracle prices
    OracleUpdate {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        price: f64,
        #[arg(long)]
        exponent: i32,
    },
    /// Trigger funding rate update
    FundingUpdate {
        #[arg(long)]
        perpetual_id: i32,
    },
    /// Liquidate a subaccount
    Liquidate {
        #[arg(long)]
        target: String,
        #[arg(long, default_value = "0")]
        subaccount: u32,
    },
    /// Setup MegaVault for a market
    SetupVault {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        allocation: Option<f64>,
    },
    /// Update perpetual contract parameters
    UpdateParams {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        initial_margin_ppm: Option<u32>,
    },
}

pub async fn execute(
    gateway: &GatewayClient,
    args: AdminArgs,
    output: OutputFormat,
    identity: &Identity,
) -> Result<()> {
    auth::require_identity(identity)?;

    let sender_index = identity.sender_index();

    match args.command {
        AdminCommand::Setup {
            perpetual_id,
            ticker,
            atomic_resolution,
            initial_margin_ppm,
            quantum_conversion_exponent,
            subticks_per_tick,
            step_base_quantums,
        } => {
            let mut body = serde_json::json!({
                "perpetual_id": perpetual_id,
                "ticker": ticker,
                "atomic_resolution": atomic_resolution,
                "initial_margin_ppm": initial_margin_ppm,
                "quantum_conversion_exponent": quantum_conversion_exponent,
                "subticks_per_tick": subticks_per_tick,
                "step_base_quantums": step_base_quantums,
            });
            if let Some(idx) = sender_index {
                body["sender_index"] = serde_json::json!(idx);
            }
            let resp = gateway.setup_perpetual(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
        AdminCommand::OracleUpdate {
            perpetual_id,
            price,
            exponent,
        } => {
            let mut body = serde_json::json!({
                "price_updates": [{
                    "perpetual_id": perpetual_id,
                    "price": price,
                    "exponent": exponent,
                }],
            });
            if let Some(idx) = sender_index {
                body["sender_index"] = serde_json::json!(idx);
            }
            let resp = gateway.update_oracle_prices(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
        AdminCommand::FundingUpdate { perpetual_id } => {
            let mut body = serde_json::json!({
                "perpetual_id": perpetual_id,
            });
            if let Some(idx) = sender_index {
                body["sender_index"] = serde_json::json!(idx);
            }
            let resp = gateway.update_funding_rates(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
        AdminCommand::Liquidate { target, subaccount } => {
            let mut body = serde_json::json!({
                "target_address": target,
                "subaccount_number": subaccount,
            });
            if let Some(idx) = sender_index {
                body["sender_index"] = serde_json::json!(idx);
            }
            let resp = gateway.liquidate(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
        AdminCommand::SetupVault {
            perpetual_id,
            allocation,
        } => {
            let mut body = serde_json::json!({
                "perpetual_id": perpetual_id,
            });
            if let Some(a) = allocation {
                body["allocation_amount"] = serde_json::json!(a);
            }
            if let Some(idx) = sender_index {
                body["sender_index"] = serde_json::json!(idx);
            }
            let resp = gateway.setup_vault(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
        AdminCommand::UpdateParams {
            perpetual_id,
            initial_margin_ppm,
        } => {
            let mut body = serde_json::json!({
                "perpetual_id": perpetual_id,
            });
            if let Some(margin) = initial_margin_ppm {
                body["initial_margin_ppm"] = serde_json::json!(margin);
            }
            if let Some(idx) = sender_index {
                body["sender_index"] = serde_json::json!(idx);
            }
            let resp = gateway.update_perpetual_params(body).await?;
            crate::output::print_gateway_result(&resp, output);
        }
    }
    Ok(())
}
