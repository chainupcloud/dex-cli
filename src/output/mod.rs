pub mod account;
pub mod market;
pub mod order;
pub mod position;

use anyhow::Error;

/// 输出格式
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

/// 输出 JSON
pub fn print_json<T: serde::Serialize + ?Sized>(data: &T) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}

/// 按格式输出错误
pub fn print_error(err: &Error, format: OutputFormat) {
    match format {
        OutputFormat::Table => {
            eprintln!("Error: {err:#}");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({"error": format!("{err:#}")})
            );
        }
    }
}

/// 输出两列 detail table
pub fn print_detail_table(rows: &[(&str, String)]) {
    use tabled::{Table, settings::Style};

    let data: Vec<[String; 2]> = rows
        .iter()
        .map(|(k, v)| [k.to_string(), v.clone()])
        .collect();

    let table = Table::new(data)
        .with(Style::rounded())
        .to_string();
    println!("{table}");
}

/// 截断字符串（Unicode 安全）
pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

/// 格式化大数字: 12345678 → "12.3M", 1234 → "1.2K"
pub fn format_decimal(n: f64) -> String {
    let abs = n.abs();
    if abs >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.1}K", n / 1_000.0)
    } else if abs >= 1.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.4}", n)
    }
}

/// 格式化交易结果
pub fn print_gateway_result(
    resp: &crate::client::gateway::GatewayResponse,
    format: OutputFormat,
) {
    match format {
        OutputFormat::Json => print_json(resp),
        OutputFormat::Table => {
            if resp.success {
                println!("✓ {}", resp.message);
                if let Some(ref digest) = resp.digest {
                    println!("  Digest: {digest}");
                }
            } else {
                eprintln!("✗ {}", resp.message);
            }
        }
    }
}
