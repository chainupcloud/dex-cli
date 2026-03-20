# dex-api Endpoints Reference

## REST — POST /info (Query)

| type | Key Params | Returns |
|------|-----------|---------|
| `meta` | — | MetaResponse (universe: PerpetualInfo[]) |
| `l2Book` | perpetualId | L2BookResponse (bids/asks: L2Level[]) |
| `allMids` | — | Map<perpetualId, midPrice> |
| `recentFills` | perpetualId | FillResponse[] |
| `candleSnapshot` | perpetualId, interval | CandleResponse[] |
| `marketStats` | perpetualId | MarketStatInfo |
| `clearinghouseState` | user, subaccountNumber | ClearinghouseStateResponse |
| `openOrders` | user, perpetualId? | OrderResponse[] |
| `historicalOrders` | user, perpetualId?, limit? | OrderResponse[] |
| `orderStatus` | user, oid | OrderStatusResponse |
| `userFills` | user, perpetualId?, limit? | FillResponse[] |
| `userBalances` | user | BalanceResponse[] |
| `userTransfers` | user | TransferResponse[] |
| `subAccounts` | user | SubAccountInfo[] |
| `userNonFundingLedgerUpdates` | user | LedgerUpdate[] |
| `userRateLimit` | user | RateLimitResponse |

## REST — POST /exchange (EIP-712 Signed)

Request format:
```json
{
  "action": { "type": "order|cancel|closePosition|updateLeverage|withdraw", ... },
  "nonce": 1707580800000,
  "deadline": 1707584400000,
  "signature": { "r": "0x...", "s": "0x...", "v": 27 }
}
```

| Action | EIP-712 Type | Fields |
|--------|-------------|--------|
| order | PlaceOrder | 14 fields (subaccountNumber, clientId, perpetualId, isBuy, quantums, subticks, timeInForce, ...) |
| cancel | CancelOrder | 5 fields (subaccountNumber, clientId, perpetualId, nonce, deadline) |
| updateLeverage | UpdateLeverage | 6 fields |
| withdraw | Withdraw | 5 fields |

## WebSocket — GET /ws

Subscribe: `{"method":"subscribeChannel","subscription":{"channel":"<name>"}}`

| Channel | Identity |
|---------|----------|
| `trades:{perpetualId}` | No |
| `orderbook:{perpetualId}` | No |
| `bbo:{perpetualId}` | No |
| `candle:{perpetualId}:{interval}` | No |
| `allMids` | No |
| `user:{address}` | Yes |
| `orderUpdates:{address}` | Yes |

## Source References

- API handlers: `../dex-sui/crates/dex-api/src/`
- Exchange handlers: `../dex-sui/crates/dex-api/src/exchange/`
- Response types: `../dex-sui/crates/dex-types/src/api/responses.rs`
- EIP-712: `../dex-sui/crates/sui-types/src/dex/eip712.rs`
- HTTP examples: `../dex-sui/docs/indexer/api_docs/http/`
- WebSocket docs: `../dex-sui/docs/indexer/api_docs/ws/dex-websocket.md`
