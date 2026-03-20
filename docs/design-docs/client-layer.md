# Client Layer

## Overview

| Client | Target | Purpose |
|--------|--------|---------|
| `InfoClient` | dex-api POST /info | 16 query types (market data + account data) |
| `ExchangeClient` | dex-api POST /exchange | EIP-712 signed trades (private-key mode) |
| `GatewayClient` | tx-gateway POST /tx/* | Dev/test transactions (sender-index mode) |
| `BridgeClient` | Sepolia EVM RPC | Cross-chain USDC deposits (private-key mode) |
| `WsClient` | dex-api GET /ws | Real-time WebSocket subscriptions |

## InfoClient (src/client/info.rs)

Single-endpoint query client. All requests go to POST /info with a `type` field.

Response types defined inline (13 structs), all with `#[serde(rename_all = "camelCase")]`:
- MetaResponse, PerpetualInfo, L2BookResponse, L2Level
- CandleResponse, MarketStatInfo, FillResponse
- OrderResponse, OrderStatusResponse
- ClearinghouseStateResponse, MarginSummary, AssetPosition, PositionInfo
- BalanceResponse, TransferResponse

## ExchangeClient (src/client/exchange.rs)

EIP-712 signing + POST /exchange. Handles:
- `place_order()` — PlaceOrder type hash (14 fields)
- `cancel_order()` — CancelOrder type hash (5 fields)
- `close_position()` — reuses PlaceOrder with size=0
- `update_leverage()` — UpdateLeverage type hash (6 fields)

Internal functions:
- `domain_separator()` — Hermes-Dex v1, chainId=1
- `eip712_signing_hash()` — keccak256(0x1901 || domain || struct)
- `sign_hash()` — secp256k1 sign → {r, s, v}
- ABI encoding helpers: `abi_encode_u8/u32/u64/u256/bool/address`

## GatewayClient (src/client/gateway.rs)

REST client for tx-gateway. 17 POST endpoints + 3 GET endpoints.
Returns `GatewayResponse { success, message, digest, data }`.

## BridgeClient (src/client/bridge.rs)

EVM contract interaction via alloy `sol!` macro:
- `IERC20`: approve, allowance, balanceOf
- `ISuiBridge`: depositUSDCForSubaccount(bytes32, uint32, uint256)

Deposit flow: check balance → approve (if needed) → deposit → await receipt.

Default addresses (Sepolia devnet):
- Bridge: `0x1A741c8Ae351eEf38c2887cE2B64587756D44d1B`
- MockUSDC: `0x8140EBa492e02Dbf137080E2E4eC0Bd3e10784a0`

## WsClient (src/client/ws.rs)

tokio-tungstenite WebSocket. Sends `subscribeChannel` message, streams events until Ctrl-C.
Table mode: pretty-printed JSON. JSON mode: NDJSON (one line per event).
