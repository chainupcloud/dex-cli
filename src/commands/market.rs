use anyhow::Result;
use clap::{Args, Subcommand};

use crate::client::info::InfoClient;
use crate::output::OutputFormat;
use crate::output::market;

#[derive(Args)]
pub struct MarketArgs {
    #[command(subcommand)]
    pub command: MarketCommand,
}

#[derive(Subcommand)]
pub enum MarketCommand {
    /// List all perpetual contracts
    List,
    /// Show details for a single contract
    Info {
        /// Perpetual contract ID
        perpetual_id: i32,
    },
    /// Show order book depth
    Book {
        /// Perpetual contract ID
        perpetual_id: i32,
    },
    /// Show recent trades
    Trades {
        /// Perpetual contract ID
        perpetual_id: i32,
    },
    /// Show candlestick/K-line data
    Candles {
        /// Perpetual contract ID
        perpetual_id: i32,
        /// Candle interval
        #[arg(long, default_value = "1h", value_parser = ["1m", "5m", "15m", "1h", "4h", "1d"])]
        interval: String,
    },
    /// Show market statistics (24h volume, OI, funding rate)
    Stats {
        /// Perpetual contract ID
        perpetual_id: i32,
    },
    /// Show mid prices for all contracts
    Mids,
}

pub async fn execute(info: &InfoClient, args: MarketArgs, output: OutputFormat) -> Result<()> {
    match args.command {
        MarketCommand::List => {
            let meta = info.meta().await?;
            let mids = info.all_mids().await?;
            market::print_market_list(&meta, &mids, output);
        }
        MarketCommand::Info { perpetual_id } => {
            let meta = info.meta().await?;
            let perp = meta
                .universe
                .iter()
                .find(|p| p.perpetual_id == perpetual_id)
                .ok_or_else(|| anyhow::anyhow!("Perpetual ID {perpetual_id} not found"))?;
            market::print_market_info(perp, output);
        }
        MarketCommand::Book { perpetual_id } => {
            let book = info.l2_book(perpetual_id).await?;
            market::print_order_book(&book, output);
        }
        MarketCommand::Trades { perpetual_id } => {
            let fills = info.recent_fills(perpetual_id).await?;
            market::print_trades(&fills, output);
        }
        MarketCommand::Candles {
            perpetual_id,
            interval,
        } => {
            let candles = info.candle_snapshot(perpetual_id, &interval).await?;
            market::print_candles(&candles, output);
        }
        MarketCommand::Stats { perpetual_id } => {
            let stats = info.market_stats(perpetual_id).await?;
            market::print_market_stats(&stats, output);
        }
        MarketCommand::Mids => {
            let mids = info.all_mids().await?;
            market::print_mids(&mids, output);
        }
    }
    Ok(())
}
