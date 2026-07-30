# Issue 0007：会话归档（macOS 语义移植）

**标签**: ready-for-agent

## Parent

PRD 0001：`docs/prd/0001-ubuntu-port.md`

## What to build

将 macOS 版独有的会话归档移植到 Ubuntu 版（与上游 Windows 版的有意功能分叉，见 PRD Further Notes）：

- 会话模型：id、工作区哈希、目录名、标题、更新时间、归档态；数据源为 `~/.kimi-code/sessions`
- 期限三档：一天 / 一周 / 一月；达到期限的会话在应用运行期间被自动归档
- 归档语义 = 将会话数据移入归档区（移出 sessions 主列表），恢复 = 移回原位；永不直接删除用户数据
- 「永不失败」原则（与本地消耗统计一致）：目录不存在、单项移动失败均容忍为部分结果
- React 前端新做归档 UI：会话列表、归档区、归档/恢复操作、期限设置；文案中英双语，术语遵循 `CONTEXT.md`（会话归档/Archive）

## Acceptance criteria

- [ ] 归档纯逻辑测试覆盖：期限分类边界（恰好一天/一周/一月）、已归档跳过、目录缺失容忍、单项移动失败容忍
- [ ] 归档 UI 可查看会话列表（标题、目录、相对更新时间）与归档区，可手动归档/恢复单个会话
- [ ] 归档后会话从主列表消失并进入归档区；恢复后数据无损回到原位
- [ ] 达到设定期限的会话被自动归档；期限修改后立即按新规则生效
- [ ] UI 文案中英双语完整，术语与 `CONTEXT.md` 一致
- [ ] 质量门禁全绿

## Blocked by

- `docs/issues/0001-base-import-minimal-run.md`
