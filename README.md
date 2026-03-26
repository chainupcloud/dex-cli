# dex-cli

Command-line trading tool for DEX on Sui. Query markets, place orders, manage positions — all from your terminal.

## Install

**From source (recommended):**

```bash
git clone https://github.com/chainupcloud/dex-cli.git
cd dex-cli
cargo install --path .
```

**From GitHub Release:**

```bash
# macOS (Apple Silicon)
curl -L https://github.com/chainupcloud/dex-cli/releases/latest/download/dex-aarch64-apple-darwin.tar.gz | tar xz
sudo mv dex /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/chainupcloud/dex-cli/releases/latest/download/dex-x86_64-apple-darwin.tar.gz | tar xz
sudo mv dex /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/chainupcloud/dex-cli/releases/latest/download/dex-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv dex /usr/local/bin/
```

Verify:

```bash
dex --version
```

## Quick Start

### 1. Create Wallet

```bash
dex wallet create
# → Generates secp256k1 keypair
# → Saves to ~/.config/dex/config.json
# → Shows your ETH address
```

### 2. Enable Agent Key (one-time, enables frictionless trading)

```bash
dex --api-url https://dex-api.hifo.one agent approve
# → Generates agent keypair
# → Master wallet signs ApproveAgent (one-time EIP-712 signature)
# → Agent key saved to config — all future trades sign locally, no popups
```

### 3. Fund Your Account

```bash
# Get Sepolia ETH for gas (visit https://sepoliafaucet.com)

# Mint test USDC on Sepolia
dex --api-url https://dex-api.hifo.one --env testnet account mint-usdc --amount 100000

# Bridge deposit to DEX (2-10 min to arrive)
dex --api-url https://dex-api.hifo.one --env testnet account deposit --amount 100000

# Check balance
dex -o json --api-url https://dex-api.hifo.one account info
```

### 4. Trade

```bash
# Check BTC price
dex --api-url https://dex-api.hifo.one market mids

# Limit buy 0.01 BTC at $69,000
dex --api-url https://dex-api.hifo.one order place \
  --perpetual-id 0 --side buy --quantity 0.01 --price 69000

# Market sell 0.01 BTC
dex --api-url https://dex-api.hifo.one order place \
  --perpetual-id 0 --side sell --quantity 0.01 --order-type market

# View orders and positions
dex -o json --api-url https://dex-api.hifo.one order list
dex -o json --api-url https://dex-api.hifo.one position list
```

> **Tip:** Set `DEX_API_URL` environment variable to avoid repeating `--api-url`:
> ```bash
> export DEX_API_URL=https://dex-api.hifo.one
> dex market mids    # now works without --api-url
> ```

## Commands

| Command | Description |
|---------|-------------|
| `dex market list` | List all perpetual contracts |
| `dex market mids` | Current mid prices |
| `dex market book <id>` | Order book depth |
| `dex market trades <id>` | Recent trades |
| `dex market candles <id> --interval 1h` | Candlestick data |
| `dex order place --perpetual-id <id> --side buy --quantity <n> --price <p>` | Place limit order |
| `dex order place --perpetual-id <id> --side sell --quantity <n> --order-type market` | Place market order |
| `dex order cancel --perpetual-id <id> --client-id <cid>` | Cancel order |
| `dex order list` | Open orders |
| `dex position list` | Current positions |
| `dex position close --perpetual-id <id>` | Close position |
| `dex position leverage --perpetual-id <id> --leverage <n>` | Set leverage |
| `dex account info` | Balance and margin |
| `dex account mint-usdc --amount <n>` | Mint test USDC (Sepolia) |
| `dex account deposit --amount <n>` | Bridge deposit to DEX |
| `dex account withdraw --amount <n>` | Withdraw from DEX |
| `dex agent approve` | Authorize agent key (one-time) |
| `dex agent show` | Show agent key info |
| `dex agent revoke --agent-address <addr>` | Revoke agent |
| `dex wallet create` | Generate new wallet |
| `dex wallet import <key>` | Import private key |
| `dex wallet show` | Show wallet info |
| `dex watch trades <id>` | Real-time trade stream |
| `dex watch book <id>` | Real-time order book |
| `dex status api` | Check API connectivity |
| `dex shell` | Interactive REPL |

Use `-o json` for JSON output, `-o table` (default) for formatted tables.

## Agent Key (Session Key)

Agent key lets you **sign once, trade forever**. After `dex agent approve`, the CLI uses a locally stored agent key to sign all trading operations — no wallet confirmations needed.

```
Master Wallet ──sign once──→ ApproveAgent ──→ Chain records agent
                                              │
Agent Key (local) ──sign every trade──→ PlaceOrder/CancelOrder
                                              │
                                     Chain verifies: agent → master → execute
```

**Security:** Agent can only trade (place/cancel orders, set leverage). It **cannot** withdraw funds or approve new agents. If agent key leaks, your funds are safe.

```bash
dex agent approve                        # Permanent agent
dex agent approve --valid-until 24h      # Expires in 24 hours
dex agent approve --valid-until 7d       # Expires in 7 days
dex agent show                           # View current agent
dex agent revoke --agent-address 0x...   # Revoke agent
```

## Environment

| Flag | Description |
|------|-------------|
| `--api-url <url>` | DEX API endpoint (or `DEX_API_URL` env) |
| `--gateway-url <url>` | Transaction gateway (or `DEX_GATEWAY_URL` env) |
| `--env testnet` | Use testnet bridge contract |
| `--env devnet` | Use devnet bridge contract (default) |
| `--private-key <key>` | Override private key (or `DEX_PRIVATE_KEY` env) |
| `--sender-index <n>` | Use gateway deterministic key (dev mode) |
| `--subaccount <n>` | Subaccount number (default: 0) |
| `-o json` | JSON output format |

## Use with AI Agent (Claude Code)

The CLI includes a [Claude Code skill](skills/dex/SKILL.md) for natural language trading:

```bash
cd dex-cli
claude            # Start Claude Code

# Then use natural language:
# > /dex check BTC price
# > /dex buy 0.1 BTC at 69000
# > /dex show my positions
# > /dex close my BTC position
```

## Config

Wallet and settings stored at `~/.config/dex/config.json` (macOS: `~/Library/Application Support/dex/config.json`).

```json
{
  "private_key": "0x...",
  "agent_key": "0x...",
  "agent_valid_until": 0,
  "api_url": "https://dex-api.hifo.one"
}
```

File permissions are set to `0600` (owner read/write only).

## License

MIT
