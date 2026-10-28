use super::*;
use crate::editor::Editor;

#[test]
fn editor_failures_are_matchable_and_keep_their_messages() {
    let cases = [
        (
            EditorError::ExportUnavailable,
            "Export format is unavailable",
        ),
        (
            EditorError::CommentsUnavailable,
            "Comments plugin is unavailable",
        ),
        (
            EditorError::HistoryReadOnly,
            "History preview is read-only. Return to live or Restore.",
        ),
        (EditorError::VersionMissing, "Version no longer exists"),
        (
            EditorError::HistorySelectionRequired,
            "Choose a history state first",
        ),
        (
            EditorError::MixedPaletteEdits,
            "Palette request mixes library and document edits",
        ),
        (EditorError::SettingsTooLarge, "Settings file too large"),
        (EditorError::ProjectNotFile, "Project is not a file"),
        (
            EditorError::PreviewChanged,
            "Project changed while creating its preview",
        ),
        (
            EditorError::PaletteImportTooLarge,
            "Palette files must be 64 KiB or smaller.",
        ),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
    let mut editor = Editor::blank().unwrap();
    assert!(matches!(
        editor.restore_preview(),
        Err(EditorError::HistorySelectionRequired)
    ));
    assert!(matches!(
        editor.preview_version(quantum_core::Uuid::new_v4()),
        Err(EditorError::VersionMissing)
    ));
    assert!(matches!(
        editor.prepare_export("missing"),
        Err(EditorError::ExportUnavailable)
    ));
}

#[test]
fn session_policy_failures_retain_their_kind_and_text() {
    for source in [
        SessionError::NotRegular,
        SessionError::TooLarge,
        SessionError::MissingFilename,
        SessionError::InvalidDestination,
        SessionError::ExternalChange,
        SessionError::FileExists,
        SessionError::DirectorySync(std::io::Error::other("sync failed")),
    ] {
        let expected = source.to_string();
        let error = EditorError::from(source);
        assert_eq!(error.to_string(), expected);
        assert!(matches!(error, EditorError::Session(_)));
    }
    let error = EditorError::from(SessionError::Io(std::io::Error::other("read failed")));
    assert!(matches!(error, EditorError::Io(_)));
}

#[test]
fn plugin_failures_retain_nested_kinds_and_messages() {
    let source = HostError::UnknownPlugin {
        id: "missing".into(),
    };
    let expected = source.to_string();
    let error = EditorError::from(source);
    assert_eq!(error.to_string(), expected);
    assert!(matches!(
        error,
        EditorError::Host(HostError::UnknownPlugin { .. })
    ));
    let error = EditorError::from(HostError::Palette(PaletteError::Capacity));
    assert_eq!(error.to_string(), PaletteError::Capacity.to_string());
    assert!(matches!(
        error,
        EditorError::Palette(PaletteError::Capacity)
    ));
    let error = EditorError::from(PaletteError::Message("invalid palette".into()));
    assert_eq!(error.to_string(), "invalid palette");
    assert!(matches!(
        error,
        EditorError::Palette(PaletteError::Message(_))
    ));
}

#[test]
fn encoding_failures_keep_their_sources() {
    let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let expected = source.to_string();
    let error = EditorError::from(source);
    assert_eq!(error.to_string(), expected);
    assert!(matches!(error, EditorError::Json(_)));
    let source = String::from_utf8(vec![255]).unwrap_err();
    let expected = source.to_string();
    let error = EditorError::from(source);
    assert_eq!(error.to_string(), expected);
    assert!(matches!(error, EditorError::PaletteUtf8(_)));
    let error = EditorError::PaletteDialog(anyhow::anyhow!("dialog failed"));
    assert_eq!(error.to_string(), "dialog failed");
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn preview_encoding_errors_keep_their_kinds_and_text() {
    let source = quantum_render::RenderError::Invalid("camera".into());
    let expected = source.to_string();
    let error = EditorError::from(source);
    assert_eq!(error.to_string(), expected);
    assert!(matches!(error, EditorError::Render(_)));
    let source = png::EncodingError::IoError(std::io::Error::other("encode failed"));
    let expected = source.to_string();
    let error = EditorError::from(source);
    assert_eq!(error.to_string(), expected);
    assert!(matches!(error, EditorError::Png(_)));
}

#[test]
fn bounded_settings_reads_return_a_typed_failure() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::File::create(&path)
        .unwrap()
        .set_len(4 * 1024 * 1024 + 1)
        .unwrap();
    assert!(matches!(
        crate::local_settings::read_local(&path),
        Err(EditorError::SettingsTooLarge)
    ));
}
