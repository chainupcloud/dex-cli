use tabled::{Table, Tabled, settings::Style};
use crate::client::info::AssetPosition;
use crate::output::OutputFormat;

#[derive(Tabled)]
struct PositionRow {
    #[tabled(rename = "Coin")]
    coin: String,
    #[tabled(rename = "Size")]
    szi: String,
    #[tabled(rename = "Entry Price")]
    entry_px: String,
    #[tabled(rename = "Position Value")]
    position_value: String,
    #[tabled(rename = "Unrealized PnL")]
    unrealized_pnl: String,
    #[tabled(rename = "Margin Used")]
    margin_used: String,
    #[tabled(rename = "Liq. Price")]
    liquidation_px: String,
    #[tabled(rename = "ROE")]
    return_on_equity: String,
}

pub fn print_positions(positions: &[AssetPosition], format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(positions),
        OutputFormat::Table => {
            if positions.is_empty() {
                println!("No open positions.");
                return;
            }
            let rows: Vec<PositionRow> = positions
                .iter()
                .map(|ap| {
                    let p = &ap.position;
                    PositionRow {
                        coin: p.coin.clone(),
                        szi: p.szi.clone(),
                        entry_px: p.entry_px.clone(),
                        position_value: p.position_value.clone(),
                        unrealized_pnl: p.unrealized_pnl.clone(),
                        margin_used: p.margin_used.clone(),
                        liquidation_px: p.liquidation_px.clone().unwrap_or_else(|| "-".to_string()),
                        return_on_equity: p.return_on_equity.clone(),
                    }
                })
                .collect();
            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}
