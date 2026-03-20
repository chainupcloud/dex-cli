# Tech Debt Tracker

| ID | Priority | Description | Notes |
|----|----------|-------------|-------|
| TD-1 | P2 | Response types duplicated from dex-types | Accepted trade-off for independent compilation |
| TD-2 | P2 | WebSocket no auto-reconnect | Exits on disconnect, user must restart |
| TD-3 | P2 | quantums/subticks conversion in EIP-712 mode | Currently passes raw values; need atomic_resolution + QCE conversion for human-readable prices |
| TD-4 | P3 | No shell command auto-completion | clap `generate` feature can add bash/zsh/fish completion |
| TD-5 | P3 | No self-upgrade command | Future: download from GitHub releases |
| TD-6 | P3 | No batch order placement | Future: `order place-batch` from JSON file |
| TD-7 | P3 | EIP-712 withdraw not implemented | Currently withdraw goes through gateway only |
| TD-8 | P3 | MockUSDC mint not exposed for EVM | Users need external tools to mint Sepolia MockUSDC |

## Priority Legend

- P1: Affects correctness or security — fix ASAP
- P2: Affects user experience — plan to fix
- P3: Feature enhancement — on demand
