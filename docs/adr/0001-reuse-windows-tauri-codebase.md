# 0001 复用 Windows 社区版 Tauri 代码库，双 remote 合并保留历史

本仓库（karl9527/KimiCodeBar）fork 自 xifandev/KimiCodeBar（macOS SwiftUI 版），但 `my` 分支开发的 Ubuntu 版不复用该仓库的任何 Swift 代码：SwiftUI 在 Linux 上不存在，业务逻辑又强绑 macOS API，无法移植。

决定：以 JYH1878/KimiCodeBar-Windows（MIT 协议，Tauri 2 + Rust + React）为代码基座，将其作为 git remote fetch 后以 `--allow-unrelated-histories` 合入 `my` 分支。理由：该代码库的 OAuth 设备码流程、配额防御性解析、wire.jsonl 本地统计等业务逻辑本已跨平台，且经 110+ 测试固化；保留完整 git 历史使日后 Windows 版的修复与功能可以直接 fetch + merge 同步（约 95% 代码共享）。

## Considered Options

- 从零自研：业务逻辑的边界情况（配额字段别名、金额单位换算、OAuth 续期）需重新踩坑，否决。
- 快照式复制代码：丢失上游历史，日后同步只能手工 diff，否决。
- 直接给上游提 PR 做跨平台版：节奏依赖上游维护者意愿，自用项目等不起，否决（但保留日后反向贡献的可能）。
