# Phase 7: Skill + 打磨

**状态**: 📋 待开始
**依赖**: 所有前序阶段
**目标**: Claude Code 集成 + 完善文档和测试

## 任务

- [ ] **7.1** 创建 Claude Code Skill
  - `.claude/skills/dex-trading.md`
  - 自然语言→CLI 命令映射
- [ ] **7.2** 完善集成测试
  - tests/cli_integration.rs 覆盖所有命令参数验证
  - help 输出测试
  - 错误格式测试
- [ ] **7.3** README.md
  - 安装、快速入门、命令参考、配置说明
- [ ] **7.4** 代码审查和清理
  - cargo fmt, cargo clippy 零警告
  - 删除未使用代码
  - 注释完善

## 完成标准

- Claude Code 可通过 skill 调用 dex CLI
- `cargo test` 全部通过
- `cargo clippy -- -D warnings` 零警告
- README 完整
