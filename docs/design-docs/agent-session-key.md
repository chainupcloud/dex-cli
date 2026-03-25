# Agent / Session Key Design

## Problem

Every trade pops a MetaMask signature dialog. Users trading actively must click "Sign" dozens of times per session. Hyperliquid solves this with a one-time "Enable Trading" signature that authorizes a browser-generated temporary key to sign subsequent trades locally.

## Solution

```
┌─────────────────────────────────────────────────────────────┐
│ One-time: MetaMask signs ApproveAgent                       │
│                                                             │
│   Master wallet (0xAAA)                                     │
│     │                                                       │
│     │ EIP-712 sign: "I authorize 0xBBB to trade for me"    │
│     │               valid_until: 24h / permanent            │
│     ▼                                                       │
│   POST /exchange { action: "approveAgent", signature }      │
│     │                                                       │
│     ▼                                                       │
│   Chain stores: agents[0xBBB] = { master_pubkey, expiry }  │
│                                                             │
│   Browser saves agent private key to localStorage           │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Every trade: Agent key signs locally (no popup)             │
│                                                             │
│   Agent key (0xBBB) in browser memory                       │
│     │                                                       │
│     │ EIP-712 sign: PlaceOrder { perpetualId, side, ... }  │
│     │ (instant, no MetaMask popup)                          │
│     ▼                                                       │
│   POST /exchange { action: "order", signature }             │
│     │                                                       │
│     ▼                                                       │
│   Chain:                                                    │
│     1. Recover signature → agent ETH addr 0xBBB            │
│     2. Lookup agents[0xBBB] → { master_pubkey, expiry }    │
│     3. Check expiry                                         │
│     4. Sui addr = Blake2b256(0x01 || master_pubkey)         │
│     5. SubaccountId = { owner: sui_addr, number: N }       │
│     6. Execute order                                        │
└─────────────────────────────────────────────────────────────┘
```

## On-chain Storage

GlobalAccounts adds one field:

```rust
/// Map from agent ETH address to authorization info.
/// Key: agent's 20-byte ETH address (recovered from agent's EIP-712 signature)
/// Value: master's compressed secp256k1 pubkey + expiry
agents: Map<[u8; 20], AgentAuth>
```

```rust
struct AgentAuth {
    /// Master wallet's compressed secp256k1 public key (33 bytes).
    /// Used to derive Sui address: Blake2b256(0x01 || master_pubkey)
    /// which becomes SubaccountId.owner.
    ///
    /// Why pubkey instead of ETH address:
    ///   ETH addr = keccak256(uncompress)[12:32] — cannot reverse to pubkey
    ///   Sui addr = Blake2b256(0x01 || compressed) — needs the actual pubkey
    ///   So we must store the pubkey to derive SubaccountId at trade time.
    master_pubkey: [u8; 33],

    /// Expiration timestamp in milliseconds. 0 = permanent.
    valid_until_ms: u64,
}
```

Why `agent → master` direction (not `master → Vec<agent>`):
- At trade time, we have the agent address (from signature recovery) and need O(1) lookup to find master
- One agent belongs to exactly one master — natural 1:1 map

## New EIP-712 Types

### ApproveAgent

Signed by **master wallet** (MetaMask, one time):

```
ApproveAgent(uint32 subaccountNumber,address agentAddress,uint64 validUntilMs,uint64 nonce,uint64 deadline)
```

| Field | Type | Description |
|-------|------|-------------|
| subaccountNumber | uint32 | Which subaccount to authorize (0-127) |
| agentAddress | address | Agent's ETH address (20 bytes) |
| validUntilMs | uint64 | Expiry timestamp ms, 0 = permanent |
| nonce | uint64 | Millisecond timestamp, strictly increasing |
| deadline | uint64 | Signature expiry |

### RevokeAgent

Signed by **master wallet**:

```
RevokeAgent(address agentAddress,uint64 nonce,uint64 deadline)
```

## New DexCommands

```rust
/// Authorize an agent key to trade on behalf of the signer.
ApproveAgent {
    global_accounts: Argument,
    signature: Argument,        // Master wallet's Eip712Signature
    params: Argument,           // Eip712ApproveAgentParams
},

/// Revoke a previously authorized agent.
RevokeAgent {
    global_accounts: Argument,
    signature: Argument,        // Master wallet's Eip712Signature
    params: Argument,           // Eip712RevokeAgentParams
},
```

## Modified Order Execution

Current flow:
```
PlaceOrderWithEip712:
  recover signature → signer_addr
  sui_addr = Blake2b256(0x01 || signer_pubkey)
  subaccount = { owner: sui_addr, number: params.subaccount_number }
  execute order
```

New flow:
```
PlaceOrderWithEip712:
  recover signature → signer_addr, signer_pubkey

  if agents[signer_addr] exists:
    // Agent mode
    agent_auth = agents[signer_addr]
    check agent_auth.valid_until_ms == 0 || agent_auth.valid_until_ms > block_time
    sui_addr = Blake2b256(0x01 || agent_auth.master_pubkey)   ← use master's pubkey
  else:
    // Direct mode (backward compatible)
    sui_addr = Blake2b256(0x01 || signer_pubkey)              ← use signer's pubkey

  subaccount = { owner: sui_addr, number: params.subaccount_number }
  execute order
```

Same change applies to: CancelOrderWithEip712, SetLeverageWithEip712.

**NOT** for: BridgeWithdrawWithEip712 — withdrawals always require master signature.

## Permission Boundary

| Operation | Who can sign | Reason |
|-----------|-------------|--------|
| PlaceOrder | Master OR Agent | Trading is the core use case for agent |
| CancelOrder | Master OR Agent | Must be able to cancel what you placed |
| UpdateLeverage | Master OR Agent | Risk management during trading |
| Withdraw | **Master ONLY** | Fund safety — agent key leak must not drain account |
| ApproveAgent | **Master ONLY** | Only the owner can authorize agents |
| RevokeAgent | **Master ONLY** | Only the owner can revoke agents |
| Deposit (bridge) | **Master ONLY** | EVM transaction requires master wallet anyway |

## Frontend Implementation (dex-ui)

### Enable Trading Flow

```typescript
// hooks/useEnableTrading.ts
async function enableTrading() {
  // 1. Generate agent keypair in browser
  const agentPrivateKey = generatePrivateKey();       // viem
  const agentAccount = privateKeyToAccount(agentPrivateKey);
  const agentAddress = agentAccount.address;

  // 2. Build ApproveAgent EIP-712 typed data
  const typedData = {
    domain: { name: "Hermes-Dex", version: "1", chainId: 1, verifyingContract: "0x0..." },
    primaryType: "ApproveAgent",
    types: {
      ApproveAgent: [
        { name: "subaccountNumber", type: "uint32" },
        { name: "agentAddress", type: "address" },
        { name: "validUntilMs", type: "uint64" },
        { name: "nonce", type: "uint64" },
        { name: "deadline", type: "uint64" },
      ]
    },
    message: {
      subaccountNumber: 0,
      agentAddress: agentAddress,
      validUntilMs: 0,               // permanent (or Date.now() + 24h)
      nonce: Date.now(),
      deadline: Date.now() + 3600000, // 1 hour
    }
  };

  // 3. MetaMask signs (THE ONLY POPUP)
  const signature = await walletClient.signTypedData(typedData);

  // 4. Submit to chain
  await postExchange({
    action: { type: "approveAgent", agentAddress, subaccountNumber: 0, validUntilMs: 0 },
    nonce: typedData.message.nonce,
    deadline: typedData.message.deadline,
    signature: parseSignature(signature),
  });

  // 5. Save agent key to localStorage
  localStorage.setItem(`dex-agent-${address.toLowerCase()}`, JSON.stringify({
    privateKey: agentPrivateKey,
    agentAddress,
    masterAddress: address,
    authorizedAt: Date.now(),
  }));
}
```

### Trade Signing (no popup)

```typescript
// hooks/useWalletSign.ts
async function signForTrade(typedData) {
  const stored = localStorage.getItem(`dex-agent-${address.toLowerCase()}`);

  if (stored) {
    // Agent mode: sign locally, instant, no MetaMask
    const { privateKey } = JSON.parse(stored);
    const account = privateKeyToAccount(privateKey);
    return account.signTypedData(typedData);
  } else {
    // Fallback: MetaMask (user hasn't enabled trading)
    return walletClient.signTypedData(typedData);
  }
}
```

### Session Key Lifecycle

```
Connect wallet
    │
    ▼
Check localStorage for agent key
    │
    ├─ Found + not expired → Ready to trade (no popup)
    │
    └─ Not found → Show "Enable Trading" button
                        │
                        ▼
                   MetaMask signs ApproveAgent (one popup)
                        │
                        ▼
                   Save agent key to localStorage
                        │
                        ▼
                   Ready to trade (no popup)

Page refresh → agent key persists in localStorage → still no popup
New tab → localStorage shared → still no popup
Clear browser data → agent key lost → need to re-enable
```

## CLI Implementation (dex-cli)

### New Commands

```bash
# Approve an agent key (signs with master wallet)
dex agent approve --agent-address 0xBBB... --valid-until 24h

# Revoke an agent
dex agent revoke --agent-address 0xBBB...

# List authorized agents
dex agent list

# Trade using agent key (reads agent private key from config or flag)
dex --agent-key 0x... order place --perpetual-id 0 --side buy --quantity 1 --price 66000
```

### Config Extension

```json
{
  "private_key": "0xAAA...",      // master key (for approveAgent, withdraw)
  "agent_key": "0xBBB...",        // agent key (for trading, optional)
  "agent_valid_until": 0           // expiry, 0 = permanent
}
```

## API Changes (dex-api)

### POST /exchange — new actions

```json
// ApproveAgent
{
  "action": { "type": "approveAgent", "agentAddress": "0xBBB...", "subaccountNumber": 0, "validUntilMs": 0 },
  "nonce": 1707580800000,
  "deadline": 1707584400000,
  "signature": { "r": "0x...", "s": "0x...", "v": 28 }
}

// RevokeAgent
{
  "action": { "type": "revokeAgent", "agentAddress": "0xBBB..." },
  "nonce": 1707580800000,
  "deadline": 1707584400000,
  "signature": { "r": "0x...", "s": "0x...", "v": 28 }
}
```

### POST /info — new query

```json
// Query user's authorized agents
{ "type": "userAgents", "user": "0xAAA..." }

// Response
[
  { "agentAddress": "0xBBB...", "validUntilMs": 0, "authorizedAtMs": 1707580800000 }
]
```

## Implementation Phases

| Phase | Scope | Changes |
|-------|-------|---------|
| **1. Chain types** | sui-types | New EIP-712 params (ApproveAgent, RevokeAgent), new DexCommands, AgentAuth struct |
| **2. Chain execution** | sui-core / execution layer | agents Map in GlobalAccounts, ApproveAgent/RevokeAgent execution, modified order validation (agent → master lookup) |
| **3. API** | dex-api | ApproveAgent/RevokeAgent handlers, userAgents query, modified signature verification |
| **4. Frontend** | dex-ui | useEnableTrading rewrite, useWalletSign agent routing, localStorage management |
| **5. CLI** | dex-cli | agent approve/revoke/list commands, --agent-key flag, ExchangeClient changes |

Phase 1-2 are the foundation (chain-side). Phase 3-5 can be parallelized after.

## Backward Compatibility

- Existing direct EIP-712 signatures still work (no agent lookup needed if signer is not in agents map)
- No migration needed — agents map starts empty
- Users who don't enable trading continue signing every trade with MetaMask
