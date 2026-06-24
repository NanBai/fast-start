//! Session 删除相关的文件系统操作边界。
//!
//! 删除是本地 destructive action，校验和执行逻辑集中放在这里，
//! 避免把路径白名单和 `remove_*` 细节扩散到 state 或 scanner。

use crate::models::{SessionDeleteKind, SessionDeleteTarget};
use std::fs;

#[derive(Debug, PartialEq, Eq)]
pub enum DeleteSessionError {
    MissingTarget,
    RootUnavailable,
    TargetMissing,
    OutsideRoot,
    KindMismatch,
    Io,
}

impl DeleteSessionError {
    pub fn message(&self) -> String {
        match self {
            DeleteSessionError::MissingTarget => "删除目标缺失，请刷新 session 列表".to_string(),
            DeleteSessionError::RootUnavailable => {
                "删除目标根目录不可用，请刷新 session 列表".to_string()
            }
            DeleteSessionError::TargetMissing => "删除目标不存在，请刷新 session 列表".to_string(),
            DeleteSessionError::OutsideRoot => "删除目标不在允许目录内".to_string(),
            DeleteSessionError::KindMismatch => "删除目标类型不匹配".to_string(),
            DeleteSessionError::Io => "删除失败：权限不足或文件被占用".to_string(),
        }
    }
}

pub fn delete_session_target(
    target: Option<&SessionDeleteTarget>,
) -> Result<(), DeleteSessionError> {
    let target = target.ok_or(DeleteSessionError::MissingTarget)?;
    let root = fs::canonicalize(&target.root).map_err(|_| DeleteSessionError::RootUnavailable)?;
    let path = fs::canonicalize(&target.path).map_err(|_| DeleteSessionError::TargetMissing)?;

    if path == root || !path.starts_with(&root) {
        return Err(DeleteSessionError::OutsideRoot);
    }

    match target.kind {
        SessionDeleteKind::File => {
            if !path.is_file() {
                return Err(DeleteSessionError::KindMismatch);
            }
            fs::remove_file(&path).map_err(|_| DeleteSessionError::Io)
        }
        SessionDeleteKind::Directory => {
            if !path.is_dir() {
                return Err(DeleteSessionError::KindMismatch);
            }
            fs::remove_dir_all(&path).map_err(|_| DeleteSessionError::Io)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{delete_session_target, DeleteSessionError};
    use crate::models::{SessionDeleteKind, SessionDeleteTarget};
    use std::fs;

    #[test]
    fn deletes_file_target_inside_root() {
        let temp = tempfile::tempdir().unwrap();
        let session_file = temp.path().join("session.jsonl");
        fs::write(&session_file, "{}").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: session_file.clone(),
            kind: SessionDeleteKind::File,
        };

        delete_session_target(Some(&target)).unwrap();

        assert!(!session_file.exists());
    }

    #[test]
    fn deletes_directory_target_inside_root() {
        let temp = tempfile::tempdir().unwrap();
        let chat_dir = temp.path().join("hash").join("chat");
        fs::create_dir_all(&chat_dir).unwrap();
        fs::write(chat_dir.join("meta.json"), "{}").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: chat_dir.clone(),
            kind: SessionDeleteKind::Directory,
        };

        delete_session_target(Some(&target)).unwrap();

        assert!(!chat_dir.exists());
    }

    #[test]
    fn rejects_target_outside_root() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("session.jsonl");
        fs::write(&outside_file, "{}").unwrap();
        let target = SessionDeleteTarget {
            root: root.path().to_path_buf(),
            path: outside_file.clone(),
            kind: SessionDeleteKind::File,
        };

        let err = delete_session_target(Some(&target)).unwrap_err();

        assert_eq!(err, DeleteSessionError::OutsideRoot);
        assert!(outside_file.exists());
    }

    #[test]
    fn rejects_missing_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: temp.path().join("missing.jsonl"),
            kind: SessionDeleteKind::File,
        };

        let err = delete_session_target(Some(&target)).unwrap_err();

        assert_eq!(err, DeleteSessionError::TargetMissing);
    }

    #[test]
    fn rejects_root_directory_as_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: temp.path().to_path_buf(),
            kind: SessionDeleteKind::Directory,
        };

        let err = delete_session_target(Some(&target)).unwrap_err();

        assert_eq!(err, DeleteSessionError::OutsideRoot);
        assert!(temp.path().exists());
    }
}
