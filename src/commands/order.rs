use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};

use crate::auth::{self, Identity};
use crate::client::exchange::ExchangeClient;
use crate::client::gateway::GatewayClient;
use crate::client::info::InfoClient;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct OrderArgs {
    #[command(subcommand)]
    pub command: OrderCommand,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Subcommand)]
pub enum OrderCommand {
    /// Place a new order
    Place {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        side: Side,
        #[arg(long)]
        quantity: f64,
        /// Limit price (omit for market order)
        #[arg(long)]
        price: Option<f64>,
        #[arg(long, default_value = "limit")]
        order_type: OrderType,
        #[arg(long, default_value = "gtc")]
        time_in_force: TimeInForce,
        #[arg(long)]
        reduce_only: bool,
        #[arg(long)]
        client_id: Option<u64>,
    },
    /// Cancel an order
    Cancel {
        #[arg(long)]
        perpetual_id: i32,
        #[arg(long)]
        client_id: u64,
    },
    /// List open orders
    List {
        #[arg(long)]
        perpetual_id: Option<i32>,
    },
    /// Show order history
    History {
        #[arg(long)]
        perpetual_id: Option<i32>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Check status of a single order
    Status {
        order_id: String,
    },
}

pub async fn execute(
    info: &InfoClient,
    gateway: &GatewayClient,
    args: OrderArgs,
    output: OutputFormat,
    identity: &Identity,
    subaccount: u32,
    exchange: &ExchangeClient,
) -> Result<()> {
    auth::require_identity(identity)?;

    match args.command {
        OrderCommand::Place {
            perpetual_id,
            side,
            quantity,
            price,
            order_type,
            time_in_force,
            reduce_only,
            client_id,
        } => {
            let is_buy = matches!(side, Side::Buy);
            let tif: u8 = match time_in_force {
                TimeInForce::Gtc => 0,
                TimeInForce::Ioc => 1,
                TimeInForce::Fok => 3,
            };

            // PrivateKey → EIP-712 签名 → POST /exchange
            // SenderIndex → POST /tx/order (gateway)
            match identity {
                Identity::PrivateKey(_) => {
                    let signer = auth::resolve_signer(identity)?;

                    // 查询市场参数，转换 human-readable → quantums/subticks
                    let meta = info.meta().await?;
                    let market = meta.universe.iter()
                        .find(|m| m.perpetual_id == perpetual_id as i32)
                        .ok_or_else(|| anyhow::anyhow!("Market perpetual_id={perpetual_id} not found"))?;

                    let ar = market.atomic_resolution;
                    let qce = market.quantum_conversion_exponent;
                    const QUOTE_DECIMALS: i32 = 6;

                    // quantums = human_quantity × 10^(-AR)
                    let quantums = (quantity * 10f64.powi(-ar)) as u64;
                    // subticks = human_price × 10^(QUOTE_DECIMALS + AR - QCE)
                    let price_f = price.unwrap_or(0.0);
                    let subticks = (price_f * 10f64.powi(QUOTE_DECIMALS + ar - qce)) as u64;
                    let cid = client_id.unwrap_or(0) as u32;
                    let worst_price = if matches!(order_type, OrderType::Market) {
                        if is_buy { u64::MAX / 2 } else { 1 }
                    } else {
                        subticks
                    };

                    let resp = exchange
                        .place_order(
                            &signer,
                            perpetual_id as u32,
                            is_buy,
                            quantums,
                            subticks,
                            tif,
                            reduce_only,
                            subaccount,
                            cid,
                            worst_price,
                        )
                        .await?;
                    print_exchange_result(&resp, output);
                }
                Identity::SenderIndex(_) => {
                    let mut body = build_gateway_fields(identity, subaccount);
                    body["perpetual_id"] = serde_json::json!(perpetual_id);
                    body["side"] = serde_json::json!(if is_buy { 0 } else { 1 });
                    body["quantity"] = serde_json::json!(quantity);
                    body["order_type"] = serde_json::json!(match order_type {
                        OrderType::Limit => 0,
                        OrderType::Market => 1,
                    });
                    body["time_in_force"] = serde_json::json!(tif);
                    body["reduce_only"] = serde_json::json!(reduce_only);
                    if let Some(p) = price {
                        body["price"] = serde_json::json!(p);
                    }
                    if let Some(cid) = client_id {
                        body["client_id"] = serde_json::json!(cid);
                    }
                    let resp = gateway.place_order(body).await?;
                    crate::output::print_gateway_result(&resp, output);
                }
                Identity::None => unreachable!(),
            }
        }
        OrderCommand::Cancel {
            perpetual_id,
            client_id,
        } => {
            match identity {
                Identity::PrivateKey(_) => {
                    let signer = auth::resolve_signer(identity)?;
                    let resp = exchange
                        .cancel_order(&signer, perpetual_id as u32, client_id as u32, subaccount)
                        .await?;
                    print_exchange_result(&resp, output);
                }
                Identity::SenderIndex(_) => {
                    let mut body = build_gateway_fields(identity, subaccount);
                    body["perpetual_id"] = serde_json::json!(perpetual_id);
                    body["client_id"] = serde_json::json!(client_id);
                    let resp = gateway.cancel_order(body).await?;
                    crate::output::print_gateway_result(&resp, output);
                }
                Identity::None => unreachable!(),
            }
        }
        OrderCommand::List { perpetual_id } => {
            let address = auth::resolve_address(identity, gateway).await?;
            let orders = info.open_orders(&address, perpetual_id).await?;
            crate::output::order::print_orders(&orders, output);
        }
        OrderCommand::History {
            perpetual_id,
            limit,
        } => {
            let address = auth::resolve_address(identity, gateway).await?;
            let orders = info.historical_orders(&address, perpetual_id, limit).await?;
            crate::output::order::print_orders(&orders, output);
        }
        OrderCommand::Status { order_id } => {
            let address = auth::resolve_address(identity, gateway).await?;
            let status = info.order_status(&address, &order_id).await?;
            crate::output::order::print_order_status(&status, output);
        }
    }
    Ok(())
}

fn build_gateway_fields(identity: &Identity, subaccount: u32) -> serde_json::Value {
    match identity {
        Identity::SenderIndex(idx) => serde_json::json!({
            "sender_index": idx,
            "subaccount_number": subaccount,
        }),
        _ => serde_json::json!({"subaccount_number": subaccount}),
    }
}

fn print_exchange_result(resp: &crate::client::exchange::ExchangeResponse, output: OutputFormat) {
    match output {
        OutputFormat::Json => crate::output::print_json(resp),
        OutputFormat::Table => {
            if resp.error.is_none() {
                println!("✓ {}", resp.status);
                if let Some(ref data) = resp.response {
                    if let Some(obj) = data.as_object() {
                        for (k, v) in obj {
                            println!("  {k}: {v}");
                        }
                    }
                }
            } else {
                eprintln!("✗ {}", resp.error.as_deref().unwrap_or(&resp.status));
            }
        }
    }
}
