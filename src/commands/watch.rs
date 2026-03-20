use anyhow::Result;
use clap::{Args, Subcommand};

use crate::auth::{self, Identity};
use crate::client::ws::WsClient;
use crate::output::OutputFormat;

#[derive(Args)]
pub struct WatchArgs {
    #[command(subcommand)]
    pub command: WatchCommand,
}

#[derive(Subcommand)]
pub enum WatchCommand {
    /// Watch real-time trades
    Trades {
        /// Perpetual contract ID
        perpetual_id: i32,
    },
    /// Watch real-time order book
    Book {
        /// Perpetual contract ID
        perpetual_id: i32,
    },
    /// Watch best bid/ask
    Bbo {
        /// Perpetual contract ID
        perpetual_id: i32,
    },
    /// Watch real-time candles
    Candles {
        /// Perpetual contract ID
        perpetual_id: i32,
        /// Candle interval
        #[arg(long, default_value = "1h", value_parser = ["1m", "5m", "15m", "1h", "4h", "1d"])]
        interval: String,
    },
    /// Watch all mid prices
    Mids,
    /// Watch user events (positions, balances)
    User,
    /// Watch user order updates
    Orders,
}

pub async fn execute(
    api_url: &str,
    args: WatchArgs,
    output: OutputFormat,
    identity: &Identity,
) -> Result<()> {
    let ws = WsClient::new(api_url);

    let channel = match &args.command {
        WatchCommand::Trades { perpetual_id } => format!("trades:{perpetual_id}"),
        WatchCommand::Book { perpetual_id } => format!("orderbook:{perpetual_id}"),
        WatchCommand::Bbo { perpetual_id } => format!("bbo:{perpetual_id}"),
        WatchCommand::Candles {
            perpetual_id,
            interval,
        } => format!("candle:{perpetual_id}:{interval}"),
        WatchCommand::Mids => "allMids".to_string(),
        WatchCommand::User => {
            auth::require_identity(identity)?;
            let gateway = crate::client::gateway::GatewayClient::new(
                &api_url.replace("9100", "3200").replace("9101", "3201"),
            );
            let address = auth::resolve_address(identity, &gateway).await?;
            format!("user:{address}")
        }
        WatchCommand::Orders => {
            auth::require_identity(identity)?;
            let gateway = crate::client::gateway::GatewayClient::new(
                &api_url.replace("9100", "3200").replace("9101", "3201"),
            );
            let address = auth::resolve_address(identity, &gateway).await?;
            format!("orderUpdates:{address}")
        }
    };

    ws.subscribe(&channel, output).await
}
