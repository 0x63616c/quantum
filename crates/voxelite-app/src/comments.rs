//! Anchored human discussion commands and camera-space pin placement.
use crate::editor::{Editor, EditorResult, OrbitCamera, raycast};
use glam::DVec3;
use voxelite_core::*;
/// Project an anchor's center; indicate when another visible voxel occludes it.
/// Hidden and orphan anchors are retained, with their state supplied by the core.
pub fn project_pin(
    snapshot: &Snapshot,
    comment: &Comment,
    camera: OrbitCamera,
    aspect: f64,
) -> Option<([f64; 2], bool)> {
    if !aspect.is_finite() || aspect <= 0.0 {
        return None;
    }
    let region = comment.anchor.region().ok()?;
    let center = DVec3::from(std::array::from_fn::<_, 3, _>(|a| {
        f64::from(region.min[a] + region.max[a]) * 0.5
    }));
    let uv = camera.project(center, aspect);
    if uv
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return None;
    }
    let occluded = camera
        .ray(uv, aspect)
        .and_then(|r| raycast(snapshot, r))
        .is_some_and(|hit| !region.contains(hit.at));
    Some((uv, occluded))
}
impl Editor {
    /// Add a plain-text thread anchored to a voxel or a half-open region.
    pub fn comment(&mut self, anchor: Anchor, text: String) -> EditorResult<Uuid> {
        let id = Uuid::new_v4();
        self.execute(
            "Add comment",
            Command::CreateComment {
                id,
                anchor,
                layer: Some(self.layer),
                message: Uuid::new_v4(),
                text,
            },
        )?;
        Ok(id)
    }
    /// Append an attributed message through shared history.
    pub fn reply(&mut self, id: Uuid, text: String) -> EditorResult<bool> {
        self.execute(
            "Reply to comment",
            Command::Reply {
                id,
                message: Uuid::new_v4(),
                text,
            },
        )
    }
    /// Resolve or reopen without deleting the discussion.
    pub fn resolve_comment(&mut self, id: Uuid, resolved: bool) -> EditorResult<bool> {
        self.execute(
            if resolved {
                "Resolve comment"
            } else {
                "Reopen comment"
            },
            Command::Resolve { id, resolved },
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discussion_survives_undo_and_file_roundtrip() {
        let mut e = Editor::blank().unwrap();
        e.apply_at([0, 0, 0]).unwrap();
        let id = e
            .comment(
                Anchor::Voxel { at: [0, 0, 0] },
                "Check the silhouette".into(),
            )
            .unwrap();
        e.reply(id, "Looks good".into()).unwrap();
        e.resolve_comment(id, true).unwrap();
        assert!(e.snapshot().comments[0].resolved);
        e.undo().unwrap();
        assert!(!e.snapshot().comments[0].resolved);
        assert!(e.reply(id, String::new()).is_err());
        assert_eq!(e.snapshot().comments[0].messages.len(), 2);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("comments.voxl");
        e.save(&path).unwrap();
        let r = Editor::open(&path).unwrap();
        assert_eq!(r.snapshot().comments, e.snapshot().comments);
        assert_eq!(r.snapshot().comments[0].messages[1].origin, Origin::Human);
    }
    #[test]
    fn pin_tracks_camera_and_retains_orphans() {
        let mut e = Editor::blank().unwrap();
        e.apply_at([0, 0, 0]).unwrap();
        e.comment(
            Anchor::Region {
                min: [0, 0, 0],
                max: [1, 1, 1],
            },
            "Region".into(),
        )
        .unwrap();
        let c = &e.snapshot().comments[0];
        let camera = OrbitCamera::fit(e.snapshot(), 1.0);
        let (uv, hidden) = project_pin(e.snapshot(), c, camera, 1.0).unwrap();
        assert!(!hidden);
        assert!(uv.iter().all(|v| (0.0..=1.0).contains(v)));
        e.tool = crate::editor::Tool::Erase;
        e.apply_at([0, 0, 0]).unwrap();
        assert_eq!(
            e.snapshot().anchor_state(&e.snapshot().comments[0]),
            AnchorState::Orphaned
        );
    }
}
