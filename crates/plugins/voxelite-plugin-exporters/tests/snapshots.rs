//! Snapshot every registered exporter through the public host dispatch path.

use voxelite_core::{Document, Engine, Uuid};
use voxelite_plugin_api::ExportImage;
use voxelite_plugin_exporters::{ExportersPlugin, GLB, GPL, HEX, PNG};
use voxelite_plugin_host::{Host, SnapshotExport};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn registered_export_outputs() {
    let mut document = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    document.content.name = "Snapshot fixture".into();
    // Endpoint colours avoid platform libm differences in sRGB conversion.
    document.content.palette.slots[0].color.rgba = [255; 4];
    let mut engine = Engine::new(document).unwrap();
    engine
        .execute(voxelite_core::Transaction {
            expected_revision: 0,
            id: Uuid::from_u128(3),
            actor: voxelite_core::Actor::new(voxelite_core::Origin::Agent),
            label: "Fixture".into(),
            command: voxelite_core::Command::SetVoxels {
                layer: Uuid::from_u128(2),
                voxels: vec![voxelite_core::Voxel {
                    at: [-1, 0, 0],
                    slot: 1,
                }],
                mode: voxelite_core::Mode::Place,
                mirror: voxelite_core::Mirror::default(),
            },
        })
        .unwrap();
    let cx = SnapshotExport {
        snapshot: &engine.document().content,
        render_view: &|_| {
            Ok(ExportImage {
                width: 2,
                height: 2,
                rgba: vec![
                    255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 123, 45, 67, 255,
                ],
            })
        },
    };
    let mut host = Host::new(vec![Box::new(ExportersPlugin)]);
    assert_eq!(host.registry().formats().count(), 4);
    for (id, name) in [(HEX, "palette_hex"), (GPL, "palette_gpl")] {
        let bytes = host.dispatch_export(id, &cx).unwrap().unwrap();
        insta::assert_snapshot!(name, String::from_utf8(bytes).unwrap());
    }
    let png = host.dispatch_export(PNG, &cx).unwrap().unwrap();
    insta::assert_snapshot!(
        "png_bytes",
        format!("length: {}\nhex: {}", png.len(), hex(&png))
    );
    let glb = host.dispatch_export(GLB, &cx).unwrap().unwrap();
    let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&glb[20..20 + json_length]).unwrap();
    // Workspace feature unification can change serde_json object insertion order.
    metadata.sort_all_objects();
    insta::assert_json_snapshot!("glb_metadata", metadata);
    insta::assert_snapshot!(
        "glb_bytes",
        format!(
            "length: {}\nheader: {}\nbinary chunk: {}",
            glb.len(),
            hex(&glb[..20]),
            hex(&glb[20 + json_length..]),
        )
    );
}
