# Phase 1: 只读查询

**状态**: 📋 待开始
**依赖**: Phase 0
**目标**: 无需钱包即可查询市场数据

## 任务

- [ ] **1.1** 实现 InfoClient
  - HTTP POST /info 封装
  - 请求/响应类型定义 (MetaResponse, L2BookResponse, AllMidsResponse, RecentFillsResponse, CandleSnapshotResponse, MarketStatsResponse)
  - 错误处理: 连接失败、HTTP 错误、JSON 解析错误
- [ ] **1.2** 实现 output/mod.rs 核心工具
  - OutputFormat enum + ValueEnum
  - truncate(), format_decimal(), format_date()
  - print_json(), print_error(), print_detail_table()
- [ ] **1.3** 实现 commands/market.rs
  - market list — meta() + all_mids() 合并
  - market info — meta() 过滤 + market_stats()
  - market book — l2_book()
  - market trades — recent_fills()
  - market candles — candle_snapshot()
  - market stats — market_stats()
  - market mids — all_mids()
- [ ] **1.4** 实现 output/market.rs 表格格式化
- [ ] **1.5** main.rs 连线: 全局选项 → InfoClient → commands::market::execute()
- [ ] **1.6** 集成测试: 参数验证、help 输出、手动烟雾测试

## 完成标准

- `dex market list` 显示合约列表 (table + json)
- `dex market book 0` 显示订单簿
- `dex -o json market mids` 输出合法 JSON
- API 连接失败时显示友好错误
