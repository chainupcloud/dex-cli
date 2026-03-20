# RELIABILITY.md

## Error Handling

All async functions return `anyhow::Result<T>`:
- `.context("what failed")` — attach context
- `bail!("message")` — immediate error
- `ensure!(cond, "message")` — validation

## Connection Failures

| Service | Error Message |
|---------|--------------|
| dex-api | "Cannot connect to dex-api at {url}. Is the service running?" |
| tx-gateway | "Cannot connect to tx-gateway at {url}. Is the service running?" |
| WebSocket | "Cannot connect to WebSocket at {url}" |
| Sepolia RPC | "Failed to query USDC balance" / "Invalid ETH RPC URL" |
| Bridge tx | "Failed to send bridge deposit transaction" |

## Output Consistency

| Mode | Success | Error |
|------|---------|-------|
| Table | stdout table | stderr text + exit 1 |
| JSON | stdout JSON | stdout `{"error":"..."}` + exit 1 |

## Empty Data

All list commands show "No X found." or empty table — never error on empty results.

## Bridge Deposit

- Balance checked before submitting EVM transaction
- Approve receipt awaited before deposit
- Deposit receipt awaited before reporting
- User informed of 2-10 minute bridge confirmation delay

## WebSocket

- Ctrl-C signal handled for clean disconnect
- Server close message detected and reported
- NDJSON output for `-o json` (one JSON per line)
