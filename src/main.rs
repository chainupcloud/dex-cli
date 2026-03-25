mod auth;
mod client;
mod commands;
mod config;
mod output;
mod shell;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "dex", about = "DEX CLI — query markets, trade, and manage positions", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: table or json
    #[arg(short, long, global = true, default_value = "table")]
    pub(crate) output: OutputFormat,

    /// dex-api URL
    #[arg(long, global = true, env = "DEX_API_URL")]
    api_url: Option<String>,

    /// tx-gateway URL
    #[arg(long, global = true, env = "DEX_GATEWAY_URL")]
    gateway_url: Option<String>,

    /// Private key (overrides config file)
    #[arg(long, global = true, env = "DEX_PRIVATE_KEY")]
    private_key: Option<String>,

    /// Deterministic key index on tx-gateway (overrides private key)
    #[arg(long, global = true, env = "DEX_SENDER_INDEX")]
    sender_index: Option<u32>,

    /// Subaccount number
    #[arg(long, global = true, env = "DEX_SUBACCOUNT", default_value = "0")]
    subaccount: u32,

    /// Environment preset: devnet or testnet
    #[arg(long, global = true, value_parser = ["devnet", "testnet"])]
    env: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Query market data (no wallet needed)
    Market(commands::market::MarketArgs),
    /// Manage orders: place, cancel, list
    Order(commands::order::OrderArgs),
    /// Manage positions: list, close, leverage
    Position(commands::position::PositionArgs),
    /// Account info, fills, balances, deposit, withdraw
    Account(commands::account::AccountArgs),
    /// Manage wallet and keys
    Wallet(commands::wallet::WalletArgs),
    /// Watch real-time WebSocket feeds
    Watch(commands::watch::WatchArgs),
    /// Manage agent/session keys (approve, revoke, list)
    Agent(commands::agent::AgentArgs),
    /// Admin commands for dev/test (setup markets, oracle, funding)
    Admin(commands::admin::AdminArgs),
    /// Check service connectivity
    Status(commands::status::StatusArgs),
    /// Interactive first-time setup wizard
    Setup,
    /// Launch interactive shell
    Shell,
}

impl Cli {
    /// 解析生效的 api_url（优先级: flag/env > config > env preset > default）
    pub(crate) fn effective_api_url(&self) -> String {
        if let Some(ref url) = self.api_url {
            return url.clone();
        }
        if let Ok(cfg) = config::load_config() {
            if let Some(ref url) = cfg.api_url {
                return url.clone();
            }
        }
        match self.env.as_deref() {
            Some("testnet") => "http://127.0.0.1:9101".to_string(),
            _ => "http://127.0.0.1:9100".to_string(),
        }
    }

    /// 解析生效的 gateway_url
    pub(crate) fn effective_gateway_url(&self) -> String {
        if let Some(ref url) = self.gateway_url {
            return url.clone();
        }
        if let Ok(cfg) = config::load_config() {
            if let Some(ref url) = cfg.gateway_url {
                return url.clone();
            }
        }
        match self.env.as_deref() {
            Some("testnet") => "http://127.0.0.1:3201".to_string(),
            _ => "http://127.0.0.1:3200".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let output = cli.output;

    if let Err(e) = run(cli).await {
        output::print_error(&e, output);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

pub(crate) async fn run(cli: Cli) -> anyhow::Result<()> {
    let api_url = cli.effective_api_url();
    let gateway_url = cli.effective_gateway_url();
    let output = cli.output;
    let subaccount = cli.subaccount;

    let info = client::info::InfoClient::new(&api_url);
    let gateway = client::gateway::GatewayClient::new(&gateway_url);
    let exchange = client::exchange::ExchangeClient::new(&api_url);

    // 解析身份
    let identity = auth::resolve_identity(
        cli.sender_index,
        cli.private_key.as_deref(),
    );

    match cli.command {
        Commands::Market(args) => {
            commands::market::execute(&info, args, output).await
        }
        Commands::Order(args) => {
            commands::order::execute(&info, &gateway, args, output, &identity, subaccount, &exchange).await
        }
        Commands::Position(args) => {
            commands::position::execute(&info, &gateway, args, output, &identity, subaccount, &exchange).await
        }
        Commands::Account(args) => {
            commands::account::execute(&info, &gateway, args, output, &identity, subaccount).await
        }
        Commands::Wallet(args) => {
            commands::wallet::execute(&gateway, args, output, &identity).await
        }
        Commands::Watch(args) => {
            commands::watch::execute(&api_url, args, output, &identity).await
        }
        Commands::Agent(args) => {
            commands::agent::execute(&exchange, &info, args, output, &identity).await
        }
        Commands::Admin(args) => {
            commands::admin::execute(&gateway, args, output, &identity).await
        }
        Commands::Status(args) => {
            commands::status::execute(&info, &gateway, args, output).await
        }
        Commands::Setup => {
            commands::setup::execute()
        }
        Commands::Shell => {
            Box::pin(shell::run_shell()).await
        }
    }
}
