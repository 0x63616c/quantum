//! Read-only bundled projects. Every open creates a pathless document session.
use crate::editor::EditorResult;
use voxelite_core::Limits;
use voxelite_format::session::DocumentSession;

/// One of the four launcher dioramas, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sample {
    /// Autumn workshop and garden.
    Autumn,
    /// Coastal lighthouse.
    Lighthouse,
    /// Cherry tree beside a pond.
    Cherry,
    /// Snow-covered cabin and fir trees.
    Snow,
}
impl Sample {
    /// Stable launcher ordering.
    pub const ALL: [Self; 4] = [Self::Autumn, Self::Lighthouse, Self::Cherry, Self::Snow];
    /// Display name of the bundled document.
    pub fn name(self) -> &'static str {
        match self {
            Self::Autumn => "Autumn Workshop",
            Self::Lighthouse => "Lighthouse",
            Self::Cherry => "Cherry Blossom",
            Self::Snow => "Snow Cabin",
        }
    }
    /// Canonical project bytes, included in the executable on every desktop platform.
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::Autumn => crate::editor::AUTUMN_PROJECT_BYTES,
            Self::Lighthouse => include_bytes!("../../../examples/lighthouse.voxl"),
            Self::Cherry => include_bytes!("../../../examples/cherry-blossom.voxl"),
            Self::Snow => include_bytes!("../../../examples/snow-cabin.voxl"),
        }
    }
    /// Parse off the window thread; no bundled file can become a save destination.
    pub fn session(self) -> EditorResult<DocumentSession> {
        Ok(DocumentSession::new(
            voxelite_format::from_bytes(self.bytes())?,
            None,
            Limits::default(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_samples_are_valid_pathless_copies_with_named_layers() {
        for sample in Sample::ALL {
            let session = sample.session().unwrap();
            assert!(session.path.is_none());
            assert!(session.needs_save());
            let content = &session.engine.document().content;
            assert_eq!(content.name, sample.name());
            assert!(!content.composite().is_empty());
            assert!(content.layers.iter().all(|layer| !layer.name.is_empty()));
        }
    }
    #[test]
    fn editing_and_saving_a_sample_never_changes_the_next_unsaved_copy() {
        use crate::editor::Editor;
        let dir = tempfile::tempdir().unwrap();
        for sample in Sample::ALL {
            let original = sample.session().unwrap().engine.document().content.clone();
            let mut editor = Editor::from_session(sample.session().unwrap()).unwrap();
            assert!(editor.path.is_none());
            editor
                .execute(
                    "Rename copy",
                    voxelite_core::Command::EditDocument {
                        name: Some("My copy".into()),
                        bounds: None,
                    },
                )
                .unwrap();
            assert!(editor.engine.is_dirty());
            let path = dir.path().join(format!("{}.voxl", sample.name()));
            editor.save(&path).unwrap();
            assert!(!editor.needs_save());
            assert_eq!(Editor::open(&path).unwrap().snapshot().name, "My copy");
            let next = sample.session().unwrap();
            assert!(next.path.is_none());
            assert!(next.needs_save());
            assert_eq!(next.engine.document().content, original);
        }
    }
}
