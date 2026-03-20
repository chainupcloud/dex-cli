# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dex-cli is a command-line trading tool for the DEX on Sui, modeled after [polymarket-cli](../polymarket-cli/). Binary name: `dex`.

Two identity modes:
- **sender-index**: tx-gateway manages keys internally, transactions via POST /tx/* (dev/test)
- **private-key**: secp256k1 EIP-712 signing, transactions via POST /exchange, bridge deposits via EVM (production)

## Documentation

- `AGENTS.md` — AI agent development guide and project spec
- `ARCHITECTURE.md` — System architecture, data flows, identity model
- `.claude/skills/dex/SKILL.md` — Claude Code skill for natural language trading
- `docs/` — Design docs, execution plans, product specs, references

## Build & Test

```bash
cargo build                    # Build
cargo run -- --help            # Show help
cargo run -- market list       # Run a command
cargo test                     # Run integration tests (11 tests)
cargo fmt --all -- --check     # Check formatting
cargo clippy -- -D warnings    # Lint
```

## Runtime Dependencies

Requires dex-dev Docker environment for dev/test:

```bash
cd ../dex-sui/docker/dex-dev && make up
# dex-api: http://127.0.0.1:9100
# tx-gateway: http://127.0.0.1:3200
```

For bridge deposits (private-key mode), needs Sepolia RPC access.

## Key Architecture

```
src/
├── main.rs              # clap CLI, 10 commands, 7 global options
├── config.rs            # ~/.config/dex/config.json (0o600), bridge config
├── auth.rs              # Identity resolution, secp256k1 signer, Sui address derivation
├── shell.rs             # rustyline REPL
├── client/
│   ├── info.rs          # InfoClient — POST /info (market queries)
│   ├── exchange.rs      # ExchangeClient — EIP-712 signing → POST /exchange
│   ├── gateway.rs       # GatewayClient — POST /tx/* (dev/test)
│   ├── bridge.rs        # BridgeClient — EVM USDC approve + bridge deposit
│   └── ws.rs            # WsClient — WebSocket subscriptions
├── commands/            # One file per command group
└── output/              # tabled table formatting + JSON output
```

## Language Rules

| Type | Language |
|------|----------|
| Code | English (variable names, function names, type names) |
| Comments | Chinese (business logic, complex algorithms) |
| Documentation | Chinese |
| Responses | Chinese |
