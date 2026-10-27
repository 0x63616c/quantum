use super::*;
use voxelite_plugin_host::SnapshotExport;

#[test]
fn disabling_lighting_changes_pixels_without_changing_save_or_undo() {
    let mut editor = Editor::blank().unwrap();
    editor.apply_at([0, 0, 0]).unwrap();
    let before = editor.project().to_bytes().unwrap();
    let head = editor.engine.head();
    let revision = editor.engine.revision();
    let options = |editor: &Editor| voxelite_render::RenderOptions {
        width: 32,
        height: 32,
        passes: editor.render_plan(),
        ..Default::default()
    };
    let enabled = editor
        .raster
        .render(editor.snapshot(), &options(&editor))
        .unwrap();
    editor
        .set_plugin_enabled(voxelite_plugin_shadows_lighting::PLUGIN_ID, false)
        .unwrap();
    let fallback = editor
        .raster
        .render(editor.snapshot(), &options(&editor))
        .unwrap();
    assert_ne!(fallback.rgba, enabled.rgba);
    let png = editor.export_format("png").unwrap();
    let bytes = editor
        .finish_export(
            editor
                .prepare_export(&png.id)
                .unwrap()
                .run(&SnapshotExport {
                    snapshot: editor.snapshot(),
                    render_view: &render_view(&options(&editor)),
                }),
        )
        .unwrap();
    assert_eq!(
        bytes,
        voxelite_plugin_api::encoding::png_rgba(32, 32, &fallback.rgba).unwrap()
    );
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    assert_eq!(editor.engine.revision(), revision);
    editor.undo().unwrap();
    editor.redo().unwrap();
    assert_eq!(editor.engine.head(), head);
    editor
        .set_plugin_enabled(voxelite_plugin_shadows_lighting::PLUGIN_ID, true)
        .unwrap();
    assert_eq!(
        editor
            .raster
            .render(editor.snapshot(), &options(&editor))
            .unwrap(),
        enabled
    );
}

#[test]
fn exports_preserve_voxl_history_and_disabled_exporters_do_not_break_save_or_undo() {
    let mut editor = Editor::blank().unwrap();
    editor
        .execute(
            "Fixture",
            Command::SetVoxels {
                layer: editor.layer,
                voxels: vec![Voxel {
                    at: [0; 3],
                    slot: 1,
                }],
                mode: Mode::Place,
                mirror: Mirror::default(),
            },
        )
        .unwrap();
    let before = editor.project().to_bytes().unwrap();
    let revision = editor.engine.revision();
    let head = editor.engine.head();
    let options = voxelite_render::RenderOptions {
        width: 32,
        height: 32,
        ..Default::default()
    };
    for ext in ["glb", "png", "hex", "gpl"] {
        let format = editor.export_format(ext).unwrap();
        let job = editor.prepare_export(&format.id).unwrap();
        let completion = job.run(&SnapshotExport {
            snapshot: editor.snapshot(),
            render_view: &render_view(&options),
        });
        assert!(!editor.finish_export(completion).unwrap().is_empty());
    }
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    assert_eq!(editor.engine.revision(), revision);
    assert_eq!(editor.engine.head(), head);
    let png = editor.export_format("png").unwrap();
    let job = editor.prepare_export(&png.id).unwrap();
    let completion = job.run(&SnapshotExport {
        snapshot: editor.snapshot(),
        render_view: &render_view(&options),
    });
    editor
        .set_plugin_enabled("dev.voxelite.exporters", false)
        .unwrap();
    assert!(editor.finish_export(completion).is_err());
    assert!(editor.export_format("png").is_err());
    assert!(editor.prepare_export(&png.id).is_err());
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    editor.undo().unwrap();
    editor.redo().unwrap();
    assert_eq!(editor.engine.head(), head);
    #[cfg(not(target_family = "wasm"))]
    {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("still-saves.voxl");
        let saved = editor.prepare_save(&path).write().unwrap();
        editor.finish_save(saved);
        assert!(!editor.needs_save());
        assert_eq!(Editor::open(&path).unwrap().snapshot(), editor.snapshot());
    }
    editor
        .set_plugin_enabled("dev.voxelite.exporters", true)
        .unwrap();
    assert_eq!(editor.export_format("png").unwrap(), png);
}
