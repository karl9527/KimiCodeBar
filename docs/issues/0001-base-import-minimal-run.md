# Issue 0001：基座合入与 Ubuntu 最小运行（曳光弹）

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

将 Windows 社区版代码库（JYH1878/KimiCodeBar-Windows，Tauri 2 + Rust + React）作为 git remote 合入 `my` 分支，`--allow-unrelated-histories` 保留双方完整历史（ADR-0001），并完成最小 Linux 适配使应用在 Ubuntu 24.04（Wayland）可构建、可运行：

- bundle 目标从 NSIS 改为 deb；应用元数据（publisher、版权）调整为自用构建
- 配置目录从 `%APPDATA%` 改为 XDG 规范（`$XDG_CONFIG_HOME` 或 `~/.config` 下的应用目录），保留 `KIMICODEBAR_CONFIG_DIR` 环境变量覆盖
- 面板按 ADR-0002 改为普通窗口模式：删除托盘图标几何吸附逻辑，托盘左键 → 面板显示在屏幕右上角，失焦自动收起
- 禁用更新检查模块及其入口（ADR-0003）
- 质量门禁（cargo fmt / clippy / test、ESLint）在 Linux 上全绿

本切片是曳光弹：打穿构建、托盘、面板、配置四个整合层，功能正确性留给后续切片。

## Acceptance criteria

- [ ] `git log` 可追溯上游 Windows 版的完整提交历史
- [ ] `npm run tauri dev` 在 Ubuntu 24.04 Wayland 启动成功，托盘图标出现
- [ ] 左键托盘图标 → 面板以普通窗口显示在屏幕右上角；失焦自动收起；右键菜单（刷新/设置/退出）可用
- [ ] 设置项写入 XDG 配置目录；`KIMICODEBAR_CONFIG_DIR` 覆盖生效且有测试
- [ ] 应用内不存在任何更新检查行为与 UI 入口
- [ ] `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test`、ESLint 全部通过
- [ ] `npm run tauri build` 能产出 deb（依赖声明不完善可接受，由 Issue 0008 收口）

## Blocked by

None - can start immediately
