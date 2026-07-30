# Issue 0005：系统集成——自启、主题跟随、全局热键降级

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

桌面系统集成三项：

- **开机自启**：设置页一键开关，底层写 XDG autostart desktop 文件（tauri-plugin-autostart）
- **主题跟随系统**：Linux 桌面门户探测深浅色，面板配色跟随；手动切换浅色/深色时覆盖系统设置
- **全局热键**：X11 会话下可录制、可唤起/收起面板（默认关闭）；Wayland 下注册失败时静默禁用并在设置页说明平台限制（ADR-0002 的已接受后果，不视为 bug）

另验证单实例行为：二次启动唤起已有面板而非重复实例。

## Acceptance criteria

- [ ] 自启开关开启后 XDG autostart 文件生成、重启桌面会话应用自启；关闭后文件移除
- [ ] GNOME 切换深浅色后面板配色自动跟随；手动选择主题时不再跟随
- [ ] X11 下热键录制与唤起/收起正常；Wayland 下热键静默禁用且设置页有说明文案（中英双语）
- [ ] 重复启动应用时唤起已有面板，托盘不出现第二个图标
- [ ] 质量门禁全绿

## Blocked by

- `docs/issues/0001-base-import-minimal-run.md`
