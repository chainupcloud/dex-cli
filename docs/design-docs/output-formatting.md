# 输出格式化规范

## 双模式

```rust
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}
```

## 错误输出约定

| 模式 | 正常 | 错误 |
|------|------|------|
| Table | stdout 表格 | stderr 文本 + 非零退出码 |
| JSON | stdout JSON | stdout `{"error":"message"}` + 非零退出码 |

## 公共工具函数

```rust
fn truncate(s: &str, max: usize) -> String;     // Unicode 安全截断，加 "…"
fn format_decimal(n: f64) -> String;             // 12345678 → "12.3M", 1234 → "1.2K"
fn format_date(ts: i64) -> String;               // Unix 时间戳 → RFC3339 UTC
fn print_json<T: Serialize>(data: &T);           // serde_json::to_string_pretty
fn print_error(err: &str, format: OutputFormat);  // 按模式输出错误
fn print_detail_table(rows: Vec<(&str, String)>); // 两列 key-value 布局
```

## 表格示例

### market list

```
╭─────┬────────┬──────────┬──────────┬───────────────┬──────────╮
│ ID  │ Ticker │ Mid Price│ 24h Vol  │ Open Interest │ Funding  │
├─────┼────────┼──────────┼──────────┼───────────────┼──────────┤
│ 0   │ BTC    │ 67,234.5 │ 12.3M    │ 45.6M         │ 0.0100%  │
│ 1   │ ETH    │  3,456.7 │  8.9M    │ 23.4M         │ 0.0050%  │
╰─────┴────────┴──────────┴──────────┴───────────────┴──────────╯
```

### order list

```
╭──────────┬────┬──────┬──────┬──────────┬───────┬─────────╮
│ ClientID │ ID │ Side │ Size │ Price    │ TIF   │ Status  │
├──────────┼────┼──────┼──────┼──────────┼───────┼─────────┤
│ 1001     │ 0  │ BUY  │ 0.5  │ 66,000.0 │ GTC   │ Open    │
╰──────────┴────┴──────┴──────┴──────────┴───────┴─────────╯
```

### position list

```
╭──────┬──────┬──────┬─────────────┬─────────────┬──────────┬───────────╮
│ Perp │ Side │ Size │ Entry Price │ Mark Price  │ Unr. PnL │ Liq Price │
├──────┼──────┼──────┼─────────────┼─────────────┼──────────┼───────────┤
│ BTC  │ LONG │ 1.0  │ 66,000.0    │ 67,234.5    │ +1,234.5 │ 55,000.0  │
╰──────┴──────┴──────┴─────────────┴─────────────┴──────────┴───────────╯
```

### account info (detail table)

```
╭─────────────────────┬─────────────────╮
│ Address             │ 0xabc...def     │
│ Subaccount          │ 0               │
│ Free Collateral     │ 10,000.00 USDC  │
│ Total Account Value │ 12,345.67 USDC  │
│ Margin Used         │ 2,345.67 USDC   │
│ Leverage            │ 3.2x            │
│ Positions           │ 2               │
╰─────────────────────┴─────────────────╯
```

### market book

```
═══ Asks ═══
╭──────────┬──────┬──────────╮
│ Price    │ Size │ Total    │
├──────────┼──────┼──────────┤
│ 67,300.0 │ 2.5  │ 5.0      │
│ 67,250.0 │ 1.5  │ 2.5      │
│ 67,200.0 │ 1.0  │ 1.0      │
╰──────────┴──────┴──────────╯

═══ Bids ═══
╭──────────┬──────┬──────────╮
│ Price    │ Size │ Total    │
├──────────┼──────┼──────────┤
│ 67,100.0 │ 1.2  │ 1.2      │
│ 67,050.0 │ 0.8  │ 2.0      │
│ 67,000.0 │ 3.0  │ 5.0      │
╰──────────┴──────┴──────────╯
```

## WebSocket 输出

- **Table 模式**: 每条事件追加为表格行（流式）
- **JSON 模式**: 每条事件一行 JSON（NDJSON 格式），便于 `jq` 管道处理

## 交易结果输出

成功:
```
✓ Order placed successfully
  Digest: 0xabc123...
```

失败:
```
✗ Order failed: insufficient margin
```

JSON 模式直接输出 GatewayResponse：
```json
{
  "success": true,
  "message": "Order placed successfully",
  "digest": "0xabc123..."
}
```
