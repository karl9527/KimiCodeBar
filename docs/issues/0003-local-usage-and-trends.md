# Issue 0003：本地消耗统计与用量趋势

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

打通本地数据链路：本地消耗统计的数据源路径从 Windows 用户目录推导改为 Linux 用户主目录下的 `~/.kimi-code/sessions`（与 Kimi Code CLI 在各平台的落盘位置一致，本机已验证）。

- 面板正确显示：今日/昨日 token 消耗、近 7 天柱状图、模型占比
- 用量趋势：近 24 小时双窗口折线图，历史在本地记录 7 天，纯事实不预测
- 保持上游「永不失败」原则：sessions 目录不存在、单文件读失败、状态写失败均容忍为（部分）空结果
- 本切片不依赖登录态，断网时本地统计照常可用

## Acceptance criteria

- [ ] 本机真实 `~/.kimi-code/sessions` 数据被增量扫描并正确统计，面板今日/昨日/柱状图/模型占比显示正确
- [ ] sessions 目录不存在 / 单文件损坏 / 状态写失败的容忍行为有测试
- [ ] 近 24 小时双窗口趋势折线正常显示；本地历史记录 7 天
- [ ] 断网状态下本地统计与趋势照常展示
- [ ] 质量门禁全绿

## Blocked by

- `docs/issues/0001-base-import-minimal-run.md`
