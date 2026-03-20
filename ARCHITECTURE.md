# ARCHITECTURE.md

## System Overview

```
┌────────────────────────────────────────────────────────────────┐
│                          dex CLI                               │
│                                                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ Commands │  │  Shell   │  │  Skill   │  │ Wallet Mgmt   │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬────────┘  │
│       └──────────────┴─────────────┴───────────────┘           │
│                          │                                     │
│  ┌───────────────────────▼──────────────────────────────────┐  │
│  │                    Core Layer                             │  │
│  │  Config (JSON)  │  Auth (secp256k1)  │  Output (tabled)  │  │
│  └──────────────────────┬───────────────────────────────────┘  │
│                         │                                      │
│  ┌──────────────────────▼──────────────────────────────────┐   │
│  │                   Client Layer                           │   │
│  │                                                          │   │
│  │  InfoClient    ExchangeClient   GatewayClient   WsClient│   │
│  │  (queries)     (EIP-712 trade)  (dev/test tx)   (subs)  │   │
│  │      │              │                │             │     │   │
│  │      │         BridgeClient                        │     │   │
│  │      │         (EVM deposit)                       │     │   │
│  └──────│──────────────│────────────────│─────────────│─────┘   │
└─────────│──────────────│────────────────│─────────────│─────────┘
          │              │                │             │
   ┌──────▼──────┐ ┌─────▼─────┐ ┌───────▼──────┐ ┌───▼───────┐
   │  dex-api    │ │  dex-api  │ │  tx-gateway  │ │  dex-api  │
   │ POST /info  │ │POST /exch │ │  POST /tx/*  │ │  GET /ws  │
   │   :9100     │ │  :9100    │ │    :3200     │ │   :9100   │
   └─────────────┘ └─────────┬─┘ └──────────────┘ └───────────┘
                             │
                    ┌────────▼────────┐
                    │  Bridge (EVM)   │
                    │  Sepolia :8545  │
                    └─────────────────┘
```

## Identity Model

```
                    ┌─────────────────────────┐
                    │   resolve_identity()    │
                    │                         │
                    │  --sender-index N       │
                    │    → SenderIndex(N)     │──→ gateway POST /tx/*
                    │    → gateway manages    │     (no local signing)
                    │      keys + gas         │
                    │                         │
                    │  --private-key 0x...    │
                    │    → PrivateKey(hex)    │──→ EIP-712 POST /exchange
                    │    → PrivateKeySigner   │     (local secp256k1 signing)
                    │    → ETH addr + Sui addr│──→ bridge deposit on EVM
                    │                         │     (approve + depositUSDCForSubaccount)
                    │  (nothing)              │
                    │    → None               │──→ read-only commands only
                    └─────────────────────────┘

  Config priority: CLI flag > env var > config file > default
```

## Transaction Flows

### Query (both modes)
```
User → clap → commands/market.rs → InfoClient::meta()
  → POST /info {"type":"meta"} → dex-api → JSON response
  → output/market.rs → tabled table or JSON → stdout
```

### Trade via EIP-712 (private-key mode)
```
User → clap → commands/order.rs → auth::resolve_signer()
  → ExchangeClient::place_order()
    → build EIP-712 struct hash (PlaceOrder, 14 fields)
    → keccak256(0x1901 || domainSep || structHash)
    → secp256k1 sign → {r, s, v}
    → POST /exchange {action, nonce, deadline, signature}
  → dex-api verifies signature, recovers address
  → submits to Sui chain
```

### Trade via Gateway (sender-index mode)
```
User → clap → commands/order.rs → build JSON body {sender_index, ...}
  → GatewayClient::place_order()
    → POST /tx/order → tx-gateway
    → gateway signs with internal Ed25519 key
    → submits to Sui chain
  → GatewayResponse {success, digest}
```

### Bridge Deposit (private-key mode)
```
User → "dex account deposit --amount 1000"
  → auth::resolve_signer() → secp256k1 PrivateKeySigner
  → auth::derive_sui_address() → Blake2b256(0x01 || compressed_pubkey)
  → BridgeClient::deposit()
    1. IERC20.balanceOf(user) → check USDC balance on Sepolia
    2. IERC20.approve(bridge, MAX) → if allowance < amount
    3. ISuiBridge.depositUSDCForSubaccount(suiAddr, subaccount, amount)
    → EVM tx submitted to Sepolia
  → Bridge Node detects event → multi-sig → DEX chain credits account
  → Funds arrive in 2-10 minutes
```

## EIP-712 Domain

```
Domain: { name: "Hermes-Dex", version: "1", chainId: 1, verifyingContract: 0x0 }

PlaceOrder(
  uint32 subaccountNumber, uint32 clientId, uint32 perpetualId,
  bool isBuy, uint64 quantums, uint64 subticks, uint8 timeInForce,
  uint64 goodTilBlockTime, bool reduceOnly, uint8 conditionType,
  uint64 triggerSubticks, uint64 worstPrice, uint64 nonce, uint64 deadline
)

CancelOrder(uint32 subaccountNumber, uint32 clientId, uint32 perpetualId, uint64 nonce, uint64 deadline)

UpdateLeverage(uint32 subaccountNumber, uint32 perpetualId, bool isCross, uint32 leverage, uint64 nonce, uint64 deadline)
```

## Address Derivation

| Address Type | Derivation | Format |
|-------------|------------|--------|
| ETH address | keccak256(uncompressed_pubkey[1..])[12..32] | 0x + 20 bytes hex |
| Sui address | Blake2b256(0x01 \|\| compressed_pubkey) | 0x + 32 bytes hex |

Same secp256k1 key produces both addresses. ETH address used for EVM operations (bridge). Sui address used as bridge deposit recipient.

## Configuration

```
~/.config/dex/config.json (permissions: 0o600)
{
  "api_url": "http://127.0.0.1:9100",
  "gateway_url": "http://127.0.0.1:3200",
  "private_key": "0x...",              // secp256k1 hex (32 bytes)
  "address": "0x...",                  // ETH address (derived)
  "sender_index": 0,                   // alternative to private_key
  "eth_rpc_url": "https://rpc.sepolia.org",
  "bridge_address": "0x1A741c8...",    // Sepolia bridge contract
  "usdc_address": "0x8140EBa4..."     // Sepolia MockUSDC
}
```

## External Services

| Service | Port (dev) | Port (test) | Purpose |
|---------|-----------|-------------|---------|
| dex-api | 9100 | 9101 | REST queries + WebSocket + EIP-712 exchange |
| tx-gateway | 3200 | 3201 | Dev/test transaction submission |
| Sui Node | 9001 | 9005 | Chain (CLI doesn't connect directly) |
| Bridge (EVM) | Sepolia | Sepolia | Cross-chain USDC deposits |
