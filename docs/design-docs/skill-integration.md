# Claude Code Skill Integration

## Overview

The `dex` skill lets Claude Code users interact with the DEX through natural language. It is maintained in the repository at `.claude/skills/dex/SKILL.md` so anyone who clones the repo gets it automatically.

## File Structure

```
.claude/skills/dex/
└── SKILL.md          # Required: YAML frontmatter + instructions
```

## Frontmatter

```yaml
name: dex
description: Interact with DEX on Sui via dex CLI. Triggers on market data queries, order placement, position management, ...
argument-hint: [command-or-question]
allowed-tools: Bash(dex *)
```

| Field | Value | Purpose |
|-------|-------|---------|
| `name` | `dex` | Invoke via `/dex` |
| `description` | Trading keywords | Claude auto-triggers when user mentions trading topics |
| `argument-hint` | `[command-or-question]` | Autocomplete hint |
| `allowed-tools` | `Bash(dex *)` | Grants permission to run any `dex` command without per-call confirmation |

## Design Decisions

### `allowed-tools: Bash(dex *)`

All `dex` commands run without tool-confirmation prompts. Rationale:
- `dex` operates against a local dev/test environment
- Per-command confirmation breaks trading workflow
- The user already opted in by invoking `/dex`

### Always `-o json`

Every command in the skill uses `-o json`. Claude parses the structured output and responds in natural language. Users never see raw JSON.

### Dangerous Operation Confirmation

Even though `allowed-tools` skips tool prompts, the skill instructions require Claude to confirm parameters in text before executing orders, close-position, and withdrawals.

### Auto vs Manual Trigger

- **Auto**: `description` contains keywords like "order", "position", "balance" — Claude loads the skill when the user's message matches
- **Manual**: User types `/dex show me BTC orderbook`
- `disable-model-invocation` is not set, so both modes work

## Extension

Additional reference files can be added to the skill directory:

```
.claude/skills/dex/
├── SKILL.md
├── perpetual-ids.md    # Contract ID mapping (auto-generated)
└── error-guide.md      # Common error solutions
```

Reference them in SKILL.md via `[perpetual-ids.md](perpetual-ids.md)` — Claude reads them on demand.
