# PRD 0001：KimiCodeBar Ubuntu 版（Linux 桌面托盘）

术语遵循仓库根目录 `CONTEXT.md`；架构边界遵循 `docs/adr/0001`~`0003`。

## Problem Statement（问题陈述）

Kimi Code 的用量监控工具 KimiCodeBar 目前只有 macOS 版（SwiftUI，官方仓库）和 Windows 社区版（Tauri 2，JYH1878 维护）。作为 Ubuntu 桌面用户（本机 Ubuntu 24.04 Wayland），我没有任何原生途径轻量化地查看自己的双窗口用量、本地消耗统计和加油包余额——只能手动跑 CLI 命令或打开网页查询，无法像其他两平台用户那样"常驻托盘、一眼看额度、快烧完变红提醒"。

## Solution（解决方案）

以 Windows 社区版的 Tauri 2 + Rust + React 代码库为基座（MIT 协议），移植出 Ubuntu 桌面版：常驻系统托盘，左键唤起用量面板，右键菜单直达刷新/设置/退出。功能覆盖 Windows 版全部能力（双窗口用量、月度总量、本地消耗统计、双模式登录、低额度预警、CLI 模式、中英双语、深浅色主题等），并额外搬入 macOS 版独有的会话归档。打包为 deb（x86_64）自用安装；纯自用定位，不做应用内更新与公开发布。

## User Stories（用户故事）

### 托盘与面板

1. 作为 Ubuntu 桌面用户，我想让应用常驻系统托盘，以便随时查看用量而不占用任务栏空间。
2. 作为用户，我想左键单击托盘图标唤起面板，以便查看用量详情。
3. 作为用户，我想面板失焦后自动收起，以便不打断当前工作流。
4. 作为用户，我想通过托盘右键菜单执行刷新/打开设置/退出，以便不打开面板也能完成常用操作。
5. 作为 Wayland 用户，我想面板显示在屏幕固定位置（右上角），以便获得与 X11 一致、可预期的唤起体验。

### 用量查看

6. 作为用户，我想查看 7天窗口的已用百分比、剩余量与重置倒计时，以便规划本周用量。
7. 作为用户，我想查看 5小时窗口的已用百分比、剩余量与重置倒计时，以便判断当前是否撞上限速。
8. 作为用户，我想粘贴一次网页 token 后查看月度总量（Kimi 与 Code 两列分开），以便掌握订阅整体消耗。
9. 作为用户，我想查看本地消耗统计（今日/昨日 token 消耗、近 7 天柱状图、模型占比），以便断网时也能复盘用量。
10. 作为用户，我想查看近 24 小时双窗口用量趋势折线图，以便感知消耗速度。
11. 作为用户，我想查看会员档位与加油包余额、月度已用/限额，以便决定是否需要充值。
12. 作为用户，我想应用默认每 5 分钟自动刷新（1–60 分钟可调），以便数据保持新鲜又不打扰。
13. 作为用户，我想随时手动触发刷新，以便在关键操作前确认额度。
14. 作为用户，我想在任一窗口剩余低于阈值（默认 20%，可调）时托盘图标变红并收到系统通知（可关闭），以便及时止损。
15. 作为用户，我想断网时面板照常展示最近一次成功查询的缓存并给出提示，以便不产生困惑或报错。

### 登录与凭证

16. 作为用户，我想通过 OAuth 授权（设备码流程，浏览器一键确认）登录并自动续期，以便免于手工管理凭证。
17. 作为用户，我想改用 API Key（sk-kimi- 前缀）登录并在两种方式间自由切换，以便适配不同网络环境。
18. 作为用户，我想 API Key 存入 Secret Service 而非明文落盘，以便凭证安全水位不低于其他平台版本。
19. 作为用户，我想在 Secret Service 不可用时降级为 0600 权限文件并在设置页看到提示，以便最小化安装的 Ubuntu 也能用。
20. 作为用户，我想 OAuth 凭证以 0600 权限文件存储（与 Kimi Code CLI 自身做法一致），以便安全与可用兼得。

### 个性化

21. 作为用户，我想界面、通知与托盘提示支持跟随系统/中文/English 三档切换，以便匹配我的语言习惯。
22. 作为用户，我想浅色/深色主题跟随系统或手动切换，以便融入桌面环境。
23. 作为用户，我想从预设渐变色或自定义图片中选择面板背景，以便个性化外观。
24. 作为 X11 用户，我想录制全局热键一键唤起/收起面板（默认关闭），以便键盘流操作；作为 Wayland 用户，我了解该功能受平台限制不可用。

### 数据与诊断

25. 作为用户，我想一键导出 CSV/JSON 用量记录，以便报销与复盘。
26. 作为用户，我想运行日志按天滚动且不含任何凭证，以便排查问题时不泄露隐私。
27. 作为用户，我想一键导出脱敏诊断文件，以便日后反馈问题时附上。

### 系统集成

28. 作为用户，我想在设置里一键开关开机自启，以便登录桌面后无需手动启动。
29. 作为用户，我想重复启动时唤起已有面板而非开第二个实例，以便托盘不出现重复图标。

### CLI 模式

30. 作为用户，我想以 `--status` 运行直接输出配额 JSON 后退出（退出码 0/1/2），以便接入脚本与状态栏工具。

### 会话归档

31. 作为用户，我想查看 `~/.kimi-code/sessions` 下的会话列表（标题、目录、相对更新时间），以便了解磁盘占用来源。
32. 作为用户，我想按期限（一天/一周/一月）自动归档旧会话，以便会话目录不无限膨胀。
33. 作为用户，我想手动归档或恢复单个会话，以便保留主动权。
34. 作为用户，我想归档只是移出主列表而非删除，以便误归档时可无损恢复。

### 安装与更新

35. 作为用户，我想通过 deb 包安装并自动获得应用菜单项与图标，以便与系统其他应用一致。
36. 作为用户，我想卸载 deb 后程序文件干净移除、个人配置仍保留在 XDG 配置目录，以便重装后设置不丢。
37. 作为用户（自用），我接受无应用内更新机制，有新版本时手动下载新 deb 覆盖安装。

## Implementation Decisions（实现决策）

### 代码基座与仓库

- 以 JYH1878/KimiCodeBar-Windows（Tauri 2 + Rust + React，MIT 协议）为代码基座，作为 git remote 合入本仓库 `my` 分支，`--allow-unrelated-histories` 保留双方完整历史（ADR-0001）。
- 保留原有分层：React 前端 + Rust lib crate + Tauri 命令层，模块边界不动，仅替换平台触点。上游 Windows 版的后续更新通过 fetch + merge 持续同步。
- 上游已有的 `cfg(windows)` 依赖隔离（windows-sys 等）保持不变，Linux 构建不编译它们。

### 平台触点适配（对照 Windows 版逐项替换）

- **配置目录**：`%APPDATA%` → XDG 规范（`$XDG_CONFIG_HOME` 或 `~/.config` 下的应用目录）；保留 `KIMICODEBAR_CONFIG_DIR` 环境变量覆盖，作为测试与便携模式的注入点。
- **本地统计数据源**：用户主目录下 `~/.kimi-code/sessions`，路径规则与 Kimi Code CLI 在各平台一致（已在本机验证）。
- **系统区域探测**：Win32 `GetUserDefaultLocaleName` → 解析 `LANG`/`LC_ALL` 环境变量，`zh` 开头判为中文，解析失败回退英文。
- **CLI 模式**：移除 Windows 控制台附加逻辑，直接写 stdout/stderr，输出契约（pretty JSON + 退出码 0/1/2）不变。
- **凭证存储**：keyring crate 从 `windows-native` 切换为 `secret-service` feature；API Key 存 Secret Service；OAuth 凭证与降级场景存 0600 权限文件；移除 DPAPI 加密。新增凭证存储抽象层承载"Secret Service 可用性检测 + 降级选择"逻辑（见 Testing Decisions）。
- **面板定位**：删除托盘图标几何吸附逻辑，改为普通窗口固定位置（右上角）+ 失焦收起，Wayland/X11 单一代码路径（ADR-0002）。
- **托盘**：沿用 Tauri tray-icon（Linux 底层为 libayatana-appindicator）；Ubuntu 24.04 默认预装 AppIndicator 支持。
- **通知**：沿用 tauri-plugin-notification（Linux 走 libnotify/portal）。
- **开机自启**：沿用 tauri-plugin-autostart（Linux 写 XDG autostart desktop 文件）。
- **全局热键**：沿用 tauri-plugin-global-shortcut；注册失败（Wayland）时静默禁用并在设置页说明，不视为 bug。
- **单实例**：沿用 tauri-plugin-single-instance。
- **更新检查模块**：整体禁用（ADR-0003）。

### 会话归档（新增，语义移植自 macOS 版）

- 会话模型：id、工作区哈希、目录名、标题、更新时间、归档态；期限三档：一天/一周/一月。
- 归档语义 = 将会话数据移入归档区（移出 sessions 主列表），恢复 = 移回；永不直接删除用户数据。
- 与本地消耗统计一致的"永不失败"原则：目录不存在、单项移动失败均容忍为部分空结果。
- React 前端新做归档列表与设置 UI，文案遵循 `CONTEXT.md` 术语（会话归档/Archive），中英双语。

### 国际化

- 沿用上游双端方案：后端文案查表 + 前端 i18next；新增用户可见文案一律中文为 key、补英文翻译，术语与既有条目一致。

### 打包与发布

- Tauri bundle 目标只留 deb（x86_64）；声明运行依赖 libwebkit2gtk-4.1 与 libayatana-appindicator3。
- deb 内含桌面文件与图标，安装后出现在应用菜单；卸载不触碰 XDG 配置目录中的用户数据。
- 本机构建或 GitHub Actions ubuntu runner 构建均可；不向任何公开渠道发布（自用）。

## Testing Decisions（测试决策）

### 好测试的标准

只测外部行为（公共 API 的输入输出），不测实现细节；真实 API 响应脱敏后作为 fixture 常驻回归；环境相关逻辑通过环境变量注入（`KIMICODEBAR_CONFIG_DIR`、`LANG`）而非引入 mock 框架。

### 测试接缝

- **主接缝（沿用现有）**：`kimicodebar` lib crate 公共 API，`cargo test` 驱动（模块内联 `#[cfg(test)]` + `tests/` 集成测试 + `fixtures/`）。上游已有 110+ 测试先例：配额响应防御性解析、OAuth 流程纯逻辑、版本比较、配置读写回环。
- **唯一新接缝**：凭证存储抽象层——以内存实现注入，测试 Secret Service 可用/不可用/读写失败三条路径下的选择与降级行为；真实 Secret Service（D-Bus）不进 CI。

### 各模块测试点

- 配置目录解析：`XDG_CONFIG_HOME` 有/无、`KIMICODEBAR_CONFIG_DIR` 覆盖优先级。
- 区域探测：`LANG=zh_CN.UTF-8` / `en_US.UTF-8` / 未设置 / 非法值的判定与回退。
- 归档纯逻辑：期限分类边界（恰好一天/一周/一月）、已归档跳过、目录不存在容忍、单项移动失败容忍。
- 配额与月度总量解析：沿用上游 fixtures，确认零回归。
- CLI 模式：stdout JSON 契约、stderr 干净、退出码 0/1/2。
- 门禁（沿用上游 CI）：`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test`、ESLint。

### 手动验收

GUI 行为（托盘可见性、面板唤起/失焦收起、通知、自启、热键、主题跟随）不做自动化，在本机 Ubuntu 24.04 Wayland 上以验收清单手动 dogfooding。

## Out of Scope（范围之外）

- 应用内更新机制（ADR-0003；自用，手动覆盖安装）。
- AppImage / snap / Flatpak / rpm；aarch64 架构。
- KimiCode CLI 更新探测（macOS 独有，明确不移植）。
- Kimi Web 本地端口探测与启动代理（macOS launchd 独有机制）。
- 面板吸附托盘图标定位（ADR-0002 明确否决，防止日后"修复"回吸附式）。
- Wayland 下全局热键（平台限制，非缺陷）。
- 任何公开发布动作：Release 运营、官网、官方 README 挂链、品牌物料。
- 对 Windows/macOS 版的功能改造（只同步，不改造）。

## Further Notes（备注）

- 开发/验证环境：本机 Ubuntu 24.04.4 LTS（Wayland）+ Kimi Code CLI 已安装，`~/.kimi-code` 数据源就绪。
- 凭证安全水位说明：0600 权限文件与 Kimi Code CLI 自身 `~/.kimi-code/credentials` 的做法一致；API Key 仍进 Secret Service，整体水位不低于 Windows 版。
- 归档功能与上游 Windows 版形成功能分叉（有意为之）；若日后归档进入上游，需以同一套语义对齐。
- 参考文档：`CONTEXT.md`（术语）、`docs/adr/0001`（代码基座）、`docs/adr/0002`（面板定位）、`docs/adr/0003`（自用边界）。
