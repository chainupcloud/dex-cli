use tabled::{Table, Tabled, settings::Style};
use crate::client::info::{OrderResponse, OrderStatusResponse};
use crate::output::OutputFormat;

#[derive(Tabled)]
struct OrderRow {
    #[tabled(rename = "Order ID")]
    order_id: String,
    #[tabled(rename = "Perp")]
    perpetual_id: i32,
    #[tabled(rename = "Coin")]
    coin: String,
    #[tabled(rename = "Side")]
    side: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Qty")]
    quantity: String,
    #[tabled(rename = "Remaining")]
    remaining: String,
    #[tabled(rename = "Type")]
    order_type: String,
    #[tabled(rename = "TIF")]
    tif: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Time")]
    time: String,
}

fn format_side(side: i16) -> &'static str {
    match side {
        0 => "BUY",
        1 => "SELL",
        _ => "UNKNOWN",
    }
}

fn format_order_type(ot: i16) -> &'static str {
    match ot {
        0 => "Limit",
        1 => "Market",
        2 => "StopLimit",
        3 => "StopMarket",
        _ => "Unknown",
    }
}

fn format_tif(tif: i16) -> &'static str {
    match tif {
        0 => "GTC",
        1 => "IOC",
        2 => "FOK",
        3 => "PostOnly",
        _ => "Unknown",
    }
}

fn format_status(status: i16) -> &'static str {
    match status {
        0 => "Open",
        1 => "Filled",
        2 => "Cancelled",
        3 => "Expired",
        _ => "Unknown",
    }
}

fn format_ts(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| ms.to_string())
}

fn truncate_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}...", &id[..12])
    } else {
        id.to_string()
    }
}

fn to_row(o: &OrderResponse) -> OrderRow {
    OrderRow {
        order_id: truncate_id(&o.order_id),
        perpetual_id: o.perpetual_id,
        coin: o.coin.clone(),
        side: format_side(o.side).to_string(),
        price: o.price.clone(),
        quantity: o.quantity.clone(),
        remaining: o.remaining_quantity.clone(),
        order_type: format_order_type(o.order_type).to_string(),
        tif: format_tif(o.time_in_force).to_string(),
        status: format_status(o.status).to_string(),
        time: format_ts(o.timestamp_ms),
    }
}

pub fn print_orders(orders: &[OrderResponse], format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(orders),
        OutputFormat::Table => {
            if orders.is_empty() {
                println!("No orders found.");
                return;
            }
            let rows: Vec<OrderRow> = orders.iter().map(to_row).collect();
            let table = Table::new(&rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}

pub fn print_order_status(resp: &OrderStatusResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => crate::output::print_json(resp),
        OutputFormat::Table => {
            match &resp.order {
                Some(o) => {
                    crate::output::print_detail_table(&[
                        ("Order ID", o.order_id.clone()),
                        ("Client ID", o.client_id.to_string()),
                        ("Perpetual ID", o.perpetual_id.to_string()),
                        ("Coin", o.coin.clone()),
                        ("Side", format_side(o.side).to_string()),
                        ("Price", o.price.clone()),
                        ("Quantity", o.quantity.clone()),
                        ("Remaining", o.remaining_quantity.clone()),
                        ("Type", format_order_type(o.order_type).to_string()),
                        ("TIF", format_tif(o.time_in_force).to_string()),
                        ("Reduce Only", o.reduce_only.to_string()),
                        ("Status", format_status(o.status).to_string()),
                        ("Account", o.account_address.clone()),
                        ("Subaccount", o.subaccount_number.to_string()),
                        ("Time", format_ts(o.timestamp_ms)),
                    ]);
                }
                None => {
                    println!("Order not found (status: {})", resp.status);
                }
            }
        }
    }
}
