# Phase 4: 账户管理

**状态**: 📋 待开始
**依赖**: Phase 3
**目标**: 完整的账户管理功能

## 任务

- [ ] **4.1** 实现 commands/account.rs
  - account info — clearinghouse_state(), detail table
  - account fills — user_fills()
  - account balances — user_balances()
  - account transfers — user_transfers()
  - account deposit — GatewayClient deposit()
  - account withdraw — GatewayClient withdraw()
  - account mint-usdc — GatewayClient mint_usdc()
- [ ] **4.2** 实现 output/account.rs
  - 账户总览 detail table
  - 成交记录表格 (Time, Perp, Side, Size, Price, Fee)
  - 余额变动表格
  - 充值/提现结果
- [ ] **4.3** 端到端测试: 铸造→充值→查余额→提现

## 完成标准

- `dex account info` 显示账户总览
- `dex account fills` 显示成交历史
- `dex account deposit --amount 1000` 充值成功
- `dex account mint-usdc --amount 10000` 铸造成功
