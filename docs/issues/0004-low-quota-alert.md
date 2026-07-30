# Issue 0004：低额度预警与系统通知

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

低额度预警端到端可用：任一双窗口用量剩余低于阈值（默认 20%，设置中可调）时，托盘图标变红并推送系统通知；预警与通知各自可关闭；额度恢复后图标恢复原状。通知在 Linux 上走 libnotify/桌面门户，文案中英双语。

## Acceptance criteria

- [ ] 阈值默认 20% 且可调，判定逻辑（含恰好等于阈值的边界）有测试
- [ ] 低于阈值时托盘图标变红；恢复高于阈值时图标恢复
- [ ] 系统通知送达且文案随语言设置切换；通知开关关闭时只变红不推送
- [ ] 预警开关关闭时不变红不推送
- [ ] 手动验收：Ubuntu 24.04 GNOME（Wayland）下图标状态与通知实际可见
- [ ] 质量门禁全绿

## Blocked by

- `docs/issues/0002-credential-storage-and-login.md`
