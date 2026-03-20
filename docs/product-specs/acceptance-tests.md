# 验收测试

## 前置条件

dex-dev Docker 环境运行中:

```bash
cd ../dex-sui/docker/dex-dev && make up
```

确认服务就绪:
- dex-api: `curl -s http://127.0.0.1:9100/info -d '{"type":"meta"}'`
- tx-gateway: `curl -s http://127.0.0.1:3200/tx/status`

---

## AT-0: 项目骨架

| ID | 命令 | 期望 |
|----|------|------|
| 0.1 | `cargo build` | 编译成功 |
| 0.2 | `dex --help` | 显示完整命令树 (market, order, position, account, wallet, watch, admin, status, setup, shell) |
| 0.3 | `dex market --help` | 显示子命令 |
| 0.4 | `dex --version` | 输出版本号 |

## AT-1: 市场数据

| ID | 命令 | 期望 |
|----|------|------|
| 1.1 | `dex market list` | 表格显示永续合约列表 |
| 1.2 | `dex -o json market list` | 合法 JSON（可 `\| jq .`） |
| 1.3 | `dex market book 0` | 显示 Asks/Bids 表格 |
| 1.4 | `dex market trades 0` | 最近成交表格 |
| 1.5 | `dex market candles 0 --interval 1h` | K 线表格 |
| 1.6 | `dex market mids` | 所有中间价 |
| 1.7 | `dex market stats 0` | 市场统计 detail table |
| 1.8 | `dex --api-url http://127.0.0.1:19999 market list` | 友好连接失败提示 |
| 1.9 | `dex --api-url http://... -o json market list` | `{"error":"..."}` JSON |
| 1.10 | `dex market candles 0 --interval invalid` | clap 报错提示有效值 |

## AT-2: 钱包与配置

| ID | 命令 | 期望 |
|----|------|------|
| 2.1 | `dex wallet create` | 输出地址，config.json 权限 0o600 |
| 2.2 | `dex wallet create` (已有) | 提示使用 `--force` |
| 2.3 | `dex wallet create --force` | 覆盖成功 |
| 2.4 | `dex wallet import <key>` | 导入成功 |
| 2.5 | `dex wallet address` | 显示地址 |
| 2.6 | `dex wallet show` | 显示详情，不含私钥明文 |
| 2.7 | `dex --sender-index 0 wallet address` | 显示 gateway 第 0 个地址 |
| 2.8 | `dex --sender-index 5 wallet address` | 显示不同地址 |
| 2.9 | `dex wallet faucet` | 成功获取 SUI |
| 2.10 | `dex wallet reset --force` | 删除配置 |
| 2.11 | `dex status api` | 显示连通状态 |
| 2.12 | `dex status gateway` | 显示 gateway 状态和地址 |
| 2.13 | `DEX_API_URL=http://...:9999 dex market list` | 使用环境变量 URL |
| 2.14 | `DEX_API_URL=... dex --api-url http://...:9100 market list` | flag 优先于 env |

## AT-3: 交易操作

**前置**: 已创建合约、设置价格、充值

| ID | 命令 | 期望 |
|----|------|------|
| 3.1 | `dex order place --perpetual-id 0 --side buy --quantity 1 --price 60000` | 成功 + digest |
| 3.2 | `dex order place ... --order-type market` | 市价单成功 |
| 3.3 | `dex order list` | 显示委托表格 |
| 3.4 | `dex order list --perpetual-id 0` | 按合约过滤 |
| 3.5 | `dex order cancel --perpetual-id 0 --client-id <id>` | 撤单成功 |
| 3.6 | `dex order history --limit 10` | 历史委托 |
| 3.7 | 两个 sender-index 下对手单 | 成交，双方 fills/position 反映 |
| 3.8 | `dex position list` | 持仓表格 |
| 3.9 | `dex position close --perpetual-id 0 --worst-price 50000` | 平仓成功 |
| 3.10 | `dex position leverage --perpetual-id 0 --leverage 5` | 成功 |
| 3.11 | 无钱包执行交易命令 | 提示 "Run 'dex wallet create'" |
| 3.12 | `dex -o json order place ...` | JSON 包含 success/message/digest |

## AT-4: 账户管理

| ID | 命令 | 期望 |
|----|------|------|
| 4.1 | `dex account mint-usdc --amount 10000` | 成功 |
| 4.2 | `dex account deposit --amount 5000` | 成功 + digest |
| 4.3 | `dex account info` | detail table (Address, Collateral, Value, Margin) |
| 4.4 | `dex account fills --limit 5` | 成交表格 |
| 4.5 | `dex account balances` | 余额变动表格 |
| 4.6 | `dex account transfers` | 划转记录 |
| 4.7 | `dex account withdraw --amount 1000` | 成功 |
| 4.8 | `dex --subaccount 1 account info` | 不同子账户 |

## AT-5: WebSocket

| ID | 命令 | 期望 |
|----|------|------|
| 5.1 | `dex watch trades 0` + 另一终端下单 | 实时显示成交 |
| 5.2 | `dex watch book 0` | 显示初始快照 + 实时更新 |
| 5.3 | `dex watch bbo 0` | BBO 更新 |
| 5.4 | `dex watch mids` | 中间价更新 |
| 5.5 | `dex watch user` | 用户事件 |
| 5.6 | `dex -o json watch trades 0` | NDJSON 格式 |
| 5.7 | Ctrl-C | 干净退出 |
| 5.8 | 连接失败 URL | 友好提示 |

## AT-6: Shell + 管理

| ID | 命令 | 期望 |
|----|------|------|
| 6.1 | `dex shell` | 显示 `dex> ` 提示符 |
| 6.2 | Shell 内 `market list` | 正常执行 |
| 6.3 | Shell 内 `exit` | 干净退出 |
| 6.4 | Shell 内 `shell` | 提示禁止嵌套 |
| 6.5 | `dex --sender-index 3 shell` → `wallet address` | 继承 sender-index |
| 6.6 | Shell 历史 (上箭头) | 显示上次命令 |
| 6.7 | `dex admin setup --perpetual-id 1 --ticker ETH ...` | 创建成功 |
| 6.8 | `dex admin oracle-update --perpetual-id 0 --price 68000 --exponent -1` | 成功 |
| 6.9 | `dex admin funding-update --perpetual-id 0` | 成功 |

## AT-7: 质量

| ID | 检查 | 期望 |
|----|------|------|
| 7.1 | `cargo test` | 全部通过 |
| 7.2 | `cargo fmt --all -- --check` | 无 diff |
| 7.3 | `cargo clippy -- -D warnings` | 零警告 |
| 7.4 | 所有命令 `-o json \| jq .` | 全部合法 JSON |

## 端到端完整流程

在干净环境中走完全部交易流程:

```bash
# 0. 环境
cd ../dex-sui/docker/dex-dev && make clean && make up

# 1. 检查
dex status api && dex status gateway

# 2. 创建合约
dex admin setup --perpetual-id 0 --ticker BTC --atomic-resolution -8 \
  --initial-margin-ppm 50000 --quantum-conversion-exponent -8 \
  --subticks-per-tick 1000 --step-base-quantums 10000000

# 3. 预言机
dex admin oracle-update --perpetual-id 0 --price 67000 --exponent -1

# 4. 准备账户
dex --sender-index 1 wallet faucet
dex --sender-index 1 account mint-usdc --amount 100000
dex --sender-index 1 account deposit --amount 50000
dex --sender-index 2 wallet faucet
dex --sender-index 2 account mint-usdc --amount 100000
dex --sender-index 2 account deposit --amount 50000

# 5. 查看市场
dex market list && dex market book 0

# 6. 交易
dex --sender-index 1 order place --perpetual-id 0 --side sell --quantity 1 --price 67000
dex --sender-index 2 order place --perpetual-id 0 --side buy --quantity 1 --price 67000

# 7. 验证
dex --sender-index 1 account fills
dex --sender-index 2 account fills
dex --sender-index 1 position list
dex --sender-index 2 position list
dex --sender-index 1 account info

# 8. 平仓
dex --sender-index 1 position close --perpetual-id 0 --worst-price 70000
dex --sender-index 2 position close --perpetual-id 0 --worst-price 60000

# 9. 验证最终状态
dex --sender-index 1 position list   # 应为空
dex --sender-index 1 account info    # 余额反映 PnL
```

**期望**: 全流程无报错，每步输出符合预期。
