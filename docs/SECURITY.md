# SECURITY.md

## Private Key Protection

| Measure | Detail |
|---------|--------|
| Storage format | secp256k1 hex with 0x prefix in `~/.config/dex/config.json` |
| File permissions | File 0o600, directory 0o700 (Unix) |
| No key echo | `wallet show` displays address and key source, never the key itself |
| No logging | HTTP request logs exclude private_key field |
| Config priority | CLI flag > env var > config file — allows ephemeral key usage |

## Identity Isolation

| Mode | Key Location | Who Signs |
|------|-------------|-----------|
| sender-index | tx-gateway internal | Gateway (CLI never touches keys) |
| private-key | CLI config file | CLI locally (secp256k1 EIP-712) |

## Confirmation Mechanisms

| Operation | Protection |
|-----------|-----------|
| `wallet create` (existing wallet) | Requires `--force` |
| `wallet import` (existing wallet) | Requires `--force` |
| `wallet reset` | Interactive confirmation, or `--force` |
| Bridge deposit | CLI shows amount + addresses before submitting EVM tx |

## EVM Transaction Safety

- Bridge deposits check USDC balance before submitting
- Allowance checked before approve (skips if sufficient)
- Transaction receipts awaited before reporting success

## Environment Isolation

- devnet / testnet via URL configuration, no production defaults
- Bridge addresses configurable (defaults to Sepolia dev contract)
- MockUSDC on Sepolia, not real assets
