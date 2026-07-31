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

- [x] CSV 与 JSON 导出内容字段完整、数值与面板显示一致（用户确认 + 抽查 usage-*.csv 与 history.json 副本）
- [x] 导出后系统文件管理器打开并定位到 exports 目录（tauri-plugin-opener reveal，Nautilus 实测两次打开正常）
- [x] 日志按天滚动（`logs/kimicodebar.log.YYYY-MM-DD`）；对日志目录全文检索 API Key / token 样本值，无任何命中（真实 access/refresh token + sk-kimi-/Bearer/Authorization 模式扫描均 0 命中）
- [x] 诊断文件导出成功且已脱敏（`mask_sensitive` 内置脱敏；诊断文件+导出物凭证扫描 0 命中）
- [x] 质量门禁全绿

**实施说明**：本切片在 Linux 上零代码改动通过验收——导出（local_usage::export_usage_report）、诊断（diagnostics.rs 的 OS 字段用 `std::env::consts::OS` 自动适配）、按天滚动日志（tracing-appender）与 reveal（tauri-plugin-opener）均为跨平台实现。

## Blocked by

- `docs/issues/0002-credential-storage-and-login.md`
