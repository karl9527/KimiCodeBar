# 0002 面板放弃吸附托盘图标，统一普通窗口模式

macOS/Windows 版的面板定位在托盘图标上方，依赖托盘图标的几何信息。Linux 上此路不通：libayatana-appindicator 不上报图标位置，且 Wayland 禁止应用自行设置窗口绝对坐标（`set_position` 被合成器忽略）。目标环境 Ubuntu 24.04 默认即 Wayland。

决定：托盘左键唤起面板时，面板作为普通窗口显示在屏幕固定位置（右上角）或记忆位置，失焦自动收起；Wayland 与 X11 共用这一条代码路径。这是有意降级，不是缺陷——不要把它"修复"回吸附式定位。

## Considered Options

- X11 吸附 + Wayland 降级双路径：两条代码路径、测试矩阵翻倍，自用项目不值得。
- GTK Layer Shell 层叠弹出层：可完美吸附，但需在 Tauri 中手写 GTK 集成，复杂度与翻车风险最高。

## Consequences

- 全局热键（tauri-plugin-global-shortcut）在 Wayland 下不可用、X11 下正常；代码保留，接受该限制，不视为 bug。
- Linux AppIndicator 不向应用投递托盘左键点击事件（点击只弹出菜单），故「显示面板」固定为托盘菜单首项作为面板入口；macOS/Windows 的左键切换面板路径保留不受影响。
