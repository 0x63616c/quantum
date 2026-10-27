//! Bytes captured from the core export path before plugin registration.
use voxelite_core::*;
use voxelite_render::RenderOptions;

fn fixture() -> Snapshot {
    let mut document = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    for rgba in [[255, 80, 20, 255], [10, 120, 255, 255]] {
        let index = document.content.palette.slots.len() as u16 + 1;
        document.content.palette.slots.push(PaletteSlot {
            index,
            color: Swatch { rgba, name: None },
        });
        document.content.palette.order.push(index);
    }
    let mut engine = Engine::new(document).unwrap();
    engine
        .execute(Transaction {
            expected_revision: 0,
            id: Uuid::from_u128(3),
            actor: Actor::new(Origin::Agent),
            label: "Export fixture".into(),
            command: Command::SetVoxels {
                layer: Uuid::from_u128(2),
                voxels: vec![
                    Voxel {
                        at: [-1, 0, 0],
                        slot: 1,
                    },
                    Voxel {
                        at: [0, 0, 0],
                        slot: 2,
                    },
                    Voxel {
                        at: [0, 1, 0],
                        slot: 3,
                    },
                ],
                mode: Mode::Place,
                mirror: Mirror::default(),
            },
        })
        .unwrap();
    let mut snapshot = engine.document().content.clone();
    let mut hidden = snapshot.layers[0].clone();
    hidden.id = Uuid::from_u128(4);
    hidden.visible = false;
    snapshot.layers.push(hidden);
    snapshot
}

#[test]
fn pre_plugin_export_bytes() {
    let snapshot = fixture();
    let options = RenderOptions {
        width: 80,
        height: 64,
        ..Default::default()
    };
    let outputs = [
        ("palette.hex", snapshot.palette.to_hex().into_bytes()),
        (
            "palette.gpl",
            snapshot
                .palette
                .to_gpl(&snapshot.name)
                .unwrap()
                .into_bytes(),
        ),
        ("scene.json", serde_json::to_vec_pretty(&snapshot).unwrap()),
        (
            // GPUI unifies serde_json/preserve_order; both pre-existing layouts are stable.
            if serde_json::json!({"z":0,"a":0})
                .to_string()
                .starts_with("{\"z\"")
            {
                "scene.ordered.glb"
            } else {
                "scene.glb"
            },
            voxelite_export::glb(&snapshot).unwrap(),
        ),
        (
            "scene.png",
            voxelite_export::render_png(&snapshot, &options).unwrap(),
        ),
        (
            "rgba.png",
            voxelite_export::png_rgba(
                2,
                2,
                &[
                    255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 123, 45, 67, 255,
                ],
            )
            .unwrap(),
        ),
    ];
    for (name, bytes) in outputs {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden")
            .join(name);
        assert_eq!(bytes, std::fs::read(path).unwrap(), "{name}");
    }
}
