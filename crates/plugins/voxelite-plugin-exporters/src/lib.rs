//! Built-in byte-stable GLB and PNG exporters, with no filesystem capabilities.
use voxelite_plugin_api::{
    ExportCallback, ExportCx, ExportResult, FormatContent, FormatContribution, Manifest, Plugin,
    Registrar,
};

pub mod encoding;

/// Stable identity of the optional built-in exporters.
pub const PLUGIN_ID: &str = "dev.voxelite.exporters";
/// Visible composite geometry as glTF 2.0 binary.
pub const GLB: &str = "dev.voxelite.exporters.glb";
/// A host-selected view or contact sheet as lossless RGBA PNG.
pub const PNG: &str = "dev.voxelite.exporters.png";

/// HEX palette text in display order.
pub const HEX: &str = "dev.voxelite.exporters.hex";
/// GIMP palette text, retaining the host-selected palette name.
pub const GPL: &str = "dev.voxelite.exporters.gpl";

/// Standard exporters; native project saving remains a core responsibility.
pub struct ExportersPlugin;

impl Plugin for ExportersPlugin {
    fn manifest(&self) -> Manifest {
        Manifest::builtin(PLUGIN_ID, "plugins.exporters", env!("CARGO_PKG_VERSION"))
    }

    fn contribute(&self, registrar: &mut Registrar) {
        for (id, label, extension, content) in [
            (GLB, "glTF Binary (.glb)", "glb", FormatContent::Document),
            (PNG, "PNG Image (.png)", "png", FormatContent::Document),
            (HEX, "HEX Palette (.hex)", "hex", FormatContent::Palette),
            (GPL, "GIMP Palette (.gpl)", "gpl", FormatContent::Palette),
        ] {
            registrar.format(FormatContribution {
                content,
                id: id.into(),
                label_key: label.into(),
                extensions: vec![extension.into()],
                import: false,
                export: true,
            });
        }
    }

    fn exporter(&self, id: &str) -> Option<ExportCallback> {
        match id {
            GLB => Some(|cx| encoding::glb(cx.snapshot())),
            PNG => Some(png),
            HEX => Some(|cx| Ok(cx.palette().to_hex().into_bytes())),
            GPL => Some(|cx| {
                cx.palette()
                    .to_gpl(&cx.snapshot().name)
                    .map(String::into_bytes)
                    .map_err(|error| voxelite_plugin_api::ExportError::Invalid(error.to_string()))
            }),
            _ => None,
        }
    }
}

fn png(cx: &dyn ExportCx) -> ExportResult<Vec<u8>> {
    let image = cx.render_view()?;
    encoding::png_rgba(image.width, image.height, &image.rgba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxelite_plugin_api::ExportImage;
    use voxelite_plugin_host::{Host, SnapshotExport};

    voxelite_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
        ExportersPlugin
    ));

    #[test]
    fn registered_exports_match_pre_migration_bytes_and_disable_cleanly() {
        let snapshot =
            serde_json::from_slice(include_bytes!("../tests/golden/scene.json")).unwrap();
        let before = serde_json::to_vec(&snapshot).unwrap();
        let options = voxelite_render::RenderOptions {
            width: 80,
            height: 64,
            ..Default::default()
        };
        let cx = SnapshotExport {
            snapshot: &snapshot,
            render_view: &|snapshot| {
                let image = voxelite_render::render(snapshot, &options)?;
                Ok(ExportImage {
                    width: image.width,
                    height: image.height,
                    rgba: image.rgba,
                })
            },
        };
        let mut host = Host::new(vec![Box::new(ExportersPlugin)]);
        for (id, golden) in [
            (
                GLB,
                if serde_json::json!({"z":0,"a":0})
                    .to_string()
                    .starts_with("{\"z\"")
                {
                    include_bytes!("../tests/golden/scene.ordered.glb").as_slice()
                } else {
                    include_bytes!("../tests/golden/scene.glb").as_slice()
                },
            ),
            (PNG, include_bytes!("../tests/golden/scene.png").as_slice()),
            (
                HEX,
                include_bytes!("../tests/golden/palette.hex").as_slice(),
            ),
            (
                GPL,
                include_bytes!("../tests/golden/palette.gpl").as_slice(),
            ),
        ] {
            let actual = host.dispatch_export(id, &cx).unwrap().unwrap();
            if id == GLB {
                assert_glb_matches(&actual, golden);
            } else {
                assert_eq!(actual, golden);
            }
        }
        host.disable(PLUGIN_ID).unwrap();
        assert_eq!(host.registry().formats().count(), 0);
        assert!(host.dispatch_export(GLB, &cx).unwrap().is_none());
        assert!(host.dispatch_export(PNG, &cx).unwrap().is_none());
        host.enable(PLUGIN_ID).unwrap();
        assert_eq!(host.registry().formats().count(), 4);
        assert_eq!(
            host.dispatch_export(PNG, &cx).unwrap().unwrap(),
            include_bytes!("../tests/golden/scene.png")
        );
        assert_eq!(serde_json::to_vec(&snapshot).unwrap(), before);
    }

    /// libm differences across platforms may move linearised colours by one ulp; everything else is exact.
    #[allow(clippy::chunks_exact_to_as_chunks)]
    fn assert_glb_matches(actual: &[u8], expected: &[u8]) {
        assert_eq!(actual.len(), expected.len());
        let json_len = u32::from_le_bytes(expected[12..16].try_into().unwrap()) as usize;
        let bin_start = 20 + json_len + 8;
        assert_eq!(actual[..bin_start], expected[..bin_start]);
        for (a, e) in actual[bin_start..]
            .chunks_exact(4)
            .zip(expected[bin_start..].chunks_exact(4))
        {
            if a == e {
                continue;
            }
            let a = u32::from_le_bytes(a.try_into().unwrap());
            let e = u32::from_le_bytes(e.try_into().unwrap());
            assert!(
                f32::from_bits(a).abs() > 1e-3
                    && f32::from_bits(e).abs() > 1e-3
                    && a.abs_diff(e) <= 1,
                "{a:#x} != {e:#x}"
            );
        }
    }
}
