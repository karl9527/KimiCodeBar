# Issue 0006：用量导出与诊断日志

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

数据导出与可诊断性：

- 面板一键导出 CSV/JSON 用量记录，写入配置目录下的 exports 子目录，并用系统文件管理器定位（tao/tauri 的 reveal 能力在 Linux 的 file manager 上可用）
- 运行日志按天滚动，落盘于配置目录；日志内容不含任何凭证（API Key、OAuth token、网页 token）
- 一键导出脱敏诊断文件，便于日后反馈问题

## Acceptance criteria

- [ ] CSV 与 JSON 导出内容字段完整、数值与面板显示一致
- [ ] 导出后系统文件管理器打开并定位到 exports 目录
- [ ] 日志按天滚动；对日志目录全文检索 API Key / token 样本值，无任何命中
- [ ] 诊断文件导出成功且已脱敏
- [ ] 质量门禁全绿

## Blocked by

- `docs/issues/0002-credential-storage-and-login.md`
