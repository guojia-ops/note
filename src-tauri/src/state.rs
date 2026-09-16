// DeskNote 应用状态
// PRD 4: Rust 内存状态作为 SOT，所有窗口通过 commands 读写
// PRD 12.6: 写操作完成后 emit 事件通知所有窗口

use crate::error::{Error, Result};
use crate::storage::Storage;
use crate::types::{Config, DataFile, Note, NoteColor};
use std::sync::Mutex;

/// 更新便签的可选字段（部分更新）
/// 所有字段为 Option，None 表示不更新
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateNoteFields {
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<NoteColor>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    /// v1.1 优化阶段 2.3：所属显示器标识（多屏记忆）
    pub monitor: Option<String>,
}

/// 导入结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

/// 应用全局状态
/// data 和 config 用 Mutex 保护（多窗口并发写）
/// storage 是只读引用（内部无状态）
pub struct AppState {
    data: Mutex<DataFile>,
    config: Mutex<Config>,
    storage: std::sync::Arc<Storage>,
}

impl AppState {
    /// 从 Storage 加载数据构造 State
    pub fn from_storage(storage: Storage) -> Result<Self> {
        let data = storage.load_data()?;
        let config = storage.load_config()?;
        Ok(Self {
            data: Mutex::new(data),
            config: Mutex::new(config),
            storage: std::sync::Arc::new(storage),
        })
    }

    /// Storage 引用（用于 export/import 等直接文件操作）
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// 获取所有便签的快照
    pub fn list_notes(&self) -> Vec<Note> {
        self.data.lock().unwrap().notes.clone()
    }

    /// 创建便签
    /// 返回新便签（调用方负责 emit 事件）
    /// SOP 10.4：落盘成功后若 auto_backup=true 调 create_backup
    pub fn create_note(&self, title: String, content: String, color: NoteColor) -> Result<Note> {
        let mut note = Note::new();
        note.title = title;
        note.content = content;
        note.color = color;

        let mut data = self.data.lock().unwrap();
        data.notes.push(note.clone());
        self.save_data_with_backup(&data)?;
        Ok(note)
    }

    /// 更新便签（部分字段）
    /// 返回更新后的便签
    pub fn update_note(&self, id: &str, fields: &UpdateNoteFields) -> Result<Note> {
        let mut data = self.data.lock().unwrap();
        let note = data
            .notes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;

        if let Some(ref t) = fields.title {
            note.title = t.clone();
        }
        if let Some(ref c) = fields.content {
            note.content = c.clone();
        }
        if let Some(color) = fields.color {
            note.color = color;
        }
        if let Some(w) = fields.width {
            note.width = w;
        }
        if let Some(h) = fields.height {
            note.height = h;
        }
        if let Some(x) = fields.x {
            note.x = x;
        }
        if let Some(y) = fields.y {
            note.y = y;
        }
        if let Some(ref m) = fields.monitor {
            note.monitor = m.clone();
        }
        note.touch();

        let updated = note.clone();
        self.save_data_with_backup(&data)?;
        Ok(updated)
    }

    /// 删除便签（硬删除，PRD 9.4）
    pub fn delete_note(&self, id: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        let before = data.notes.len();
        data.notes.retain(|n| n.id != id);
        if data.notes.len() == before {
            return Err(Error::NotFound(id.to_string()));
        }
        self.save_data_with_backup(&data)?;
        Ok(())
    }

    /// 设置便签 pinned 状态
    pub fn set_pinned(&self, id: &str, pinned: bool) -> Result<Note> {
        let mut data = self.data.lock().unwrap();
        let note = data
            .notes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;
        note.pinned = pinned;
        note.touch();
        let updated = note.clone();
        self.save_data_with_backup(&data)?;
        Ok(updated)
    }

    /// 标记便签为已完成（completed_at = now），返回更新后的便签
    /// 若已是完成态则重复调用是 no-op，返回当前便签
    pub fn complete(&self, id: &str) -> Result<Note> {
        let mut data = self.data.lock().unwrap();
        let note = data
            .notes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;
        if note.completed_at.is_none() {
            note.completed_at = Some(chrono::Utc::now().timestamp_millis());
            note.touch();
        }
        let updated = note.clone();
        self.save_data_with_backup(&data)?;
        Ok(updated)
    }

    /// 撤销完成（completed_at = None），返回更新后的便签
    /// 若本就是未完成态则 no-op
    pub fn uncomplete(&self, id: &str) -> Result<Note> {
        let mut data = self.data.lock().unwrap();
        let note = data
            .notes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;
        if note.completed_at.is_some() {
            note.completed_at = None;
            note.touch();
        }
        let updated = note.clone();
        self.save_data_with_backup(&data)?;
        Ok(updated)
    }

    /// 获取配置
    pub fn get_config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    /// 更新配置（部分字段）
    pub fn update_config(&self, partial: serde_json::Value) -> Result<Config> {
        let mut config = self.config.lock().unwrap();
        // 用 json 反序列化合并：把当前配置序列化为 Value，用 partial 覆盖顶层字段，再反序列化回 Config
        let mut current = serde_json::to_value(&*config)
            .map_err(|e| Error::Path(format!("配置序列化失败: {}", e)))?;
        if let serde_json::Value::Object(ref mut cur_map) = current {
            if let serde_json::Value::Object(updates) = partial {
                for (k, v) in updates {
                    cur_map.insert(k, v);
                }
            }
        }
        *config = serde_json::from_value(current)
            .map_err(|e| Error::Path(format!("配置反序列化失败: {}", e)))?;
        let updated = config.clone();
        self.storage.save_config(&config)?;
        Ok(updated)
    }

    /// 导出所有便签到指定路径
    pub fn export_notes(&self, path: &std::path::Path) -> Result<()> {
        let data = self.data.lock().unwrap();
        let content = serde_json::to_string_pretty(&*data)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 从文件导入便签
    /// 已存在的 id 跳过，返回 (导入数, 跳过数)
    pub fn import_notes(&self, path: &std::path::Path) -> Result<ImportResult> {
        let content = std::fs::read_to_string(path)?;
        let imported_data: DataFile = serde_json::from_str(&content)?;

        let mut data = self.data.lock().unwrap();
        let existing_ids: std::collections::HashSet<_> =
            data.notes.iter().map(|n| n.id.clone()).collect();

        let mut imported = 0;
        let mut skipped = 0;
        for note in imported_data.notes {
            if existing_ids.contains(&note.id) {
                skipped += 1;
            } else {
                data.notes.push(note);
                imported += 1;
            }
        }

        self.save_data_with_backup(&data)?;
        Ok(ImportResult { imported, skipped })
    }

    /// SOP 10.4：落盘 + 自动备份
    /// 落盘成功后，若 config.auto_backup=true，调 create_backup(backup_keep)
    /// 备份失败只记日志，不影响主流程（数据已落盘）
    /// 落盘失败返回 Err，供 commands 层 emit storage:error
    fn save_data_with_backup(&self, data: &DataFile) -> Result<()> {
        self.storage.save_data(data)?;
        let config = self.config.lock().unwrap();
        if config.auto_backup {
            if let Err(e) = self.storage.create_backup(config.backup_keep) {
                eprintln!("[DeskNote] 自动备份失败: {}", e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{resolve_data_dir, PREFERRED_DIR};
    use crate::types::Note;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn make_state() -> (TempDir, AppState) {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().to_path_buf();
        std::fs::create_dir_all(data_dir.join("backups")).unwrap();
        let storage = Storage { data_dir };
        let state = AppState::from_storage(storage).unwrap();
        (tmp, state)
    }

    /// 构造测试用 Storage（不经过 D 盘检测）
    fn make_test_storage_pair() -> (TempDir, Storage) {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().to_path_buf();
        std::fs::create_dir_all(data_dir.join("backups")).unwrap();
        let storage = Storage { data_dir };
        (tmp, storage)
    }

    #[test]
    fn create_note_persists_and_returns() {
        let (_tmp, state) = make_state();
        let note = state.create_note("T1".into(), "C1".into(), NoteColor::Yellow).unwrap();
        assert_eq!(note.title, "T1");
        assert_eq!(state.list_notes().len(), 1);

        // 重新从 storage 加载验证持久化
        let storage = Storage {
            data_dir: state.storage().data_dir().to_path_buf(),
        };
        let reloaded = storage.load_data().unwrap();
        assert_eq!(reloaded.notes.len(), 1);
        assert_eq!(reloaded.notes[0].title, "T1");
    }

    #[test]
    fn update_note_partial_fields() {
        let (_tmp, state) = make_state();
        let note = state.create_note("T".into(), "C".into(), NoteColor::Yellow).unwrap();

        let fields = UpdateNoteFields {
            title: Some("New".into()),
            content: None,
            color: Some(NoteColor::Pink),
            width: None,
            height: None,
            x: Some(200),
            y: None,
            monitor: None,
        };
        let updated = state.update_note(&note.id, &fields).unwrap();
        assert_eq!(updated.title, "New");
        assert_eq!(updated.color, NoteColor::Pink);
        assert_eq!(updated.x, 200);
        assert_eq!(updated.content, "C"); // 未更新
        assert_eq!(updated.height, 240); // 未更新
    }

    #[test]
    fn update_note_not_found() {
        let (_tmp, state) = make_state();
        let fields = UpdateNoteFields {
            title: Some("X".into()),
            content: None,
            color: None,
            width: None,
            height: None,
            x: None,
            y: None,
            monitor: None,
        };
        assert!(state.update_note("nonexistent", &fields).is_err());
    }

    #[test]
    fn delete_note_removes_and_persists() {
        let (_tmp, state) = make_state();
        let n1 = state.create_note("A".into(), "".into(), NoteColor::Yellow).unwrap();
        let _n2 = state.create_note("B".into(), "".into(), NoteColor::Pink).unwrap();

        state.delete_note(&n1.id).unwrap();
        assert_eq!(state.list_notes().len(), 1);
        assert_eq!(state.list_notes()[0].title, "B");
    }

    #[test]
    fn set_pinned_toggles_state() {
        let (_tmp, state) = make_state();
        let note = state.create_note("T".into(), "".into(), NoteColor::Yellow).unwrap();
        assert!(!note.pinned);

        let pinned = state.set_pinned(&note.id, true).unwrap();
        assert!(pinned.pinned);

        let unpinned = state.set_pinned(&note.id, false).unwrap();
        assert!(!unpinned.pinned);
    }

    #[test]
    fn complete_sets_timestamp_and_persists() {
        let (_tmp, state) = make_state();
        let note = state.create_note("T".into(), "".into(), NoteColor::Yellow).unwrap();
        assert!(note.completed_at.is_none());

        std::thread::sleep(std::time::Duration::from_millis(5));
        let completed = state.complete(&note.id).unwrap();
        assert!(completed.completed_at.is_some());
        assert!(completed.completed_at.unwrap() > note.created_at);
        assert_eq!(completed.updated_at, completed.completed_at.unwrap());

        // no-op：再次 complete 不改变 timestamp
        let completed2 = state.complete(&note.id).unwrap();
        assert_eq!(completed2.completed_at, completed.completed_at);

        // 重新从 storage 加载验证持久化
        let storage = Storage {
            data_dir: state.storage().data_dir().to_path_buf(),
        };
        let reloaded = storage.load_data().unwrap();
        assert_eq!(reloaded.notes.len(), 1);
        assert_eq!(reloaded.notes[0].completed_at, completed.completed_at);
    }

    #[test]
    fn uncomplete_clears_timestamp() {
        let (_tmp, state) = make_state();
        let note = state.create_note("T".into(), "".into(), NoteColor::Yellow).unwrap();

        let completed = state.complete(&note.id).unwrap();
        assert!(completed.completed_at.is_some());

        std::thread::sleep(std::time::Duration::from_millis(5));
        let uncompleted = state.uncomplete(&note.id).unwrap();
        assert!(uncompleted.completed_at.is_none());
        assert!(uncompleted.updated_at > completed.updated_at);

        // no-op：再次 uncomplete 不报错
        assert!(state.uncomplete(&note.id).is_ok());
    }

    #[test]
    fn complete_not_found_returns_err() {
        let (_tmp, state) = make_state();
        assert!(state.complete("nope").is_err());
        assert!(state.uncomplete("nope").is_err());
    }

    #[test]
    fn old_notes_default_completed_at_none() {
        // 模拟旧 data.json（不含 completed_at 字段）反序列化时默认 None
        let json_without_field = serde_json::json!({
            "id": "old-uuid",
            "title": "old",
            "content": "",
            "color": "yellow",
            "width": 240,
            "height": 240,
            "x": 100,
            "y": 100,
            "created_at": 0,
            "updated_at": 0,
            "pinned": false,
            "rotation": 0.0,
            "monitor": "",
            // 无 completed_at 字段
        });
        let n: Note = serde_json::from_value(json_without_field).unwrap();
        assert!(n.completed_at.is_none());
    }

    #[test]
    fn update_config_partial_merge() {
        let (_tmp, state) = make_state();
        let original = state.get_config();
        assert_eq!(original.hotkey, "Ctrl+Alt+N");

        let partial = serde_json::json!({ "hotkey": "Ctrl+Shift+N" });
        let updated = state.update_config(partial).unwrap();
        assert_eq!(updated.hotkey, "Ctrl+Shift+N");
        assert_eq!(updated.theme, original.theme); // 未更新
        assert_eq!(updated.auto_backup, original.auto_backup); // 未更新
    }

    #[test]
    fn export_then_import_roundtrip() {
        let (_tmp, state) = make_state();
        state.create_note("A".into(), "content a".into(), NoteColor::Yellow).unwrap();
        state.create_note("B".into(), "content b".into(), NoteColor::Pink).unwrap();

        let export_path = state.storage().data_dir().join("export.json");
        state.export_notes(&export_path).unwrap();
        assert!(export_path.exists());

        // 新 state 导入
        let tmp2 = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp2.path().join("backups")).unwrap();
        let storage2 = Storage {
            data_dir: tmp2.path().to_path_buf(),
        };
        let state2 = AppState::from_storage(storage2).unwrap();
        assert_eq!(state2.list_notes().len(), 0);

        let result = state2.import_notes(&export_path).unwrap();
        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped, 0);
        assert_eq!(state2.list_notes().len(), 2);
    }

    #[test]
    fn import_skips_existing_ids() {
        let (_tmp, state) = make_state();
        let n = state.create_note("A".into(), "old".into(), NoteColor::Yellow).unwrap();

        // 构造一个含相同 id 的导入文件
        let export_path = state.storage().data_dir().join("export.json");
        let mut import_data = DataFile::default();
        let mut dup = n.clone();
        dup.content = "new".into();
        import_data.notes.push(dup);
        import_data.notes.push(Note::new()); // 另一个新 id
        std::fs::write(&export_path, serde_json::to_string_pretty(&import_data).unwrap()).unwrap();

        let result = state.import_notes(&export_path).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(state.list_notes().len(), 2);
    }

    /// SOP 10.2：data.json 损坏时从备份恢复
    #[test]
    fn load_data_restores_from_backup_when_corrupted() {
        let (_tmp, storage) = make_test_storage_pair();
        // 先写正常数据
        let mut data = DataFile::default();
        data.notes.push(Note::new());
        storage.save_data(&data).unwrap();

        // 创建一份备份
        std::thread::sleep(std::time::Duration::from_millis(10));
        storage.create_backup(5).unwrap();

        // 破坏 data.json
        std::fs::write(storage.data_path(), "{ corrupted json !!!").unwrap();

        // load_data 应从备份恢复
        let restored = storage.load_data().unwrap();
        assert_eq!(restored.notes.len(), 1);
    }

    /// SOP 10.4：auto_backup=true 时 save 后产生备份
    #[test]
    fn auto_backup_creates_backup_after_save() {
        let (_tmp, state) = make_state();
        // 默认 config.auto_backup=true, backup_keep=5
        state.create_note("T".into(), "C".into(), NoteColor::Yellow).unwrap();

        let backup_count = std::fs::read_dir(state.storage().backup_dir())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .ok()
                    .and_then(|e| e.file_name().to_str().map(|s| s.starts_with("data-")))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(backup_count, 1, "auto_backup=true 时 save 后应产生 1 份备份");
    }

    /// SOP 10.3：路径兜底（D 盘不存在时回退 appDataDir）
    #[test]
    fn resolve_data_dir_fallback_logic() {
        // 模拟 D 盘不存在：用临时目录作为 appDataDir
        let tmp = TempDir::new().unwrap();
        // 无论 D 盘是否存在，appDataDir 路径都应被正确处理
        let resolved = resolve_data_dir(Some(tmp.path()));
        // D 盘存在时返回 D 盘路径，不存在时返回 tmp 路径
        let d_drive = Path::new(r"D:\");
        if d_drive.exists() {
            assert_eq!(resolved.unwrap(), PathBuf::from(PREFERRED_DIR));
        } else {
            assert_eq!(resolved.unwrap(), tmp.path().to_path_buf());
        }
    }
}
