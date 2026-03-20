# AGENTS.md

AI agent development guide for dex-cli.

## Project

| Key | Value |
|-----|-------|
| Language | Rust (edition 2021) |
| Binary | `dex` |
| Pattern | polymarket-cli (clap derive + commands/ + output/ + shell) |
| Identity modes | sender-index (gateway) / private-key (EIP-712 + bridge) |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` 4 | CLI parsing (derive macros, env vars) |
| `tokio` 1 | Async runtime |
| `reqwest` 0.12 | HTTP client (rustls) |
| `alloy` 1.6 | secp256k1 signing, EVM provider, contract calls, sol! macro |
| `blake2` 0.10 | Sui address derivation (Blake2b256) |
| `serde` / `serde_json` | JSON serialization |
| `tabled` 0.17 | Terminal tables |
| `rustyline` 15 | Interactive shell |
| `tokio-tungstenite` | WebSocket client |
| `anyhow` 1 | Error handling |
| `hex` 0.4 | Hex encoding |

## Module Structure

```
src/
├── main.rs              # Cli struct, Commands enum, run() dispatch
├── config.rs            # DexConfig JSON, bridge addresses, file permissions
├── auth.rs              # Identity enum, PrivateKeySigner, Sui address derivation
├── shell.rs             # rustyline REPL, split_args, history
├── client/
│   ├── info.rs          # InfoClient: POST /info (16 query types, 13 response structs)
│   ├── exchange.rs      # ExchangeClient: EIP-712 domain/type hashes, ABI encoding, signing
│   ├── gateway.rs       # GatewayClient: POST /tx/* (17 endpoints)
│   ├── bridge.rs        # BridgeClient: EVM IERC20.approve + ISuiBridge.depositUSDCForSubaccount
│   └── ws.rs            # WsClient: tokio-tungstenite, subscribeChannel, Ctrl-C
├── commands/
│   ├── market.rs        # 7 subcommands (list/info/book/trades/candles/stats/mids)
│   ├── order.rs         # 5 subcommands, dual-mode: EIP-712 or gateway
│   ├── position.rs      # 3 subcommands, dual-mode
│   ├── account.rs       # 8 subcommands, bridge deposit for private-key mode
│   ├── wallet.rs        # 6 subcommands, real secp256k1 key generation
│   ├── watch.rs         # 7 WebSocket channels
│   ├── admin.rs         # 6 subcommands (setup/oracle/funding/liquidate/vault/params)
│   ├── status.rs        # api/gateway connectivity check
│   └── setup.rs         # Interactive wizard
└── output/
    ├── mod.rs           # OutputFormat, print_json, print_error, print_detail_table
    ├── market.rs        # MarketRow, BookRow, TradeRow, CandleRow, MidRow
    ├── order.rs         # OrderRow, print_order_status
    ├── position.rs      # PositionRow
    └── account.rs       # FillRow, BalanceRow, TransferRow, print_account_info
```

## Identity & Signing

### sender-index mode (dev/test)
- `--sender-index N` → gateway manages deterministic Ed25519 keys
- Transactions: POST /tx/order, /tx/cancel, etc.
- No private key needed on CLI side

### private-key mode (production)
- `dex wallet create` → secp256k1 keypair, hex format, stored in config
- Trading: EIP-712 signing → POST /exchange (same path as dex-ui frontend)
- Deposits: EVM bridge (approve USDC + depositUSDCForSubaccount on Sepolia)
- Sui address: Blake2b256(0x01 || compressed_pubkey)
- Config priority: CLI flag > env var > config file > default

### EIP-712 Domain
- Name: "Hermes-Dex", Version: "1", ChainId: 1, VerifyingContract: 0x0
- Types: PlaceOrder (14 fields), CancelOrder (5), UpdateLeverage (6)
- Nonce: millisecond timestamp, strictly increasing
- Deadline: nonce + 1 hour (orders) or + 5 minutes (cancel)

## Coding Patterns

```rust
// Command pattern (every command file)
#[derive(Args)]
pub struct XxxArgs { #[command(subcommand)] pub command: XxxCommand }

#[derive(Subcommand)]
pub enum XxxCommand { List, Place { #[arg(long)] field: T }, ... }

pub async fn execute(...) -> Result<()> { match args.command { ... } }

// Dual-mode trading (order.rs, position.rs)
match identity {
    Identity::PrivateKey(_) => { /* EIP-712 → exchange client */ }
    Identity::SenderIndex(_) => { /* JSON body → gateway client */ }
    Identity::None => unreachable!(),
}

// Output: table vs json
match format {
    OutputFormat::Json => crate::output::print_json(&data),
    OutputFormat::Table => { /* tabled + Style::rounded() */ }
}

// Errors: anyhow
.context("describe what failed")?
anyhow::bail!("direct error")
anyhow::ensure!(condition, "validation message")
```

## References

| Resource | Location |
|----------|----------|
| polymarket-cli source | `../polymarket-cli/src/` |
| dex-api handlers | `../dex-sui/crates/dex-api/src/` |
| tx-gateway routes | `../dex-sui/crates/dex-node-test/src/gateway.rs` |
| EIP-712 types | `../dex-sui/crates/sui-types/src/dex/eip712.rs` |
| Bridge contract | `../dex-sui/bridge/evm/contracts/SuiBridge.sol` |
| API docs | `../dex-sui/docs/indexer/api_docs/` |
