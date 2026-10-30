//! Interactive startup keeps file failures recoverable on the Welcome window.
use crate::editor::Editor;
use std::path::Path;

/// Resolve a CLI project path into an editor or a Welcome error.
/// No path opens Welcome without an error. Headless exports use `Editor::open`
/// directly so their callers still receive a failing result.
pub fn open(path: Option<&Path>) -> (Option<Editor>, Option<String>) {
    match path.map(Editor::open) {
        Some(Ok(editor)) => (Some(editor), None),
        Some(Err(error)) => (None, Some(error.to_string())),
        None => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_bad_files_return_welcome_with_filename() {
        let dir = tempfile::tempdir().unwrap();
        let invalid = dir.path().join("invalid.voxl");
        std::fs::write(&invalid, b"invalid project").unwrap();
        for path in [
            invalid,
            dir.path().join("missing.voxl"),
            dir.path().to_owned(),
        ] {
            let (editor, error) = open(Some(&path));
            assert!(editor.is_none());
            assert!(error.unwrap().contains(&path.display().to_string()));
        }
        let (editor, error) = open(None);
        assert!(editor.is_none() && error.is_none());
    }

    #[test]
    fn cli_valid_file_opens_editor() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("valid.voxl");
        Editor::blank().unwrap().save(&path).unwrap();
        let (editor, error) = open(Some(&path));
        assert!(editor.is_some() && error.is_none());
    }

    #[test]
    fn in_app_failed_load_preserves_dirty_document_and_history() {
        let dir = tempfile::tempdir().unwrap();
        // macOS temp dirs sit behind the /var -> /private/var symlink and
        // `Editor::save` stores the canonical path.
        let root = dir.path().canonicalize().unwrap();
        let invalid = root.join("invalid.voxl");
        std::fs::write(&invalid, b"invalid project").unwrap();
        let mut editor = Editor::blank().unwrap();
        let path = root.join("current.voxl");
        editor.save(&path).unwrap();
        editor.apply_at([1, 2, 3]).unwrap();
        let before = editor.engine.document().clone();
        let history = editor.engine.history().count();
        let error = Editor::load_session(&invalid).err().unwrap();
        assert!(error.to_string().contains("invalid.voxl"));
        assert_eq!(editor.path.as_ref(), Some(&path));
        assert_eq!(editor.engine.document(), &before);
        assert_eq!(editor.engine.history().count(), history);
        assert!(editor.engine.is_dirty());
        editor.undo().unwrap();
        assert!(!editor.engine.is_dirty());
    }
}
