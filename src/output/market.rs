use tabled::{Table, Tabled, settings::Style};
use crate::client::info::{
    CandleResponse, FillResponse, L2BookResponse, MarketStatInfo, MetaResponse,
    PerpetualInfo,
};
use crate::output::OutputFormat;

#[derive(Tabled)]
struct MarketRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Ticker")]
    ticker: String,
    #[tabled(rename = "Max Leverage")]
    max_leverage: String,
    #[tabled(rename = "AR")]
    atomic_resolution: i32,
    #[tabled(rename = "Sz Decimals")]
    sz_decimals: i32,
}

#[derive(Tabled)]
struct BookRow {
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Size")]
    size: String,
}

#[derive(Tabled)]
struct TradeRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Side")]
    side: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Size")]
    size: String,
}

#[derive(Tabled)]
struct CandleRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Open")]
    open: String,
    #[tabled(rename = "High")]
    high: String,
    #[tabled(rename = "Low")]
    low: String,
    #[tabled(rename = "Close")]
    close: String,
    #[tabled(rename = "Volume")]
    volume: String,
}

#[derive(Tabled)]
struct MidRow {
    #[tabled(rename = "Perpetual ID")]
    perpetual_id: String,
    #[tabled(rename = "Mid Price")]
    mid_price: String,
}

fn format_side(side: i16) -> &'static str {
    match side {
        0 => "BUY",
        1 => "SELL",
        _ => "UNKNOWN",
    }
}

fn format_ts(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| ms.to_string())
}

pub fn print_market_list(meta: &MetaResponse, mids: &serde_json::Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            crate::output::print_json(&serde_json::json!({
                "universe": meta.universe,
                "mids": mids,
            }));
        }
        OutputFormat::Table => {
            let rows: Vec<MarketRow> = meta
                .universe
                .iter()
                .map(|p| MarketRow {
                    id: p.perpetual_id,
                    ticker: p.coin.clone(),
                    max_leverage: format!("{}x", p.max_leverage),
                    atomic_resolution: p.atomic_resolution,
                    sz_decimals: p.sz_decimals,
                })
                .collect();

            if rows.is_empty() {
                println!("No markets found.");
                return;
            }

            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}

pub fn print_order_book(book: &L2BookResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(book),
        OutputFormat::Table => {
            println!("=== ASKS ===");
            let ask_rows: Vec<BookRow> = book
                .asks
                .iter()
                .rev()
                .map(|l| BookRow {
                    price: l.price.clone(),
                    size: l.size.clone(),
                })
                .collect();
            if ask_rows.is_empty() {
                println!("  (empty)");
            } else {
                let table = Table::new(&ask_rows).with(Style::rounded()).to_string();
                println!("{table}");
            }

            println!("\n=== BIDS ===");
            let bid_rows: Vec<BookRow> = book
                .bids
                .iter()
                .map(|l| BookRow {
                    price: l.price.clone(),
                    size: l.size.clone(),
                })
                .collect();
            if bid_rows.is_empty() {
                println!("  (empty)");
            } else {
                let table = Table::new(&bid_rows).with(Style::rounded()).to_string();
                println!("{table}");
            }
        }
    }
}

pub fn print_trades(fills: &[FillResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(fills),
        OutputFormat::Table => {
            let rows: Vec<TradeRow> = fills
                .iter()
                .map(|f| TradeRow {
                    time: format_ts(f.timestamp_ms),
                    side: format_side(f.side).to_string(),
                    price: f.price.clone(),
                    size: f.quantity.clone(),
                })
                .collect();
            if rows.is_empty() {
                println!("No recent trades.");
                return;
            }
            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}

pub fn print_candles(candles: &[CandleResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(candles),
        OutputFormat::Table => {
            let rows: Vec<CandleRow> = candles
                .iter()
                .map(|c| CandleRow {
                    time: format_ts(c.timestamp_ms),
                    open: c.open.clone(),
                    high: c.high.clone(),
                    low: c.low.clone(),
                    close: c.close.clone(),
                    volume: c.volume.clone(),
                })
                .collect();
            if rows.is_empty() {
                println!("No candle data.");
                return;
            }
            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}

pub fn print_market_stats(stats: &MarketStatInfo, format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(stats),
        OutputFormat::Table => {
            crate::output::print_detail_table(&[
                ("Perpetual ID", stats.perpetual_id.to_string()),
                ("Coin", stats.coin.clone()),
                ("Last Price", stats.last_price.clone()),
                ("24h Volume", stats.volume_24h.clone()),
                ("24h High", stats.high_24h.clone()),
                ("24h Low", stats.low_24h.clone()),
                ("24h Trades", stats.num_trades_24h.to_string()),
            ]);
        }
    }
}

pub fn print_market_info(info: &PerpetualInfo, format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(info),
        OutputFormat::Table => {
            crate::output::print_detail_table(&[
                ("Perpetual ID", info.perpetual_id.to_string()),
                ("Coin", info.coin.clone()),
                ("Atomic Resolution", info.atomic_resolution.to_string()),
                ("Max Leverage", format!("{}x", info.max_leverage)),
                ("Size Decimals", info.sz_decimals.to_string()),
                ("QCE", info.quantum_conversion_exponent.to_string()),
                ("Subticks Per Tick", info.subticks_per_tick.to_string()),
                ("Step Base Quantums", info.step_base_quantums.to_string()),
            ]);
        }
    }
}

pub fn print_mids(mids: &serde_json::Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(mids),
        OutputFormat::Table => {
            if let Some(obj) = mids.as_object() {
                let rows: Vec<MidRow> = obj
                    .iter()
                    .map(|(k, v)| MidRow {
                        perpetual_id: k.clone(),
                        mid_price: v.as_str().unwrap_or(&v.to_string()).to_string(),
                    })
                    .collect();
                if rows.is_empty() {
                    println!("No mid prices available.");
                    return;
                }
                let table = Table::new(&rows).with(Style::rounded()).to_string();
                println!("{table}");
            } else {
                println!("{}", serde_json::to_string_pretty(mids).unwrap_or_default());
            }
        }
    }
}
