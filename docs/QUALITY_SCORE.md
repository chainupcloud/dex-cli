# QUALITY_SCORE.md

## Code Quality

| Metric | Standard | Current |
|--------|----------|---------|
| `cargo build` | Zero errors | ✅ Pass (7 dead_code warnings — expected for unused-yet methods) |
| `cargo test` | 100% pass | ✅ 11/11 pass |
| `cargo fmt` | Zero diff | ✅ |

## Test Coverage

| Layer | Scope | Tool |
|-------|-------|------|
| Argument validation | All commands' required params, enum values | assert_cmd + predicates |
| Help output | All commands and subcommands | assert_cmd |
| Identity enforcement | Order/account commands require identity | assert_cmd with isolated HOME |
| End-to-end | Full trading flow (needs dex-dev) | Manual / script |

## User Experience

| Metric | Standard |
|--------|----------|
| No-wallet prompt | Trading commands show "Run 'dex wallet create'" |
| Connection failure | Shows URL and "Is the service running?" |
| Empty data | Shows "No X found.", never errors |
| JSON validity | All `-o json` output parseable by `jq` |
| Help completeness | Every command has description, every param has help text |
| Bridge deposit | Shows EVM tx hash + estimated arrival time |

## Consistency

| Dimension | Standard |
|-----------|----------|
| Command pattern | Args → Subcommand → execute() in every command file |
| Output pattern | All commands support table and json |
| Error pattern | anyhow + context everywhere |
| Identity pattern | Dual-mode (EIP-712 / gateway) in order.rs, position.rs, account.rs |
| Naming | CLI flags: kebab-case, Rust code: snake_case |
