# Phase 2: 钱包与配置

**状态**: 📋 待开始
**依赖**: Phase 0
**可并行**: 与 Phase 1 并行
**目标**: 管理密钥和配置持久化

## 任务

- [ ] **2.1** 实现 config.rs
  - DexConfig struct + JSON 序列化
  - config_path(), load_config(), save_config()
  - resolve_*() 优先级链函数
  - 文件权限: 目录 0o700, 文件 0o600
- [ ] **2.2** 实现 auth.rs
  - Identity 枚举: SenderIndex(u32) / PrivateKey(String) / None
  - resolve_identity() — 从 CLI + env + config 解析
  - resolve_address() — 从 identity 获取地址
- [ ] **2.3** 实现 commands/wallet.rs
  - wallet create/import/address/show/reset/faucet
- [ ] **2.4** 实现 commands/status.rs
  - status api — 检查 dex-api
  - status gateway — GET /tx/status
- [ ] **2.5** 实现 setup 交互式向导
- [ ] **2.6** 集成测试

## 完成标准

- `dex wallet create` 生成密钥并保存
- `dex wallet show` 显示信息 (不含私钥明文)
- `dex --sender-index 3 wallet address` 显示 gateway 第 3 个地址
- `dex status gateway` 显示连通性
- 配置文件权限 0o600
