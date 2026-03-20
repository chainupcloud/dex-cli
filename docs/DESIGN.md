# DESIGN.md — Design Decisions

## 1. Independent from dex-sui

dex-cli does not depend on any dex-sui internal crate. All communication is via HTTP:
- Queries → dex-api POST /info
- EIP-712 trades → dex-api POST /exchange
- Dev transactions → tx-gateway POST /tx/*
- Bridge deposits → Sepolia EVM RPC

Trade-off: response types are redefined in `client/info.rs` (duplicated from dex-types). Acceptable for independent compilation and distribution.

## 2. Dual Identity Mode

| Mode | Signing | Transaction Path | Deposit Path |
|------|---------|------------------|-------------|
| `--sender-index N` | Gateway internal | POST /tx/* | Gateway mint+deposit |
| `--private-key` | Local secp256k1 EIP-712 | POST /exchange | EVM bridge contract |

Same CLI, same commands — the identity mode determines which backend path is used. Users can start with sender-index for dev, then switch to private-key for production.

## 3. EIP-712 Signing (same as dex-ui frontend)

Private-key mode signs orders/cancels/leverage with EIP-712 typed data, matching the dex-ui frontend exactly:
- Domain: Hermes-Dex v1, chainId=1
- Types: PlaceOrder (14 fields), CancelOrder (5), UpdateLeverage (6)
- Signature: secp256k1 → {r, s, v} → POST /exchange

## 4. Cross-chain Bridge Deposit

Private-key mode deposits via the Sepolia bridge contract:
1. IERC20.approve(bridge, amount)
2. ISuiBridge.depositUSDCForSubaccount(suiAddress, subaccount, amount)
3. Bridge node confirms in 2-10 minutes

The same secp256k1 key derives both ETH address (for EVM) and Sui address (Blake2b256 for DEX).

## 5. Table/JSON Dual Output

All commands support `-o table` (human) and `-o json` (machine/skill). The Claude Code skill uses `-o json` exclusively.

## Design Docs

| Document | Content |
|----------|---------|
| [core-beliefs.md](design-docs/core-beliefs.md) | Core design principles |
| [cli-commands.md](design-docs/cli-commands.md) | Full command tree |
| [client-layer.md](design-docs/client-layer.md) | Client interfaces |
| [config-and-auth.md](design-docs/config-and-auth.md) | Config + identity |
| [output-formatting.md](design-docs/output-formatting.md) | Output formatting |
| [skill-integration.md](design-docs/skill-integration.md) | Claude Code skill |
