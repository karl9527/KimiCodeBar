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

- [ ] 在干净 Ubuntu 24.04 环境（或卸载重装后的本机）`sudo dpkg -i` 安装成功，依赖自动拉齐
- [ ] 应用菜单出现 KimiCodeBar 条目与图标，从菜单启动正常
- [ ] `sudo dpkg -r` 卸载后程序文件干净移除，XDG 配置目录中的设置与缓存保留；覆盖安装后设置不丢
- [ ] release 构建的质量门禁（fmt / clippy / test / ESLint）全绿
- [ ] PRD 37 条用户故事对应行为逐条手动验收通过，发现的问题清零或降级为后续 issue

## Blocked by

- `docs/issues/0002-credential-storage-and-login.md`
- `docs/issues/0003-local-usage-and-trends.md`
- `docs/issues/0004-low-quota-alert.md`
- `docs/issues/0005-system-integration.md`
- `docs/issues/0006-export-and-diagnostics.md`
- `docs/issues/0007-session-archive.md`
