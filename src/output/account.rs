use tabled::{Table, Tabled, settings::Style};
use crate::client::info::{BalanceResponse, ClearinghouseStateResponse, FillResponse, TransferResponse};
use crate::output::OutputFormat;

pub fn print_account_info(
    address: &str,
    subaccount: u32,
    state: &ClearinghouseStateResponse,
    format: OutputFormat,
) {
    match format {
        OutputFormat::Json => crate::output::print_json(state),
        OutputFormat::Table => {
            let ms = &state.margin_summary;
            crate::output::print_detail_table(&[
                ("Address", address.to_string()),
                ("Subaccount", subaccount.to_string()),
                ("Account Value", ms.account_value.clone()),
                ("Total Notional", ms.total_ntl_pos.clone()),
                ("Free Collateral", state.withdrawable.clone()),
                ("Total Raw USD", ms.total_raw_usd.clone()),
                ("Margin Used", ms.total_margin_used.clone()),
                ("Positions", state.asset_positions.len().to_string()),
            ]);
        }
    }
}

#[derive(Tabled)]
struct FillRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Coin")]
    coin: String,
    #[tabled(rename = "Side")]
    side: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Quantity")]
    quantity: String,
}

fn format_ts(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| ms.to_string())
}

fn format_side(side: i16) -> &'static str {
    match side {
        0 => "BUY",
        1 => "SELL",
        _ => "UNKNOWN",
    }
}

pub fn print_fills(fills: &[FillResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(fills),
        OutputFormat::Table => {
            if fills.is_empty() {
                println!("No fills found.");
                return;
            }
            let rows: Vec<FillRow> = fills
                .iter()
                .map(|f| FillRow {
                    time: format_ts(f.timestamp_ms),
                    coin: f.coin.clone(),
                    side: format_side(f.side).to_string(),
                    price: f.price.clone(),
                    quantity: f.quantity.clone(),
                })
                .collect();
            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}

#[derive(Tabled)]
struct BalanceRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Delta")]
    delta: String,
    #[tabled(rename = "New Balance")]
    new_balance: String,
    #[tabled(rename = "Type")]
    update_type: String,
}

fn format_update_type(t: i16) -> &'static str {
    match t {
        0 => "Deposit",
        1 => "Withdraw",
        2 => "Trade",
        3 => "Fee",
        4 => "Funding",
        _ => "Unknown",
    }
}

pub fn print_balances(balances: &[BalanceResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(balances),
        OutputFormat::Table => {
            if balances.is_empty() {
                println!("No balance history.");
                return;
            }
            let rows: Vec<BalanceRow> = balances
                .iter()
                .map(|b| BalanceRow {
                    time: format_ts(b.timestamp_ms),
                    delta: b.delta.clone(),
                    new_balance: b.new_balance.clone(),
                    update_type: format_update_type(b.update_type).to_string(),
                })
                .collect();
            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}

#[derive(Tabled)]
struct TransferRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "From")]
    from: String,
    #[tabled(rename = "To")]
    to: String,
    #[tabled(rename = "Amount")]
    amount: String,
}

pub fn print_transfers(transfers: &[TransferResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(transfers),
        OutputFormat::Table => {
            if transfers.is_empty() {
                println!("No transfer history.");
                return;
            }
            let rows: Vec<TransferRow> = transfers
                .iter()
                .map(|t| TransferRow {
                    time: format_ts(t.timestamp_ms),
                    from: crate::output::truncate(&t.from_account, 16),
                    to: crate::output::truncate(&t.to_account, 16),
                    amount: t.amount.clone(),
                })
                .collect();
            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}
