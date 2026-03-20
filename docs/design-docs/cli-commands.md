# CLI Commands

## Global Options

```
dex [OPTIONS] <COMMAND>

  -o, --output <FORMAT>        table | json (default: table)
  --api-url <URL>              dex-api URL [env: DEX_API_URL]
  --gateway-url <URL>          tx-gateway URL [env: DEX_GATEWAY_URL]
  --private-key <KEY>          secp256k1 hex key [env: DEX_PRIVATE_KEY]
  --sender-index <N>           gateway key index [env: DEX_SENDER_INDEX]
  --subaccount <N>             subaccount number [env: DEX_SUBACCOUNT] (default: 0)
  --env <ENV>                  devnet | testnet
```

## Command Tree

```
dex
├── market                          # No identity required
│   ├── list                        # All perpetual contracts
│   ├── info <perpetual_id>         # Single contract details
│   ├── book <perpetual_id>         # Order book depth
│   ├── trades <perpetual_id>       # Recent trades
│   ├── candles <perpetual_id>      # K-line (--interval 1m|5m|15m|1h|4h|1d)
│   ├── stats <perpetual_id>        # 24h statistics
│   └── mids                        # All mid prices
│
├── order                           # Requires identity
│   ├── place                       # --perpetual-id --side --quantity [--price --order-type --time-in-force --reduce-only --client-id]
│   ├── cancel                      # --perpetual-id --client-id
│   ├── list                        # [--perpetual-id]
│   ├── history                     # [--perpetual-id --limit]
│   └── status <order_id>
│
├── position                        # Requires identity
│   ├── list
│   ├── close                       # --perpetual-id --worst-price [--size]
│   └── leverage                    # --perpetual-id --leverage
│
├── account                         # Requires identity
│   ├── info                        # Account overview
│   ├── fills                       # [--perpetual-id --limit]
│   ├── balances                    # Balance change history
│   ├── transfers                   # Transfer history
│   ├── deposit                     # --amount (bridge for private-key, gateway for sender-index)
│   ├── withdraw                    # --amount
│   ├── mint-usdc                   # --amount (sender-index only)
│   └── evm-balance                 # USDC balance on EVM (private-key only)
│
├── wallet
│   ├── create [--force]            # Generate secp256k1 keypair
│   ├── import <key> [--force]      # Import hex private key
│   ├── address                     # Show address
│   ├── show                        # Full wallet info
│   ├── reset [--force]             # Delete config
│   └── faucet                      # Request test SUI (sender-index only)
│
├── watch                           # WebSocket real-time
│   ├── trades <perpetual_id>
│   ├── book <perpetual_id>
│   ├── bbo <perpetual_id>
│   ├── candles <perpetual_id>      # [--interval]
│   ├── mids
│   ├── user                        # Requires identity
│   └── orders                      # Requires identity
│
├── admin                           # Requires identity (sender-index)
│   ├── setup                       # Create perpetual (--perpetual-id --ticker --atomic-resolution ...)
│   ├── oracle-update               # --perpetual-id --price --exponent
│   ├── funding-update              # --perpetual-id
│   ├── liquidate                   # --target --subaccount
│   ├── setup-vault                 # --perpetual-id [--allocation]
│   └── update-params               # --perpetual-id [--initial-margin-ppm]
│
├── status
│   ├── api                         # Check dex-api
│   └── gateway                     # Check tx-gateway
│
├── setup                           # Interactive wizard
└── shell                           # REPL
```
