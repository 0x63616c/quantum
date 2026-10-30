//! Characterization run against the original app implementation before extraction.
use super::*;
use glam::DVec3;
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
fn edits_and_removal_use_existing_core_history() {
    let mut e = Editor::blank().unwrap();
    e.apply_at([0, 0, 0]).unwrap();
    let before = e.project().to_bytes().unwrap();
    let id = e
        .comment(Anchor::Voxel { at: [0, 0, 0] }, "Original".into())
        .unwrap();
    let created = e.project().to_bytes().unwrap();
    e.comments_action(CommentsAction::Reanchor {
        id,
        anchor: Anchor::Region {
            min: [0; 3],
            max: [2; 3],
        },
        layer: None,
    })
    .unwrap();
    e.reply(id, "Edited discussion".into()).unwrap();
    e.resolve_comment(id, true).unwrap();
    e.resolve_comment(id, false).unwrap();
    for _ in 0..4 {
        e.undo().unwrap();
    }
    assert_eq!(e.project().to_bytes().unwrap(), created);
    e.undo().unwrap();
    assert!(e.snapshot().comments.is_empty());
    assert_eq!(e.project().to_bytes().unwrap(), before);
    e.redo().unwrap();
    assert_eq!(e.project().to_bytes().unwrap(), created);
}

#[test]
fn pin_projection_picks_front_voxels_and_rejects_offscreen_anchors() {
    let mut e = Editor::blank().unwrap();
    e.apply_at([0, 0, 0]).unwrap();
    let id = e
        .comment(Anchor::Voxel { at: [0, 0, 0] }, "Behind".into())
        .unwrap();
    e.apply_at([0, 0, 2]).unwrap();
    let mut camera = OrbitCamera::fit_view(e.snapshot(), 1.0, voxelite_render::PresetView::Front);
    let c = e.snapshot().comments.iter().find(|c| c.id == id).unwrap();
    let (uv, occluded) = project_pin(&e, c, camera, 1.0).unwrap();
    assert!(occluded);
    assert_eq!(
        raycast(e.snapshot(), camera.ray(uv, 1.0).unwrap())
            .unwrap()
            .at,
        [0, 0, 2]
    );
    assert!(project_pin(&e, c, camera, f64::NAN).is_none());
    assert!(project_pin(&e, c, camera, 0.0).is_none());
    camera.target += DVec3::splat(1000.0);
    assert!(project_pin(&e, c, camera, 1.0).is_none());
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
    let (uv, hidden) = project_pin(&e, c, camera, 1.0).unwrap();
    assert!(!hidden);
    assert!(uv.iter().all(|v| (0.0..=1.0).contains(v)));
    e.update_layer(e.layer, Some(false), None).unwrap();
    assert!(e.comment_pins(camera, 1.0)[0].dim);
    e.update_layer(e.layer, Some(true), None).unwrap();
    assert!(!e.comment_pins(camera, 1.0)[0].dim);
    e.tool = crate::editor::Tool::Erase;
    e.apply_at([0, 0, 0]).unwrap();
    let orphan = &e.comment_pins(camera, 1.0)[0];
    assert!(orphan.dim);
    assert_eq!(orphan.tooltip, "Occluded / detached · Region");
    assert_eq!(
        e.snapshot().anchor_state(&e.snapshot().comments[0]),
        AnchorState::Orphaned
    );
}

fn project_pin(
    e: &Editor,
    comment: &Comment,
    camera: OrbitCamera,
    aspect: f64,
) -> Option<([f64; 2], bool)> {
    e.comment_pins(camera, aspect)
        .into_iter()
        .find(|pin| pin.id == comment.id)
        .map(|pin| (pin.uv, pin.dim))
}

#[test]
fn registered_placement_panel_and_pin_selection_preserve_document_until_post() {
    let mut e = Editor::blank().unwrap();
    e.apply_at([0; 3]).unwrap();
    let bytes = e.project().to_bytes().unwrap();
    let history = e.engine.history().count();
    let begin = CommentsAction::Command(voxelite_plugin_comments::COMMAND_ID.into());
    let out = e.comments_action(begin.clone()).unwrap().unwrap();
    assert!(out.open_panel && e.comments_state.placing);
    assert_eq!(
        out.status.as_deref(),
        Some("Click a voxel to anchor a comment")
    );
    let camera = OrbitCamera::fit(e.snapshot(), 1.0);
    let ray = camera
        .ray(camera.project(DVec3::splat(0.5), 1.0), 1.0)
        .unwrap();
    e.comment_gesture(Some(ray), ToolGesture::Click).unwrap();
    assert_eq!(e.comments_state.draft, Some(Anchor::Voxel { at: [0; 3] }));
    assert!(!e.comments_state.placing);
    assert_eq!(e.project().to_bytes().unwrap(), bytes);
    assert_eq!(e.engine.history().count(), history);
    let state = e.comments_state.clone();
    assert!(
        e.comments_action(CommentsAction::Post("  ".into()))
            .is_err()
    );
    assert_eq!(e.comments_state, state);
    assert_eq!(e.project().to_bytes().unwrap(), bytes);
    let out = e
        .comments_action(CommentsAction::Post("  Review silhouette  ".into()))
        .unwrap()
        .unwrap();
    assert!(out.clear_text && out.scroll_bottom);
    assert_eq!(
        e.snapshot().comments[0].messages[0].text,
        "Review silhouette"
    );
    assert_eq!(e.snapshot().comments[0].layer, Some(e.layer));
    assert_eq!(e.engine.history().last().unwrap().label, "Add comment");
    let pin = e.comment_pins(camera, 1.0).remove(0);
    assert_eq!(pin.number, 1);
    assert!(!pin.dim);
    e.comments_action(CommentsAction::Select(Some(pin.id)))
        .unwrap();
    assert_eq!(e.comments_view().unwrap().thread.unwrap().1.id, pin.id);
    e.resolve_comment(pin.id, true).unwrap();
    assert!(e.comment_pins(camera, 1.0).is_empty());
    e.comments_action(CommentsAction::Filter(1)).unwrap();
    assert!(e.comment_pins(camera, 1.0)[0].resolved);
    e.comments_action(CommentsAction::ToggleRegion).unwrap();
    e.comments_action(begin).unwrap();
    e.comment_gesture(None, ToolGesture::ApplyAt([3, 2, 1]))
        .unwrap();
    assert_eq!(e.comments_state.first, Some([3, 2, 1]));
    assert!(e.comments_state.draft.is_none());
    e.comment_gesture(None, ToolGesture::ApplyAt([1, 1, 1]))
        .unwrap();
    assert_eq!(
        e.comments_state.draft,
        Some(Anchor::Region {
            min: [1, 1, 1],
            max: [4, 3, 2]
        })
    );
    e.comments_action(CommentsAction::Post("Region".into()))
        .unwrap();
    let id = e.comments_state.selected.unwrap();
    assert_eq!(
        e.snapshot()
            .comments
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .layer,
        None
    );
}

#[test]
fn disabled_or_absent_comments_hide_contributions_but_preserve_file_and_undo() {
    let mut e = Editor::blank().unwrap();
    let id = e
        .comment(Anchor::Voxel { at: [0; 3] }, "Retained".into())
        .unwrap();
    let camera = OrbitCamera::fit(e.snapshot(), 1.0);
    let bytes = e.project().to_bytes().unwrap();
    let history = e.engine.history().count();
    e.set_plugin_enabled(voxelite_plugin_comments::PLUGIN_ID, false)
        .unwrap();
    assert!(!e.comments_enabled());
    assert!(e.comments_view().is_none());
    assert!(e.comment_pins(camera, 1.0).is_empty());
    assert!(
        e.comment_gesture(None, ToolGesture::ApplyAt([0; 3]))
            .unwrap()
            .is_none()
    );
    assert!(
        e.comments_action(CommentsAction::Command(
            voxelite_plugin_comments::COMMAND_ID.into()
        ))
        .unwrap()
        .is_none()
    );
    assert!(!e.resolve_comment(id, true).unwrap());
    assert!(
        e.comment(Anchor::Voxel { at: [1; 3] }, "Unavailable".into())
            .is_err()
    );
    assert_eq!(e.project().to_bytes().unwrap(), bytes);
    assert_eq!(e.engine.history().count(), history);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("disabled-comments.voxl");
    e.save(&path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let restored = Editor::open(&path).unwrap();
    assert_eq!(restored.snapshot().comments, e.snapshot().comments);
    assert!(restored.project().document.extensions.is_empty());
    e.undo().unwrap();
    assert!(e.snapshot().comments.is_empty());
    e.redo().unwrap();
    assert_eq!(e.project().to_bytes().unwrap(), bytes);
    e.set_plugin_enabled(voxelite_plugin_comments::PLUGIN_ID, true)
        .unwrap();
    assert!(e.comments_enabled());
    assert_eq!(e.comments_view().unwrap().comments[0].id, id);
    assert_eq!(e.project().to_bytes().unwrap(), bytes);
    e.plugins = std::cell::RefCell::new(voxelite_plugin_host::Host::default());
    assert!(e.comments_view().is_none());
    assert!(e.comment_pins(camera, 1.0).is_empty());
    assert_eq!(e.project().to_bytes().unwrap(), bytes);
}

#[test]
fn preview_is_read_only_and_empty_pointer_hits_do_not_create_a_draft() {
    let mut e = Editor::blank().unwrap();
    let begin = CommentsAction::Command(voxelite_plugin_comments::COMMAND_ID.into());
    e.comments_action(begin.clone()).unwrap();
    let ray = e.camera.ray([0.5; 2], 1.0).unwrap();
    assert!(
        e.comment_gesture(Some(ray), ToolGesture::Click)
            .unwrap()
            .is_none()
    );
    assert!(e.comments_state.draft.is_none());
    assert!(e.comments_state.placing);
    e.comment_gesture(None, ToolGesture::ApplyAt([0; 3]))
        .unwrap();
    e.comments_action(CommentsAction::Post("History".into()))
        .unwrap();
    let bytes = e.project().to_bytes().unwrap();
    e.preview_history(None).unwrap();
    let state = e.comments_state.clone();
    assert!(e.comments_action(begin).is_err());
    assert_eq!(e.comments_state, state);
    assert!(
        e.comment_gesture(None, ToolGesture::ApplyAt([0; 3]))
            .is_err()
    );
    assert_eq!(e.comments_state, state);
    assert!(
        e.comments_action(CommentsAction::Post("Must not write".into()))
            .is_err()
    );
    assert_eq!(e.comments_state, state);
    assert_eq!(e.project().to_bytes().unwrap(), bytes);
}
