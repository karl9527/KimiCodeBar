# Issue 0002：凭证存储与双模式登录（核心闭环）

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

凭证存储 Linux 化，并打通「双模式登录 → 云端用量完整显示」的核心闭环：

- 新增凭证存储抽象层（唯一新测试接缝）：API Key 存 Secret Service（keyring crate 从 `windows-native` 切换为 `secret-service` feature）；检测到 Secret Service 不可用时降级为 0600 权限文件并在设置页给出提示；OAuth 凭证存 0600 权限文件；移除 DPAPI 加密
- OAuth 授权（设备码流程，浏览器一键确认）与 API Key（sk-kimi- 前缀）两种登录方式均可用、可切换，token 到期自动续期
- 面板完整显示：双窗口用量（7天/5小时的已用百分比、剩余量、重置倒计时）、月度总量（粘贴网页 token 后，Kimi/Code 两列）、会员档位与加油包余额
- 自动刷新（默认 5 分钟，1–60 可调）、手动刷新、断网时展示缓存并出现提示横幅
- 系统区域探测从 Win32 API 改为解析 `LANG`/`LC_ALL`，zh 开头判为中文，异常回退英文
- CLI 模式移除 Windows 控制台附加逻辑，`--status` 直接写 stdout/stderr，输出契约（pretty JSON + 退出码 0/1/2）不变

## Acceptance criteria

- [ ] OAuth 授权登录端到端成功，token 过期自动续期；API Key 登录成功；两种方式可在设置中切换
- [ ] API Key 可经 Secret Service 存取；无 Secret Service 的环境降级为 0600 文件且设置页有明确提示
- [ ] 凭证存储抽象层以内存实现覆盖「Secret Service 可用 / 不可用 / 读写失败」三条路径的测试
- [ ] 面板正确显示双窗口用量、月度总量、会员档位、加油包余额
- [ ] 自动刷新间隔可调且生效；手动刷新生效；断网展示缓存 + 横幅、不报错不崩溃
- [ ] `LANG=zh_CN.UTF-8` / `en_US.UTF-8` / 未设置 / 非法值的判定与回退有测试
- [ ] `--status` 输出 pretty JSON、退出码 0/1/2、stdout/stderr 契约有测试
- [ ] 质量门禁全绿

## Blocked by

- `docs/issues/0001-base-import-minimal-run.md`
