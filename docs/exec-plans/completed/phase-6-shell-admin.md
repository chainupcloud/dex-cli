# Phase 6: Shell + 管理命令

**状态**: 📋 待开始
**依赖**: Phase 2
**目标**: 交互式 REPL 和开发管理命令

## 任务

- [ ] **6.1** 实现 shell.rs
  - rustyline Editor
  - 提示符 `dex> `
  - split_args() + Cli::try_parse_from()
  - 禁止 shell/setup 嵌套
  - exit/quit 退出
  - 全局选项继承
  - 历史文件 ~/.config/dex/history
- [ ] **6.2** 实现 commands/admin.rs
  - admin setup — POST /tx/setup
  - admin oracle-update — POST /tx/update-oracle-prices
  - admin funding-update — POST /tx/update-funding-rates
  - admin liquidate — POST /tx/liquidate
  - admin setup-vault — POST /tx/setup-vault
  - admin update-params — POST /tx/update-perpetual-params
- [ ] **6.3** 测试: Shell 完整流程, admin 参数验证

## 完成标准

- `dex shell` 进入 REPL, 可执行所有命令
- Shell 历史持久化
- `dex admin setup --perpetual-id 0 --ticker BTC ...` 创建成功
- `dex admin oracle-update --perpetual-id 0 --price 67000 --exponent -1` 成功
