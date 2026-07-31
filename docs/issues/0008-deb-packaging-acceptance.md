# Issue 0008：deb 打包与安装验收（v1 收口）

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

v1 收口切片：完善 deb 打包并完成全链路安装验收与 dogfooding。

- deb 声明运行依赖 libwebkit2gtk-4.1 与 libayatana-appindicator3
- 桌面文件与图标随包装入，安装后应用菜单出现 KimiCodeBar 条目
- 安装 / 卸载 / 覆盖安装全链路验证；卸载不触碰 XDG 配置目录中的用户数据
- 对照 PRD 的 37 条用户故事逐条手动验收（本机 Ubuntu 24.04 Wayland）

## Acceptance criteria

- [x] 在干净 Ubuntu 24.04 环境（或卸载重装后的本机）`sudo dpkg -i` 安装成功，依赖自动拉齐（deb 控制信息 Depends: libayatana-appindicator3-1, libwebkit2gtk-4.1-0, libgtk-3-0）
- [x] 应用菜单出现 KimiCodeBar 条目与图标，从菜单启动正常
- [x] `sudo dpkg -r` 卸载后程序文件干净移除，XDG 配置目录中的设置与缓存保留（credentials/settings/cache/history 全数实证）；覆盖安装后设置不丢
- [x] release 构建的质量门禁（fmt / clippy / test / ESLint）全绿
- [x] PRD 37 条用户故事对应行为逐条手动验收通过：1–5 → Issue 0001+本切片修复；6–15 → 0002/0003/0004；16–20 → 0002；21–23 → 0005+本切片补验；24 → Wayland 降级已实证（ADR-0002 已接受，X11 侧未实测）；25–27 → 0006；28–29 → 0005；30 → 0002；31–34 → 0007；35–36 → 本切片；37 → ADR-0003 有意无机制

## 本切片修复的三个平台级 bug（重要）

1. **AppIndicator 不投递托盘左键点击**：点击只弹菜单，面板无入口。修复：托盘菜单首项固定「显示面板」（Linux AppIndicator 惯例）。
2. **tao/Wayland 窗口重显 bug**：初始未映射的窗口（visible:false）后续 show()/hide() 可能静默失败（证据：show=Ok 但 is_visible=false）。修复：两个窗口一律 visible:true 创建、setup 完成首次映射配置后立即隐藏。
3. **正式二进制必须用 `npm run tauri build`**：裸 `cargo build --release` 产出的二进制在 webview 里加载 devUrl（connection refused），不可用于验证或分发。

## Blocked by

- `docs/issues/0002-credential-storage-and-login.md`
- `docs/issues/0003-local-usage-and-trends.md`
- `docs/issues/0004-low-quota-alert.md`
- `docs/issues/0005-system-integration.md`
- `docs/issues/0006-export-and-diagnostics.md`
- `docs/issues/0007-session-archive.md`
