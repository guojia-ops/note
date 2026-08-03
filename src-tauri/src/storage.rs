// DeskNote 存储层
// PRD 4: JSON 单文件 + Rust 内存 SOT + 防抖落盘
// PRD 4.1: 先 Windows 预留跨平台
// PRD 4.2: D 盘优先 D:\ProgramData\notes_data，回退 Tauri appDataDir

use crate::error::{Error, Result};
use crate::types::{Config, DataFile};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// D 盘首选路径（PRD 4.2）
pub(crate) const PREFERRED_DIR: &str = r"D:\ProgramData\notes_data";

/// 数据文件名
const DATA_FILENAME: &str = "data.json";
/// 配置文件名
const CONFIG_FILENAME: &str = "config.json";
/// 备份子目录名
const BACKUP_DIR: &str = "backups";

/// 解析数据目录路径（PRD 4.2 决策）
/// 优先 D:\ProgramData\notes_data，D 盘不存在时回退 Tauri appDataDir
pub fn resolve_data_dir(app_data_dir: Option<&Path>) -> Result<PathBuf> {
    // 检测 D 盘是否存在（通过 D:\ 根目录是否存在）
    let d_drive = Path::new(r"D:\");
    if d_drive.exists() {
        Ok(PathBuf::from(PREFERRED_DIR))
    } else {
        // 回退到 Tauri appDataDir
        app_data_dir
            .ok_or_else(|| Error::Path("Tauri appDataDir 不可用且 D 盘不存在".into()))
            .map(|p| p.to_path_buf())
    }
}

/// 确保目录存在（递归创建）
fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 数据目录路径访问器
/// 缓存解析结果，避免每次都检测 D 盘
pub struct Storage {
    pub(crate) data_dir: PathBuf,
}

/// 落盘重试参数（SOP 10.1）
const SAVE_MAX_RETRIES: usize = 3;
const SAVE_RETRY_INTERVAL_MS: u64 = 100;

impl Storage {
    /// 创建 Storage 实例，确保目录结构存在
    pub fn new(app_data_dir: Option<&Path>) -> Result<Self> {
        let data_dir = resolve_data_dir(app_data_dir)?;
        ensure_dir(&data_dir)?;

        // 确保备份子目录存在
        let backup_dir = data_dir.join(BACKUP_DIR);
        ensure_dir(&backup_dir)?;

        Ok(Self { data_dir })
    }

    /// 数据目录路径
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// data.json 路径
    pub fn data_path(&self) -> PathBuf {
        self.data_dir.join(DATA_FILENAME)
    }

    /// config.json 路径
    pub fn config_path(&self) -> PathBuf {
        self.data_dir.join(CONFIG_FILENAME)
    }

    /// backups/ 路径
    pub fn backup_dir(&self) -> PathBuf {
        self.data_dir.join(BACKUP_DIR)
    }

    /// 读取所有便签数据
    /// 文件不存在时返回空 DataFile（首次启动）
    /// SOP 10.2：JSON 损坏时尝试从 backups/ 最近一份恢复
    pub fn load_data(&self) -> Result<DataFile> {
        let path = self.data_path();
        if !path.exists() {
            return Ok(DataFile::default());
        }
        let content = fs::read_to_string(&path).map_err(|e| {
            eprintln!("[DeskNote] data.json 读取失败，尝试从备份恢复: {}", e);
            Error::Io(e)
        })?;
        if content.trim().is_empty() {
            return Ok(DataFile::default());
        }
        match serde_json::from_str::<DataFile>(&content) {
            Ok(data) => Ok(data),
            Err(e) => {
                // SOP 10.2：JSON 损坏，尝试从备份恢复
                eprintln!("[DeskNote] data.json 解析失败，尝试从备份恢复: {}", e);
                match self.restore_from_backup() {
                    Some(data) => {
                        println!("[DeskNote] 已从备份恢复 {} 条便签", data.notes.len());
                        Ok(data)
                    }
                    None => Err(Error::Path(format!("data.json 损坏且无可用备份: {}", e))),
                }
            }
        }
    }

    /// 读取配置
    /// 文件不存在时返回默认配置
    pub fn load_config(&self) -> Result<Config> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(&path)?;
        if content.trim().is_empty() {
            return Ok(Config::default());
        }
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 原子写入文件（先写 .tmp 再 rename，避免崩溃导致数据损坏）
    /// PRD 4: 崩溃恢复
    fn write_atomic(&self, path: &Path, content: &str) -> Result<()> {
        let tmp_path = path.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(content.as_bytes())?;
            f.sync_all()?; // 确保落盘
        }
        // rename 是原子操作（同一文件系统内）
        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// SOP 10.1：带重试的原子写入
    /// 失败重试 SAVE_MAX_RETRIES 次，间隔 SAVE_RETRY_INTERVAL_MS
    fn write_atomic_with_retry(&self, path: &Path, content: &str) -> Result<()> {
        let mut last_err: Option<Error> = None;
        for attempt in 1..=SAVE_MAX_RETRIES {
            match self.write_atomic(path, content) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    eprintln!(
                        "[DeskNote] 落盘失败 (第 {}/{} 次): {}",
                        attempt, SAVE_MAX_RETRIES, e
                    );
                    last_err = Some(e);
                    if attempt < SAVE_MAX_RETRIES {
                        std::thread::sleep(std::time::Duration::from_millis(
                            SAVE_RETRY_INTERVAL_MS,
                        ));
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| Error::Path("落盘重试耗尽".into())))
    }

    /// 保存所有便签数据（带重试，SOP 10.1）
    pub fn save_data(&self, data: &DataFile) -> Result<()> {
        let content = serde_json::to_string_pretty(data)?;
        self.write_atomic_with_retry(&self.data_path(), &content)
    }

    /// 保存配置（带重试，SOP 10.1）
    pub fn save_config(&self, config: &Config) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        self.write_atomic_with_retry(&self.config_path(), &content)
    }

    /// SOP 10.2：从 backups/ 最近一份备份恢复数据
    /// 返回 Some(DataFile) 表示恢复成功，None 表示无可用备份
    fn restore_from_backup(&self) -> Option<DataFile> {
        let backup_dir = self.backup_dir();
        if !backup_dir.exists() {
            return None;
        }
        // 列出所有备份，按修改时间倒序（最新在前）
        let mut backups: Vec<_> = fs::read_dir(&backup_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.starts_with("data-") && s.ends_with(".json"))
                    .unwrap_or(false)
            })
            .collect();
        backups.sort_by(|a, b| {
            b.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });

        for entry in backups {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(data) = serde_json::from_str::<DataFile>(&content) {
                    return Some(data);
                }
            }
        }
        None
    }

    /// 创建备份（PRD 设置项 6）
    /// 复制当前 data.json 到 backups/data-<timestamp>.json
    /// 保留最近 N 份，超出删除最旧的
    pub fn create_backup(&self, keep: u32) -> Result<()> {
        let data_path = self.data_path();
        if !data_path.exists() {
            return Ok(()); // 无数据可备份
        }

        let backup_dir = self.backup_dir();
        ensure_dir(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S%.3f");
        let backup_name = format!("data-{}.json", timestamp);
        let backup_path = backup_dir.join(backup_name);
        fs::copy(data_path, &backup_path)?;

        // 清理超出 keep 数量的旧备份
        self.cleanup_old_backups(keep)?;
        Ok(())
    }

    /// 清理旧备份，保留最近 keep 份
    fn cleanup_old_backups(&self, keep: u32) -> Result<()> {
        let backup_dir = self.backup_dir();
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut backups: Vec<_> = fs::read_dir(&backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.starts_with("data-") && s.ends_with(".json"))
                    .unwrap_or(false)
            })
            .collect();

        // 按修改时间倒序（最新在前）
        backups.sort_by(|a, b| {
            b.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });

        // 删除超出 keep 的
        for entry in backups.into_iter().skip(keep as usize) {
            let _ = fs::remove_file(entry.path()); // 单个删除失败不影响整体
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Note;
    use tempfile::TempDir;

    /// 用临时目录测试 Storage（避免污染真实数据目录）
    fn make_test_storage() -> (TempDir, Storage) {
        let tmp = TempDir::new().unwrap();
        // 直接构造 Storage（绕过 D 盘检测，用 tmp 路径）
        let data_dir = tmp.path().to_path_buf();
        ensure_dir(&data_dir).unwrap();
        ensure_dir(&data_dir.join(BACKUP_DIR)).unwrap();
        let storage = Storage { data_dir };
        (tmp, storage)
    }

    #[test]
    fn load_data_missing_file_returns_default() {
        let (_tmp, storage) = make_test_storage();
        let data = storage.load_data().unwrap();
        assert!(data.notes.is_empty());
    }

    #[test]
    fn load_config_missing_file_returns_default() {
        let (_tmp, storage) = make_test_storage();
        let config = storage.load_config().unwrap();
        assert_eq!(config.hotkey, "Ctrl+Alt+N");
    }

    #[test]
    fn save_then_load_data_roundtrip() {
        let (_tmp, storage) = make_test_storage();
        let mut data = DataFile::default();
        let note = Note::new();
        let note_id = note.id.clone();
        data.notes.push(note);

        storage.save_data(&data).unwrap();
        let loaded = storage.load_data().unwrap();

        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.notes[0].id, note_id);
    }

    #[test]
    fn save_then_load_config_roundtrip() {
        let (_tmp, storage) = make_test_storage();
        let mut config = Config::default();
        config.hotkey = "Ctrl+Shift+N".to_string();
        config.theme = crate::types::Theme::Dark;

        storage.save_config(&config).unwrap();
        let loaded = storage.load_config().unwrap();

        assert_eq!(loaded.hotkey, "Ctrl+Shift+N");
        assert_eq!(loaded.theme, crate::types::Theme::Dark);
    }

    #[test]
    fn atomic_write_does_not_leave_tmp() {
        let (_tmp, storage) = make_test_storage();
        storage.save_data(&DataFile::default()).unwrap();
        let tmp_path = storage.data_path().with_extension("tmp");
        assert!(!tmp_path.exists());
    }

    #[test]
    fn backup_creates_file_and_cleans_old() {
        let (_tmp, storage) = make_test_storage();
        // 先写数据
        storage.save_data(&DataFile::default()).unwrap();

        // 创建 3 份备份，keep=2
        std::thread::sleep(std::time::Duration::from_millis(10));
        storage.create_backup(2).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        storage.create_backup(2).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        storage.create_backup(2).unwrap();

        let backup_count = fs::read_dir(storage.backup_dir())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .ok()
                    .and_then(|e| e.file_name().to_str().map(|s| s.starts_with("data-")))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(backup_count, 2, "应保留最近 2 份备份");
    }

    #[test]
    fn resolve_data_dir_prefers_d_drive_if_exists() {
        let d_drive = Path::new(r"D:\");
        if d_drive.exists() {
            let resolved = resolve_data_dir(None).unwrap();
            assert_eq!(resolved, PathBuf::from(PREFERRED_DIR));
        }
        // 无 D 盘的环境跳过此测试
    }

    #[test]
    fn resolve_data_dir_falls_back_to_appdata() {
        let tmp = TempDir::new().unwrap();
        let resolved = resolve_data_dir(Some(tmp.path())).unwrap();
        // 仅当 D 盘不存在时才会用 tmp 路径
        let d_drive = Path::new(r"D:\");
        if !d_drive.exists() {
            assert_eq!(resolved, tmp.path());
        }
    }
}
