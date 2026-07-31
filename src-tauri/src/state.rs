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

    /// 获取单个便签
    pub fn get_note(&self, id: &str) -> Result<Note> {
        self.data
            .lock()
            .unwrap()
            .notes
            .iter()
            .find(|n| n.id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound(id.to_string()))
    }

    /// 创建便签
    /// 返回新便签（调用方负责 emit 事件）
    pub fn create_note(&self, title: String, content: String, color: NoteColor) -> Note {
        let mut note = Note::new();
        note.title = title;
        note.content = content;
        note.color = color;

        let mut data = self.data.lock().unwrap();
        data.notes.push(note.clone());
        // 同步落盘（阶段 10 改为防抖）
        let _ = self.storage.save_data(&data);
        note
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
        note.touch();

        let updated = note.clone();
        let _ = self.storage.save_data(&data);
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
        let _ = self.storage.save_data(&data);
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
        let _ = self.storage.save_data(&data);
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
        let _ = self.storage.save_config(&config);
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

        let _ = self.storage.save_data(&data);
        Ok(ImportResult { imported, skipped })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_state() -> (TempDir, AppState) {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().to_path_buf();
        std::fs::create_dir_all(data_dir.join("backups")).unwrap();
        let storage = Storage { data_dir };
        let state = AppState::from_storage(storage).unwrap();
        (tmp, state)
    }

    #[test]
    fn create_note_persists_and_returns() {
        let (_tmp, state) = make_state();
        let note = state.create_note("T1".into(), "C1".into(), NoteColor::Yellow);
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
        let note = state.create_note("T".into(), "C".into(), NoteColor::Yellow);

        let fields = UpdateNoteFields {
            title: Some("New".into()),
            content: None,
            color: Some(NoteColor::Pink),
            width: None,
            height: None,
            x: Some(200),
            y: None,
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
        };
        assert!(state.update_note("nonexistent", &fields).is_err());
    }

    #[test]
    fn delete_note_removes_and_persists() {
        let (_tmp, state) = make_state();
        let n1 = state.create_note("A".into(), "".into(), NoteColor::Yellow);
        let _n2 = state.create_note("B".into(), "".into(), NoteColor::Pink);

        state.delete_note(&n1.id).unwrap();
        assert_eq!(state.list_notes().len(), 1);
        assert_eq!(state.list_notes()[0].title, "B");
    }

    #[test]
    fn set_pinned_toggles_state() {
        let (_tmp, state) = make_state();
        let note = state.create_note("T".into(), "".into(), NoteColor::Yellow);
        assert!(!note.pinned);

        let pinned = state.set_pinned(&note.id, true).unwrap();
        assert!(pinned.pinned);

        let unpinned = state.set_pinned(&note.id, false).unwrap();
        assert!(!unpinned.pinned);
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
        state.create_note("A".into(), "content a".into(), NoteColor::Yellow);
        state.create_note("B".into(), "content b".into(), NoteColor::Pink);

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
        let n = state.create_note("A".into(), "old".into(), NoteColor::Yellow);

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
}
