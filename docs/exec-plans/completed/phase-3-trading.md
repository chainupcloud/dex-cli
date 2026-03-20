# Phase 3: 交易操作

**状态**: 📋 待开始
**依赖**: Phase 2
**目标**: 通过 tx-gateway 提交交易

## 任务

- [ ] **3.1** 实现 GatewayClient
  - place_order, cancel_order, close_position, set_leverage
  - deposit, withdraw, mint_usdc, faucet
  - status, addresses
  - GatewayResponse 统一响应解析
- [ ] **3.2** 实现 commands/order.rs
  - order place (ValueEnum: side, order-type, time-in-force)
  - order cancel
  - order list (→ InfoClient open_orders)
  - order history (→ InfoClient historical_orders)
  - order status (→ InfoClient order_status)
- [ ] **3.3** 实现 commands/position.rs
  - position list (→ InfoClient clearinghouse_state)
  - position close (→ GatewayClient)
  - position leverage (→ GatewayClient)
- [ ] **3.4** 实现 output/order.rs + output/position.rs
- [ ] **3.5** 身份验证集成: 交易命令检查 identity, 无钱包时友好提示
- [ ] **3.6** 端到端测试 (需 dex-dev): 下单→查委托→撤单; 对手单→成交→持仓→平仓

## 完成标准

- `dex order place --perpetual-id 0 --side buy --quantity 1 --price 50000` 成功
- `dex order list` 显示委托
- `dex order cancel` 撤单成功
- `dex position list` 显示持仓
- `dex position close --perpetual-id 0` 平仓成功
- 无钱包时提示 "Run 'dex wallet create'"
