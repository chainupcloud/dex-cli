# Configuration & Authentication

## Config File

Path: `~/.config/dex/config.json` (permissions 0o600)

```json
{
  "api_url": "http://127.0.0.1:9100",
  "gateway_url": "http://127.0.0.1:3200",
  "private_key": "0x...",
  "address": "0x...",
  "sender_index": 0,
  "environment": "devnet",
  "default_subaccount": 0,
  "eth_rpc_url": "https://rpc.sepolia.org",
  "bridge_address": "0x1A741c8Ae351eEf38c2887cE2B64587756D44d1B",
  "usdc_address": "0x8140EBa492e02Dbf137080E2E4eC0Bd3e10784a0"
}
```

## Config Priority

```
CLI flag > env var > config file > default
```

| Parameter | CLI Flag | Env Var | Default |
|-----------|----------|---------|---------|
| API URL | `--api-url` | `DEX_API_URL` | `http://127.0.0.1:9100` |
| Gateway URL | `--gateway-url` | `DEX_GATEWAY_URL` | `http://127.0.0.1:3200` |
| Private key | `--private-key` | `DEX_PRIVATE_KEY` | config file |
| Sender index | `--sender-index` | `DEX_SENDER_INDEX` | config file |
| Subaccount | `--subaccount` | `DEX_SUBACCOUNT` | `0` |

## Identity Resolution

```
--sender-index (flag) → SenderIndex(N)
  > --private-key (flag/env) → PrivateKey(hex)
  > config.sender_index → SenderIndex(N)
  > config.private_key → PrivateKey(hex)
  > Identity::None (read-only only)
```

## Key Management (polymarket-cli pattern)

- `wallet create` → `PrivateKeySigner::random()` → secp256k1 keypair
- Stored as `0x`-prefixed hex (32 bytes) in config JSON
- `wallet import <hex>` → validates and stores
- `wallet show` → shows address + key source, never the key itself
- `wallet reset` → deletes config (requires confirmation)

## Address Derivation

From one secp256k1 key, two addresses are derived:

| Address | Algorithm | Usage |
|---------|-----------|-------|
| ETH | keccak256(uncompressed[1..])[12..32] | EVM transactions, wallet display |
| Sui | Blake2b256(0x01 \|\| compressed) | Bridge deposit recipient, DEX subaccount owner |

## Dual-mode Transaction Routing

| Command | private-key mode | sender-index mode |
|---------|-----------------|-------------------|
| order place/cancel | EIP-712 → POST /exchange | JSON → POST /tx/order |
| position close/leverage | EIP-712 → POST /exchange | JSON → POST /tx/* |
| account deposit | EVM bridge contract | Gateway mint+deposit |
| account withdraw | Gateway (future: EIP-712) | Gateway |
| account mint-usdc | N/A | Gateway |
| admin * | N/A | Gateway |
| market/order list/etc | POST /info (same) | POST /info (same) |
