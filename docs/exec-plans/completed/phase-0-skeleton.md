# Phase 0: 项目骨架

**状态**: ✅ 已完成
**目标**: 可编译、可运行 `dex --help` 的空壳

## 任务

- [ ] **0.1** 初始化 Cargo 项目 (Cargo.toml, .gitignore, binary = "dex")
- [ ] **0.2** 定义顶层 clap 结构
  - `Cli` struct: 全局选项 (`-o`, `--api-url`, `--gateway-url`, `--sender-index`, `--subaccount`, `--env`)
  - `Command` enum: Market, Order, Position, Account, Wallet, Watch, Admin, Status, Setup, Shell
  - 每个命令先用空的 Args/Subcommand 占位
- [ ] **0.3** 创建模块骨架文件
  - `src/config.rs` — 空 struct
  - `src/auth.rs` — 空 struct
  - `src/shell.rs` — 空函数
  - `src/client/{mod,info,gateway,ws}.rs` — 空 struct
  - `src/commands/{mod,market,order,position,account,wallet,watch,admin,status}.rs` — 空 execute()
  - `src/output/{mod,market,order,position,account}.rs` — OutputFormat enum
- [ ] **0.4** 验证编译和 help 输出

## 完成标准

- `cargo build` 无错误
- `dex --help` 显示完整命令树
- `dex market --help` 显示子命令
- 所有模块文件已创建
