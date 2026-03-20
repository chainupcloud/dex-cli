# User Workflows

## 1. Dev Environment Quick Start (sender-index)

```bash
dex status api                                                # Check services
dex --sender-index 0 account mint-usdc --amount 50000         # Mint test USDC
dex --sender-index 0 account deposit --amount 25000           # Deposit to subaccount
dex --sender-index 0 account info                             # Verify
```

## 2. Two-party Trade Test

```bash
# Seller (sender-index 1)
dex --sender-index 1 account mint-usdc --amount 50000
dex --sender-index 1 account deposit --amount 25000
dex --sender-index 1 order place --perpetual-id 0 --side sell --quantity 1 --price 67000

# Buyer (sender-index 2)
dex --sender-index 2 account mint-usdc --amount 50000
dex --sender-index 2 account deposit --amount 25000
dex --sender-index 2 order place --perpetual-id 0 --side buy --quantity 1 --price 67000

# Verify
dex --sender-index 1 position list
dex --sender-index 2 account fills --limit 1
```

## 3. Production Wallet Setup (private-key)

```bash
dex wallet create                        # Generate secp256k1 keypair
dex wallet show                          # Verify address and signing mode
# Fund EVM address with Sepolia ETH + MockUSDC externally
dex account evm-balance                  # Check EVM USDC balance
dex account deposit --amount 1000        # Bridge deposit (2-10 min)
dex account info                         # Verify DEX balance
```

## 4. Trading with EIP-712

```bash
dex market book 0                                            # Check orderbook
dex order place --perpetual-id 0 --side buy --quantity 0.5 --price 66000
dex order list                                               # Verify order
dex position list                                            # After fill
dex position close --perpetual-id 0 --worst-price 60000      # Close
```

## 5. Real-time Monitoring

```bash
# Terminal 1
dex watch trades 0

# Terminal 2
dex --sender-index 0 watch user

# Terminal 3 - trade
dex --sender-index 1 order place ...
```

## 6. Script / Automation

```bash
mid=$(dex -o json market mids | jq -r '.["0"]')
echo "BTC mid: $mid"

# Conditional order
dex -o json order place --perpetual-id 0 --side buy --quantity 0.1 --price 64000
```

## 7. Claude Code Skill

```
User: "BTC 现在什么价?"
→ /dex triggers → dex -o json market mids → "BTC mid: 67,234.5 USDC"

User: "帮我在 66000 挂买单"
→ confirms params → dex -o json order place ... → shows result
```
