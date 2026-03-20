# polymarket-cli 架构参考

> 源码位置: `../polymarket-cli/src/`

## 核心文件

| 文件 | 行数 | 职责 |
|------|------|------|
| `main.rs` | ~134 | clap CLI 定义，LazyCell 客户端初始化，命令分发 |
| `auth.rs` | ~107 | 钱包/签名解析（resolve_signer, authenticated_clob_client） |
| `config.rs` | ~226 | 配置管理（config.json, resolve_key, 优先级链） |
| `shell.rs` | ~85 | rustyline REPL（split_args, try_parse_from） |

## 命令模块 (commands/)

18 个模块，全部遵循 Args → Subcommand → execute() 模式:

| 模块 | 说明 |
|------|------|
| markets, events, tags, series, comments, profiles, sports | 只读 API 查询 |
| clob | CLOB 交易（下单/撤单/查询/价格/订单簿） |
| approve | 合约授权（ERC20/ERC1155） |
| ctf | 头寸操作（split/merge/redeem） |
| data | 数据查询（持仓/交易量/排行榜） |
| bridge | 跨链充值 |
| wallet | 钱包管理（create/import/show/reset） |
| setup | 交互式向导 |
| upgrade | 自升级 |

## 输出模块 (output/)

16 个模块，每个命令对应一个格式化模块:

- `mod.rs`: OutputFormat enum, truncate(), format_decimal(), print_json(), print_error(), print_detail_table(), detail_field! 宏
- `clob/`: 5 个子模块（prices, orders, books, account, markets）

## 关键设计模式

### LazyCell 客户端初始化

```rust
let gamma = std::cell::LazyCell::new(|| polymarket_client_sdk::gamma::Client::default());
// 仅在需要时初始化
```

### clap ValueEnum 类型转换

```rust
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum CliSide { Buy, Sell }

impl From<CliSide> for sdk::Side {
    fn from(s: CliSide) -> Self { ... }
}
```

### 配置优先级链

```rust
fn resolve_key(cli_flag: Option<&str>) -> (Option<String>, KeySource) {
    // 1. CLI flag
    // 2. 环境变量 POLYMARKET_PRIVATE_KEY
    // 3. 配置文件 ~/.config/polymarket/config.json
    // 4. None
}
```

### 错误处理

```rust
// anyhow 全局
.context("Invalid private key")?
bail!("Address cannot be empty")
ensure!(!address.is_empty(), "...")
```

## 依赖

| Crate | 用途 |
|-------|------|
| polymarket-client-sdk 0.4 | API 客户端 |
| alloy 1.6 | EVM 链交互 |
| clap 4 | CLI |
| tokio 1 | 异步 |
| tabled 0.17 | 表格 |
| rustyline 15 | REPL |
| anyhow 1 | 错误 |
| serde_json 1 | JSON |
| rust_decimal 1 | 精确小数 |
| assert_cmd + predicates | 集成测试 |

## 测试

`tests/cli_integration.rs` (~489 行):
- assert_cmd 驱动 CLI 执行
- predicates 断言输出
- 覆盖: 参数验证、help 输出、错误格式
