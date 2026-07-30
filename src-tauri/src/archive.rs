//! 会话归档：按期限将 `~/.kimi-code/sessions` 下的旧会话标记为归档——
//! 仅改写会话 state.json 的 archived 标志（顶层与 custom 内各一份，
//! Kimi Code CLI 原生识别该标志并从会话列表隐藏），不移动、不删除任何数据。
//! 语义移植自 macOS 版 KimiArchiveManager.swift（术语见 CONTEXT.md「会话归档」）。

use std::path::Path;

use serde::Serialize;

use crate::local_usage;

/// 归档期限（与 macOS 版 ArchiveThreshold 一致；字符串取值是设置项与前端的契约）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveThreshold {
    OneDay,
    OneWeek,
    OneMonth,
}

impl ArchiveThreshold {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchiveThreshold::OneDay => "oneDay",
            ArchiveThreshold::OneWeek => "oneWeek",
            ArchiveThreshold::OneMonth => "oneMonth",
        }
    }

    /// 非法值返回 None（设置项校验用）
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "oneDay" => Some(ArchiveThreshold::OneDay),
            "oneWeek" => Some(ArchiveThreshold::OneWeek),
            "oneMonth" => Some(ArchiveThreshold::OneMonth),
            _ => None,
        }
    }

    pub fn ms(&self) -> i64 {
        match self {
            ArchiveThreshold::OneDay => 24 * 60 * 60 * 1000,
            ArchiveThreshold::OneWeek => 7 * 24 * 60 * 60 * 1000,
            ArchiveThreshold::OneMonth => 30 * 24 * 60 * 60 * 1000,
        }
    }
}

/// 会话（与 src/types.ts 的 ArchiveSession 一一对应）
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ArchiveSession {
    /// 会话目录名（ses_<uuid>）
    pub id: String,
    /// 工作区目录名（wd_<name>_<hash>）
    pub workspace_hash: String,
    /// 工作区显示名：state.json 的 workDir / custom.cwd 末段，回退 workspace_hash
    pub folder_name: String,
    /// 会话标题：state.json 的 title，回退会话目录名
    pub title: String,
    /// 最后更新时间（epoch 毫秒）
    pub updated_at_ms: i64,
    pub is_archived: bool,
    /// 会话目录完整路径（前端归档/恢复操作的标识）
    pub path: String,
}

/// 扫描结果（updatedAt 降序）。永不失败：根目录不存在视为正常的空结果
#[derive(Debug, Clone, Serialize, Default)]
pub struct ScanResult {
    pub sessions: Vec<ArchiveSession>,
    pub error: Option<String>,
}

/// 扫描默认会话根目录（~/.kimi-code/sessions）
pub fn scan_sessions() -> ScanResult {
    match local_usage::sessions_dir() {
        Some(root) => scan_sessions_in(&root),
        None => ScanResult::default(),
    }
}

/// 扫描指定根目录下的全部会话（含已归档；无 state.json / 无法解析的目录跳过）
pub fn scan_sessions_in(root: &Path) -> ScanResult {
    let mut result = ScanResult::default();
    let Ok(workspaces) = std::fs::read_dir(root) else {
        // 根目录不存在/不可读：新机或异常都按空处理，不视为错误
        return result;
    };
    for workspace in workspaces.flatten().map(|e| e.path()) {
        if !workspace.is_dir() {
            continue;
        }
        let Ok(subs) = std::fs::read_dir(&workspace) else {
            continue;
        };
        for session_dir in subs.flatten().map(|e| e.path()) {
            if !session_dir.is_dir() {
                continue;
            }
            let state_path = session_dir.join("state.json");
            if !state_path.is_file() {
                continue;
            }
            if let Some(session) = parse_session(&state_path, &session_dir, &workspace) {
                result.sessions.push(session);
            }
        }
    }
    result
        .sessions
        .sort_by_key(|s| std::cmp::Reverse(s.updated_at_ms));
    result
}

/// 按期限归档默认根目录下的旧会话（严格大于期限才归档；已归档跳过；
/// 单项写失败容忍跳过），返回本次归档个数
pub fn archive_older_than(threshold: ArchiveThreshold, now_ms: i64) -> usize {
    match local_usage::sessions_dir() {
        Some(root) => archive_older_than_in(&root, threshold, now_ms),
        None => 0,
    }
}

pub fn archive_older_than_in(root: &Path, threshold: ArchiveThreshold, now_ms: i64) -> usize {
    let mut count = 0;
    for session in scan_sessions_in(root).sessions {
        if session.is_archived {
            continue;
        }
        // 边界：恰好等于期限不归档（与 macOS 的严格大于一致）
        if now_ms - session.updated_at_ms <= threshold.ms() {
            continue;
        }
        if set_archived(Path::new(&session.path), true) {
            count += 1;
        }
    }
    count
}

/// 设置单个会话的归档标志（恢复 = false）；updatedAt 按原格式刷新为当前时间
pub fn set_archived(session_dir: &Path, archived: bool) -> bool {
    set_archive_state(
        &session_dir.join("state.json"),
        archived,
        chrono::Utc::now().timestamp_millis(),
    )
}

// ---------------------------------------------------------------------------
// 以下为内部实现
// ---------------------------------------------------------------------------

/// 解析单个会话目录：state.json 可读、updatedAt 可解析才采信，否则跳过
fn parse_session(
    state_path: &Path,
    session_dir: &Path,
    workspace: &Path,
) -> Option<ArchiveSession> {
    let text = std::fs::read_to_string(state_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let updated_at_ms = parse_updated_at(json.get("updatedAt")?)?;

    let dir_name = session_dir.file_name()?.to_string_lossy().to_string();
    let title = json
        .get("title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&dir_name)
        .to_string();
    let is_archived = json
        .get("archived")
        .and_then(|v| v.as_bool())
        // Kimi Code CLI 自身归档只写 custom.archived（macOS 版与我们归档时两者都写），
        // 两处任一置位都视为已归档
        .or_else(|| {
            json.get("custom")
                .and_then(|c| c.get("archived"))
                .and_then(|v| v.as_bool())
        })
        .unwrap_or(false);
    let folder_name = resolve_folder_name(&json).unwrap_or_else(|| {
        workspace
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    Some(ArchiveSession {
        id: dir_name,
        workspace_hash: workspace
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        folder_name,
        title,
        updated_at_ms,
        is_archived,
        path: session_dir.to_string_lossy().to_string(),
    })
}

/// 工作区显示名：workDir 非空取其末段，其次 custom.cwd，都没有返回 None
fn resolve_folder_name(json: &serde_json::Value) -> Option<String> {
    let path = json
        .get("workDir")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            json.get("custom")
                .and_then(|c| c.get("cwd"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
        })?;
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

/// updatedAt 解析：ISO8601 字符串（带/不带小数秒）或数字（>1e12 判毫秒，否则秒）
fn parse_updated_at(value: &serde_json::Value) -> Option<i64> {
    if let Some(s) = value.as_str() {
        let dt = chrono::DateTime::parse_from_rfc3339(s).ok()?;
        return Some(dt.timestamp_millis());
    }
    if let Some(n) = value.as_i64() {
        return Some(if n > 1_000_000_000_000 { n } else { n * 1000 });
    }
    None
}

/// 改写 state.json：顶层 archived 与 custom.archived 同步置位，
/// updatedAt 刷新为 now 且保持原格式（ISO 字符串进 → ISO 出；数字进 → 毫秒数字出）
/// ——保持格式是为了不破坏 Kimi Code 对会话文件的读取。任一步失败返回 false
fn set_archive_state(state_path: &Path, archived: bool, now_ms: i64) -> bool {
    let Ok(text) = std::fs::read_to_string(state_path) else {
        return false;
    };
    let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    let Some(obj) = json.as_object_mut() else {
        return false;
    };

    let original_is_string = obj.get("updatedAt").is_some_and(|v| v.is_string());
    obj.insert("archived".to_string(), serde_json::Value::Bool(archived));
    let mut custom = obj
        .get("custom")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    custom.insert("archived".to_string(), serde_json::Value::Bool(archived));
    obj.insert("custom".to_string(), serde_json::Value::Object(custom));
    obj.insert(
        "updatedAt".to_string(),
        if original_is_string {
            serde_json::Value::String(format_iso_ms(now_ms))
        } else {
            serde_json::Value::Number(now_ms.into())
        },
    );

    let Ok(out) = serde_json::to_string_pretty(&json) else {
        return false;
    };
    atomic_write(state_path, &out)
}

/// 毫秒 → ISO8601（带毫秒小数，与 macOS 版写入格式一致）
fn format_iso_ms(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
        .unwrap_or_default()
}

/// 原子写入（临时文件 + rename；先删目标再 rename，与 storage::save_json 同款）
fn atomic_write(path: &Path, contents: &str) -> bool {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = dir.join("state.json.tmp");
    if std::fs::write(&tmp, contents).is_err() {
        return false;
    }
    if path.exists() && std::fs::remove_file(path).is_err() {
        return false;
    }
    std::fs::rename(&tmp, path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// 在临时根目录下构造一个会话目录并写入 state.json，返回 (root, session_dir)
    fn write_session(root: &Path, wd: &str, ses: &str, state_json: &str) -> PathBuf {
        let dir = root.join(wd).join(ses);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.json"), state_json).unwrap();
        dir
    }

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kimicodebar-archive-{tag}-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn state_iso(updated_at: &str) -> String {
        format!(
            r#"{{"title":"测试会话","workDir":"/home/u/erp-web","updatedAt":"{updated_at}","custom":{{"archived":false}}}}"#
        )
    }

    // ---- 期限边界：恰好等于期限不归档，超过 1ms 归档 ----

    #[test]
    fn archive_boundary_exact_threshold_not_archived() {
        let root = temp_root("boundary");
        let now = 1_800_000_000_000i64;
        let exact = chrono::DateTime::from_timestamp_millis(now - ArchiveThreshold::OneDay.ms())
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        let older =
            chrono::DateTime::from_timestamp_millis(now - ArchiveThreshold::OneDay.ms() - 1)
                .unwrap()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();
        let dir_exact = write_session(&root, "wd_a_1", "ses_exact", &state_iso(&exact));
        let dir_older = write_session(&root, "wd_a_1", "ses_older", &state_iso(&older));

        let count = archive_older_than_in(&root, ArchiveThreshold::OneDay, now);
        assert_eq!(count, 1, "只有超过期限的会话应被归档");

        let read = |dir: &Path| {
            let text = std::fs::read_to_string(dir.join("state.json")).unwrap();
            serde_json::from_str::<serde_json::Value>(&text).unwrap()
        };
        // 未触碰的会话顶层 archived 字段不存在（state_iso 模板本就没有该字段），
        // 只能断言"不为 true"，不能断言"等于 false"
        assert!(read(&dir_exact)["archived"] != true, "恰好等于期限不归档");
        assert_eq!(read(&dir_older)["archived"], true);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn archive_skips_already_archived() {
        let root = temp_root("skip-archived");
        let now = 1_800_000_000_000i64;
        let old = chrono::DateTime::from_timestamp_millis(now - ArchiveThreshold::OneWeek.ms() - 1)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        let state = format!(
            r#"{{"title":"旧会话","updatedAt":"{old}","archived":true,"custom":{{"archived":true}}}}"#
        );
        write_session(&root, "wd_a_1", "ses_old", &state);

        let count = archive_older_than_in(&root, ArchiveThreshold::OneDay, now);
        assert_eq!(count, 0, "已归档会话不应重复计入");

        let _ = std::fs::remove_dir_all(&root);
    }

    // ---- 容忍：根目录缺失 / 单项写失败 ----

    #[test]
    fn scan_missing_root_returns_empty_without_error() {
        let result = scan_sessions_in(Path::new("/proc/kimicodebar-no-such-dir"));
        assert!(result.sessions.is_empty());
        assert!(result.error.is_none());
        assert_eq!(
            archive_older_than_in(
                Path::new("/proc/kimicodebar-no-such-dir"),
                ArchiveThreshold::OneDay,
                0
            ),
            0
        );
    }

    #[cfg(unix)]
    #[test]
    fn archive_tolerates_single_unwritable_session() {
        let root = temp_root("unwritable");
        let now = 1_800_000_000_000i64;
        let old = chrono::DateTime::from_timestamp_millis(now - ArchiveThreshold::OneDay.ms() - 1)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        let dir_ro = write_session(&root, "wd_a_1", "ses_ro", &state_iso(&old));
        let dir_ok = write_session(&root, "wd_a_1", "ses_ok", &state_iso(&old));

        // ses_ro 目录只读：临时文件写不进 → 该项失败但不影响 ses_ok
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir_ro, std::fs::Permissions::from_mode(0o555)).unwrap();

        let count = archive_older_than_in(&root, ArchiveThreshold::OneDay, now);
        assert_eq!(count, 1, "单项写失败应容忍跳过，其余照常归档");

        let ok_text = std::fs::read_to_string(dir_ok.join("state.json")).unwrap();
        assert!(ok_text.contains("\"archived\": true"));

        // 恢复权限以便清理
        std::fs::set_permissions(&dir_ro, std::fs::Permissions::from_mode(0o755)).unwrap();
        let _ = std::fs::remove_dir_all(&root);
    }

    // ---- set_archived：格式保持与往返 ----

    #[test]
    fn set_archived_preserves_iso_format_and_roundtrips() {
        let root = temp_root("iso-roundtrip");
        let dir = write_session(
            &root,
            "wd_a_1",
            "ses_iso",
            &state_iso("2026-05-19T03:30:51.748Z"),
        );

        assert!(set_archived(&dir, true));
        let text = std::fs::read_to_string(dir.join("state.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["archived"], true);
        assert_eq!(json["custom"]["archived"], true);
        // ISO 进 → ISO 出：仍是 RFC3339 字符串且时间被刷新
        let refreshed = json["updatedAt"]
            .as_str()
            .expect("updatedAt 应保持字符串格式");
        assert!(chrono::DateTime::parse_from_rfc3339(refreshed).is_ok());
        assert_ne!(refreshed, "2026-05-19T03:30:51.748Z");

        assert!(set_archived(&dir, false));
        let text = std::fs::read_to_string(dir.join("state.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["archived"], false);
        assert_eq!(json["custom"]["archived"], false);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn set_archived_preserves_numeric_format() {
        let root = temp_root("num-format");
        let dir = write_session(
            &root,
            "wd_a_1",
            "ses_num",
            r#"{"title":"数字时间","updatedAt":1747611051}"#,
        );

        assert!(set_archived(&dir, true));
        let text = std::fs::read_to_string(dir.join("state.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        // 数字进 → 毫秒数字出
        let refreshed = json["updatedAt"]
            .as_i64()
            .expect("updatedAt 应保持数字格式");
        assert!(refreshed > 1_000_000_000_000);

        let _ = std::fs::remove_dir_all(&root);
    }

    // ---- 解析：回退与跳过 ----

    #[test]
    fn parse_fallbacks_and_skips() {
        let root = temp_root("parse");
        // 无 title → 目录名；updatedAt 为秒级数字 → 乘 1000
        write_session(
            &root,
            "wd_proj_ab12",
            "ses_notitle",
            r#"{"updatedAt":1747611051}"#,
        );
        // 只有 custom.cwd → folder_name 取其末段
        write_session(
            &root,
            "wd_x_cd34",
            "ses_cwd",
            r#"{"title":"t","updatedAt":"2026-07-01T00:00:00Z","custom":{"cwd":"/a/b/myproj"}}"#,
        );
        // updatedAt 无法解析 → 整个会话跳过
        write_session(
            &root,
            "wd_x_cd34",
            "ses_bad",
            r#"{"title":"t","updatedAt":"garbage"}"#,
        );
        // 无 state.json → 跳过
        std::fs::create_dir_all(root.join("wd_x_cd34").join("ses_nostate")).unwrap();

        let result = scan_sessions_in(&root);
        assert_eq!(result.sessions.len(), 2, "bad/nostate 应被跳过");

        let notitle = result
            .sessions
            .iter()
            .find(|s| s.id == "ses_notitle")
            .unwrap();
        assert_eq!(notitle.title, "ses_notitle");
        assert_eq!(notitle.updated_at_ms, 1747611051 * 1000);
        // 无 workDir/custom.cwd → folder_name 回退工作区目录名
        assert_eq!(notitle.folder_name, "wd_proj_ab12");

        let cwd = result.sessions.iter().find(|s| s.id == "ses_cwd").unwrap();
        assert_eq!(cwd.folder_name, "myproj");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_sorts_updated_at_desc() {
        let root = temp_root("sort");
        write_session(
            &root,
            "wd_a_1",
            "ses_old",
            &state_iso("2026-07-01T00:00:00Z"),
        );
        write_session(
            &root,
            "wd_a_1",
            "ses_new",
            &state_iso("2026-07-20T00:00:00Z"),
        );

        let result = scan_sessions_in(&root);
        assert_eq!(result.sessions[0].id, "ses_new");
        assert_eq!(result.sessions[1].id, "ses_old");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn parse_reads_custom_archived_flag() {
        // Kimi Code CLI 自身归档只写 custom.archived（顶层 archived 缺省）
        let root = temp_root("custom-archived");
        write_session(
            &root,
            "wd_a_1",
            "ses_cli",
            r#"{"title":"t","updatedAt":"2026-07-01T00:00:00Z","custom":{"archived":true}}"#,
        );

        let result = scan_sessions_in(&root);
        assert_eq!(result.sessions.len(), 1);
        assert!(
            result.sessions[0].is_archived,
            "custom.archived=true 应被识别为已归档"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    // ---- 期限枚举契约 ----

    #[test]
    fn threshold_str_roundtrip_and_reject_garbage() {
        for t in [
            ArchiveThreshold::OneDay,
            ArchiveThreshold::OneWeek,
            ArchiveThreshold::OneMonth,
        ] {
            assert_eq!(ArchiveThreshold::parse(t.as_str()), Some(t));
        }
        assert_eq!(ArchiveThreshold::parse("oneYear"), None);
        assert_eq!(ArchiveThreshold::parse(""), None);
    }
}
