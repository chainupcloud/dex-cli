# Core Design Principles

## 1. Independent & Lightweight

dex-cli is a standalone Rust binary. No dependency on dex-sui internal crates. Users only need `cargo install` — no Sui dev environment required.

## 2. Dual Identity Mode

- **sender-index**: Zero-config dev/test. Gateway manages keys and gas.
- **private-key**: Production-ready. Same secp256k1 key signs EIP-712 trades and EVM bridge deposits.

Both modes use identical CLI commands — the backend path is determined automatically.

## 3. Same Signing Path as Frontend

Private-key mode uses EIP-712 signing → POST /exchange, identical to the dex-ui frontend. Same domain, same type hashes, same signature format.

## 4. polymarket-cli Architecture

Follows the same structural patterns:
- clap derive: Args → Subcommand → execute()
- Dual output: table (tabled) / json (serde_json)
- Config priority: flag > env > config > default
- Shell: rustyline REPL
- Errors: anyhow + context
- Key storage: hex in JSON config file with 0o600 permissions

## 5. Machine-Readable

All commands support `-o json`. The Claude Code skill uses this for natural language trading. Scripts can pipe `dex -o json ... | jq .field`.
