//! 凭证管理：API Key / 网页 token 走安全存储，OAuth 走 kimi::oauth。
//!
//! `get_active_token` 是数据链路取 token 的唯一入口：按设置里的登录方式选择，
//! 未显式选择时优先 API Key、其次 OAuth；OAuth token 临期自动刷新。
//!
//! 安全存储（PRD 0001 唯一新测试接缝）：优先 Secret Service
//! （Linux 为 gnome-keyring 等 D-Bus 实现，Windows 为凭据管理器）；
//! 探测不可用时降级为配置目录下 0600 权限的 secrets.json，
//! 并通过 `active_backend` 暴露给设置页提示用户（见 CredentialStatus）。

use std::path::PathBuf;

use keyring::Entry;
use thiserror::Error;

use crate::kimi::oauth::{self, OAuthError};
use crate::storage;

/// keyring 服务名（Secret Service / 凭据管理器里的"服务/目标"）
const KEYRING_SERVICE: &str = "KimiCodeBar";
/// 条目名：API Key
const KEY_API: &str = "api_key";
/// 条目名：网页端 kimi-auth token（月度总量用）
const KEY_WEB: &str = "web_token";
/// 降级文件名（配置目录下）
const SECRETS_FILE: &str = "secrets.json";
/// OAuth token 剩余有效期小于该值（秒）即提前刷新，与 Mac 版 5 分钟一致
const REFRESH_MARGIN_SECS: i64 = 300;

#[derive(Debug, Error)]
pub enum CredError {
    #[error("安全存储错误: {0}")]
    Store(String),
    #[error("OAuth 错误: {0}")]
    OAuth(#[from] OAuthError),
    #[error("本地存储错误: {0}")]
    Storage(String),
}

/// 当前生效的凭证类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialKind {
    ApiKey,
    OAuth,
}

/// 安全存储后端种类（暴露给设置页展示降级提示；
/// as_str 取值与 src/types.ts 的 CredentialStatus.storage_backend 契约一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// Secret Service（gnome-keyring / KWallet / Windows 凭据管理器）
    SecretService,
    /// 降级：配置目录下 0600 权限文件
    File,
}

impl BackendKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendKind::SecretService => "secret_service",
            BackendKind::File => "file",
        }
    }
}

/// 保存 API Key 到安全存储
pub fn save_api_key(key: &str) -> Result<(), CredError> {
    save_secret(KEY_API, key, &*store())
}

/// 读取 API Key：未保存过 → Ok(None)
pub fn load_api_key() -> Result<Option<String>, CredError> {
    load_secret(KEY_API, &*store())
}

/// 删除 API Key：本来就不存在也算成功
pub fn clear_api_key() -> Result<(), CredError> {
    delete_secret(KEY_API, &*store())
}

/// 保存网页端 kimi-auth token（月度总量用）到安全存储
pub fn save_web_token(token: &str) -> Result<(), CredError> {
    save_secret(KEY_WEB, token, &*store())
}

/// 读取网页端 token：未保存过 → Ok(None)
pub fn load_web_token() -> Result<Option<String>, CredError> {
    load_secret(KEY_WEB, &*store())
}

/// 删除网页端 token：本来就不存在也算成功
pub fn clear_web_token() -> Result<(), CredError> {
    delete_secret(KEY_WEB, &*store())
}

/// 取当前生效的 token：
/// - `settings.login_method == "api_key"` → 只查安全存储
/// - `settings.login_method == "oauth"` → 只查 OAuth 凭证
/// - 未显式选择 → 优先 API Key，其次 OAuth
///
/// OAuth token 临期（<300s）时自动刷新并回写；刷新返回 NotAuthorized（授权已吊销）
/// 时清除本地凭证并返回 Ok(None)，由上层 UI 提示重新登录。
pub async fn get_active_token() -> Result<Option<(CredentialKind, String)>, CredError> {
    let settings = storage::load_settings().map_err(CredError::Storage)?;
    match settings.login_method.as_deref() {
        Some("api_key") => Ok(load_api_key()?.map(|key| (CredentialKind::ApiKey, key))),
        Some("oauth") => oauth_token().await,
        // 未显式选择（或值非法）：优先 api_key，其次 oauth
        _ => {
            if let Some(key) = load_api_key()? {
                return Ok(Some((CredentialKind::ApiKey, key)));
            }
            oauth_token().await
        }
    }
}

// ---------------------------------------------------------------------------
// 安全存储抽象与后端选择（测试接缝）
// ---------------------------------------------------------------------------

/// 安全存储抽象：get 未保存过返回 Ok(None)；delete 不存在也视为成功
trait SecretStore {
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    fn set(&self, key: &str, value: &str) -> Result<(), String>;
    fn delete(&self, key: &str) -> Result<(), String>;
}

/// 进程级后端选择：探测一次后缓存——Secret Service 在会话内出现/消失都罕见，
/// 且同一时刻双写分裂比短暂不可用的危害更大
static BACKEND: std::sync::OnceLock<BackendKind> = std::sync::OnceLock::new();

/// 当前生效的安全存储后端（设置页降级提示用）
pub fn active_backend() -> BackendKind {
    *BACKEND.get_or_init(|| select_backend(probe_secret_service()))
}

/// 选择逻辑（纯函数）：可用 → SecretService，否则降级 File
fn select_backend(secret_service_available: bool) -> BackendKind {
    if secret_service_available {
        BackendKind::SecretService
    } else {
        BackendKind::File
    }
}

/// 探测 Secret Service 可用性：Windows 凭据管理器恒可用；
/// Linux 下对探针条目做一次读——NoEntry 说明服务在线（只是没存过），
/// 其余错误（无 D-Bus 会话、无 keyring 守护等）视为不可用
fn probe_secret_service() -> bool {
    #[cfg(windows)]
    {
        true
    }
    #[cfg(not(windows))]
    {
        match Entry::new(KEYRING_SERVICE, "availability_probe") {
            Ok(entry) => matches!(entry.get_password(), Ok(_) | Err(keyring::Error::NoEntry)),
            Err(_) => false,
        }
    }
}

/// 按当前后端构造存储实例
fn store() -> Box<dyn SecretStore> {
    match active_backend() {
        BackendKind::SecretService => Box::new(KeyringStore),
        BackendKind::File => Box::new(FileStore::new(storage::config_dir().join(SECRETS_FILE))),
    }
}

fn save_secret(key: &str, value: &str, store: &dyn SecretStore) -> Result<(), CredError> {
    store.set(key, value).map_err(CredError::Store)
}

fn load_secret(key: &str, store: &dyn SecretStore) -> Result<Option<String>, CredError> {
    store.get(key).map_err(CredError::Store)
}

fn delete_secret(key: &str, store: &dyn SecretStore) -> Result<(), CredError> {
    store.delete(key).map_err(CredError::Store)
}

/// Secret Service / 凭据管理器后端
struct KeyringStore;

impl KeyringStore {
    fn entry(&self, key: &str) -> Result<Entry, String> {
        Entry::new(KEYRING_SERVICE, key).map_err(|e| e.to_string())
    }
}

impl SecretStore for KeyringStore {
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        match self.entry(key)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        self.entry(key)?
            .set_password(value)
            .map_err(|e| e.to_string())
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        match self.entry(key)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// 0600 文件降级后端：整个存储为单个 JSON 对象 {"key": "value"}，
/// 原子写入（临时文件 + rename），Unix 下强制 0600 权限
struct FileStore {
    path: PathBuf,
}

impl FileStore {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn read_all(&self) -> Result<serde_json::Map<String, serde_json::Value>, String> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => {
                match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text) {
                    Ok(map) => Ok(map),
                    // 文件损坏不致命：按空存储处理（读出 None），写时会整体覆盖
                    Err(_) => Ok(serde_json::Map::new()),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::Map::new()),
            Err(e) => Err(format!("读取降级存储失败: {e}")),
        }
    }

    /// 原子写入并强制 0600：权限设在临时文件上，rename 后目标即 0600
    fn write_all(&self, map: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
        let dir = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        std::fs::create_dir_all(dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
        let tmp = dir.join("secrets.json.tmp");
        let json = serde_json::to_string_pretty(map).map_err(|e| e.to_string())?;
        {
            use std::io::Write;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }
            let mut file = opts
                .open(&tmp)
                .map_err(|e| format!("写入降级存储失败: {e}"))?;
            file.write_all(json.as_bytes())
                .map_err(|e| format!("写入降级存储失败: {e}"))?;
        }
        // 已存在的临时文件可能权限偏宽：显式收紧到 0600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("收紧降级存储权限失败: {e}"))?;
        }
        std::fs::rename(&tmp, &self.path).map_err(|e| format!("保存降级存储失败: {e}"))
    }
}

impl SecretStore for FileStore {
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self
            .read_all()?
            .get(key)
            .and_then(|v| v.as_str().map(str::to_string)))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let mut map = self.read_all()?;
        map.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        self.write_all(&map)
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        let mut map = self.read_all()?;
        if map.remove(key).is_some() {
            self.write_all(&map)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 以下为内部实现
// ---------------------------------------------------------------------------

/// OAuth 路径：读本地凭证 → 临期刷新 → 返回 access_token
async fn oauth_token() -> Result<Option<(CredentialKind, String)>, CredError> {
    let Some(creds) = oauth::load_credentials()? else {
        return Ok(None);
    };

    let creds = if oauth::is_expiring_soon(&creds, REFRESH_MARGIN_SECS) {
        match oauth::refresh_token(&creds).await {
            Ok(new_creds) => {
                tracing::info!("OAuth token 刷新成功");
                oauth::save_credentials(&new_creds)?;
                new_creds
            }
            Err(OAuthError::NotAuthorized) => {
                // 授权已被吊销：清掉本地凭证，让上层提示重新登录
                tracing::warn!("OAuth 授权已被吊销，已清除本地凭证");
                oauth::clear_credentials()?;
                return Ok(None);
            }
            // 其余刷新失败（网络抖动等）：token 未必真的失效，先继续用旧的，
            // 真失效时 usages 接口会返回 401，由上层按"凭证无效"提示
            Err(e) => {
                tracing::warn!("OAuth token 刷新失败，暂用旧 token: {e}");
                creds
            }
        }
    } else {
        creds
    };

    Ok(Some((CredentialKind::OAuth, creds.access_token)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- 后端选择逻辑 ----

    #[test]
    fn select_backend_prefers_secret_service_when_available() {
        assert_eq!(select_backend(true), BackendKind::SecretService);
    }

    #[test]
    fn select_backend_falls_back_to_file_when_unavailable() {
        assert_eq!(select_backend(false), BackendKind::File);
    }

    #[test]
    fn backend_kind_contract_strings() {
        assert_eq!(BackendKind::SecretService.as_str(), "secret_service");
        assert_eq!(BackendKind::File.as_str(), "file");
    }

    // ---- 内存实现：覆盖「可用 / 读写失败」路径 ----

    /// 内存 SecretStore：fail 打开后所有读写都报错，模拟 Secret Service 掉线
    struct MemStore {
        map: Mutex<HashMap<String, String>>,
        fail: bool,
    }

    impl MemStore {
        fn new() -> Self {
            Self {
                map: Mutex::new(HashMap::new()),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                map: Mutex::new(HashMap::new()),
                fail: true,
            }
        }
    }

    impl SecretStore for MemStore {
        fn get(&self, key: &str) -> Result<Option<String>, String> {
            if self.fail {
                return Err("模拟存储掉线".to_string());
            }
            Ok(self.map.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &str) -> Result<(), String> {
            if self.fail {
                return Err("模拟存储掉线".to_string());
            }
            self.map
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), String> {
            if self.fail {
                return Err("模拟存储掉线".to_string());
            }
            self.map.lock().unwrap().remove(key);
            Ok(())
        }
    }

    #[test]
    fn mem_store_roundtrip_via_public_helpers() {
        let store = MemStore::new();
        save_secret(KEY_API, "sk-kimi-test", &store).unwrap();
        assert_eq!(
            load_secret(KEY_API, &store).unwrap().as_deref(),
            Some("sk-kimi-test")
        );
        // 不同 key 互不影响
        assert_eq!(load_secret(KEY_WEB, &store).unwrap(), None);
        delete_secret(KEY_API, &store).unwrap();
        assert_eq!(load_secret(KEY_API, &store).unwrap(), None);
        // 删除不存在的条目也算成功
        delete_secret(KEY_API, &store).unwrap();
    }

    #[test]
    fn failing_store_surfaces_errors_not_panics() {
        let store = MemStore::failing();
        assert!(matches!(
            save_secret(KEY_API, "k", &store),
            Err(CredError::Store(_))
        ));
        assert!(matches!(
            load_secret(KEY_API, &store),
            Err(CredError::Store(_))
        ));
        assert!(matches!(
            delete_secret(KEY_API, &store),
            Err(CredError::Store(_))
        ));
    }

    // ---- FileStore：降级后端的实际行为 ----

    use crate::TEST_ENV_LOCK as ENV_LOCK;

    /// 指向独立临时配置目录，返回 (目录, FileStore)
    fn temp_file_store() -> (PathBuf, FileStore) {
        let dir =
            std::env::temp_dir().join(format!("kimicodebar-creds-test-{}", uuid::Uuid::new_v4()));
        std::env::set_var("KIMICODEBAR_CONFIG_DIR", &dir);
        let store = FileStore::new(dir.join(SECRETS_FILE));
        (dir, store)
    }

    fn cleanup(dir: &PathBuf) {
        std::env::remove_var("KIMICODEBAR_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn file_store_roundtrip_and_delete() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (dir, store) = temp_file_store();

        assert_eq!(store.get(KEY_API).unwrap(), None);
        store.set(KEY_API, "sk-kimi-file").unwrap();
        store.set(KEY_WEB, "web-cookie").unwrap();
        assert_eq!(store.get(KEY_API).unwrap().as_deref(), Some("sk-kimi-file"));
        assert_eq!(store.get(KEY_WEB).unwrap().as_deref(), Some("web-cookie"));

        store.delete(KEY_API).unwrap();
        assert_eq!(store.get(KEY_API).unwrap(), None);
        assert_eq!(store.get(KEY_WEB).unwrap().as_deref(), Some("web-cookie"));

        cleanup(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn file_store_secrets_file_is_0600() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (dir, store) = temp_file_store();

        store.set(KEY_API, "sk-kimi-file").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dir.join(SECRETS_FILE))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "降级存储文件必须是 0600 权限");

        cleanup(&dir);
    }

    #[test]
    fn file_store_corrupt_file_reads_as_empty() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (dir, store) = temp_file_store();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(SECRETS_FILE), "not json").unwrap();

        assert_eq!(store.get(KEY_API).unwrap(), None);
        // 写入时整体覆盖损坏内容
        store.set(KEY_API, "sk-kimi-new").unwrap();
        assert_eq!(store.get(KEY_API).unwrap().as_deref(), Some("sk-kimi-new"));

        cleanup(&dir);
    }

    #[test]
    fn file_store_unwritable_dir_surfaces_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        // 指向不可能存在的深层路径且无创建权限（/proc 下不可写）
        let store = FileStore::new(PathBuf::from("/proc/kimicodebar-nope/secrets.json"));
        assert!(store.set(KEY_API, "k").is_err());

        std::env::remove_var("KIMICODEBAR_CONFIG_DIR");
    }

    /// 给架构方手工种 Key 用：
    /// `KIMI_API_KEY=sk-xxx cargo test --offline -- --ignored seed_api_key --nocapture`
    /// 写入当前生效的安全存储后端，断言写后能读回。
    #[test]
    #[ignore]
    fn seed_api_key() {
        let key = std::env::var("KIMI_API_KEY").expect("请先设置 KIMI_API_KEY 环境变量");
        super::save_api_key(&key).expect("写入安全存储失败");
        let loaded = super::load_api_key().expect("读取安全存储失败");
        assert_eq!(loaded.as_deref(), Some(key.as_str()));
        println!(
            "API Key 已写入安全存储（后端：{:?}）",
            super::active_backend()
        );
    }
}
