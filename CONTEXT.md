# KimiCodeBar

Kimi Code 用量监控工具的领域语言。本文件是术语表：统一三平台（macOS / Windows / Ubuntu）对同一概念的称呼，不记录实现细节。

## 用量与配额

**双窗口用量**:
Kimi Code 配额的两个滑动限流窗口——「7天窗口」是周期限额，「5小时窗口」是短时限速；两者均以已用百分比 + 剩余量 + 重置倒计时呈现。
_Avoid_: 周配额、小时配额、速率限制

**月度总量**:
仅 Kimi 网页端接口提供的每月总用量（Kimi 与 Code 两列分开），需用户粘贴网页 cookie token 才可查询，OAuth token 无权访问该接口。
_Avoid_: 月度配额、总量统计

**本地消耗统计**:
扫描本地 wire.jsonl 中的 usage.record 事件汇总出的 token 消耗（今日/昨日、近 7 天柱状图、模型占比）。纯本地数据，不依赖 API。
_Avoid_: 本地用量、离线统计

**会员档位**:
Kimi 会员等级的官方命名，以音乐速度记号由低到高：Andante / Moderato / Allegretto / Allegro，按 API 返回原样显示。

**加油包 (Booster Pack)**:
Kimi Code 的按量付费钱包；订阅额度用完后从预存余额按量扣费。
_Avoid_: 钱包、余额包

## 身份与凭证

**API Key**:
以 `sk-kimi-` 前缀的手工凭证，在 kimi.com/code/console 创建；与开放平台 platform.moonshot.cn 的 `sk-` Key 不互通。两种登录方式之一。

**OAuth 授权**:
与 Kimi Code CLI 相同的设备码授权流程：浏览器一键确认，token 到期自动续期。两种登录方式之一。
_Avoid_: 账号登录、快捷登录

## 会话数据

**wire.jsonl**:
Kimi Code CLI 落盘的会话日志文件（每行一个 JSON 事件），位于 `~/.kimi-code/sessions/` 目录树下；本地消耗统计的唯一数据源。

**会话归档 (Archive)**:
按期限（一天 / 一周 / 一月）将 `~/.kimi-code/sessions` 下的旧会话移出主列表的清理机制。
_Avoid_: 会话清理、历史隐藏

## 应用形态

**托盘 (Tray)**:
系统托盘常驻图标，应用的唯一常驻入口；左键唤起面板，右键菜单（刷新 / 设置 / 退出）。
_Avoid_: 菜单栏（macOS 平台叫法）、状态栏、通知区域

**面板 (Panel)**:
托盘左键唤起的用量详情窗口，失焦自动收起。
_Avoid_: 弹窗、浮层

**CLI 模式**:
以 `--status` 参数运行时直接输出配额 JSON 并退出的无头模式，供脚本 / CI 使用（退出码 0/1/2）。
_Avoid_: 命令行模式
