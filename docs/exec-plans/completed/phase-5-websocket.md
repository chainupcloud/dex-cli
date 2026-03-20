# Phase 5: WebSocket 实时监听

**状态**: 📋 待开始
**依赖**: Phase 1
**目标**: 实时订阅市场和用户事件

## 任务

- [ ] **5.1** 实现 WsClient
  - WebSocket 连接 ws://{api_url}/ws
  - subscribeChannel 消息发送
  - 持续接收并输出
  - Ctrl-C 优雅退出
- [ ] **5.2** 实现 commands/watch.rs
  - watch trades/book/bbo/candles/mids (市场频道)
  - watch user/orders (用户频道, 需地址)
- [ ] **5.3** 输出格式化
  - Table: 流式表格行
  - JSON: NDJSON (每行一条)
- [ ] **5.4** 手动测试: 启动 watch 后另一终端下单, 验证事件到达

## 完成标准

- `dex watch trades 0` 实时显示成交
- `dex watch book 0` 实时刷新订单簿
- `dex watch user` 显示用户事件
- Ctrl-C 干净退出
- `-o json` 输出 NDJSON
