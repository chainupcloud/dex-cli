# tx-gateway Endpoints Reference

Used by sender-index mode (dev/test). Source: `../dex-sui/crates/dex-node-test/src/gateway.rs`

## Transaction Endpoints (POST)

| Route | Purpose |
|-------|---------|
| `/tx/order` | Place order (sender_index, perpetual_id, side, quantity, price, ...) |
| `/tx/cancel` | Cancel order (sender_index, perpetual_id, client_id) |
| `/tx/close-position` | Close position (sender_index, perpetual_id, worst_price) |
| `/tx/set-leverage` | Set leverage (sender_index, perpetual_id, leverage) |
| `/tx/deposit` | Mint + deposit USDC (sender_index, subaccount_number, amount) |
| `/tx/withdraw` | Withdraw USDC (sender_index, subaccount_number, amount) |
| `/tx/mint-usdc` | Mint test USDC (sender_index, amount) |
| `/tx/faucet` | Request SUI gas (sender_index) |

## Admin Endpoints (POST)

| Route | Purpose |
|-------|---------|
| `/tx/setup` | Create perpetual (perpetual_id, ticker, atomic_resolution, ...) |
| `/tx/update-oracle-prices` | Update oracle ({price_updates: [{perpetual_id, price, exponent}]}) |
| `/tx/update-funding-rates` | Trigger funding (perpetual_id) |
| `/tx/liquidate` | Liquidate (target_address, subaccount_number) |
| `/tx/setup-vault` | Setup MegaVault (perpetual_id, allocation_amount?) |
| `/tx/update-perpetual-params` | Update params (perpetual_id, ...) |

## Query Endpoints (GET)

| Route | Purpose |
|-------|---------|
| `/tx/status` | Gateway status + address + object IDs |
| `/tx/address` | Default sender address |
| `/tx/addresses?count=N` | Deterministic address list |

## Response Format

```json
{ "success": true, "message": "...", "digest": "0x...", "data": {} }
```
