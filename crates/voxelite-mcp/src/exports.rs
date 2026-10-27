//! Resolve wire export requests against the enabled host registry.
use crate::error::{Failure, Result};
use voxelite_plugin_api::{ExportCx, FormatContent, Snapshot};
use voxelite_plugin_host::{Host, SnapshotExport};

pub(crate) fn host() -> Host {
    Host::new(vec![
        Box::new(voxelite_plugin_exporters::ExportersPlugin),
        Box::new(voxelite_plugin_palette::PalettePlugin),
        Box::new(voxelite_plugin_comments::CommentsPlugin),
    ])
}

pub(crate) fn extensions(host: &Host) -> Vec<String> {
    std::iter::once("voxl".into())
        .chain(
            host.registry()
                .formats()
                .filter(|(_, f)| f.export)
                .flat_map(|(_, f)| f.extensions.clone()),
        )
        .collect()
}

pub(crate) fn format(host: &Host, extension: &str) -> Result<String> {
    host.registry()
        .formats()
        .find(|(_, f)| f.export && f.extensions.iter().any(|ext| ext == extension))
        .map(|(_, f)| f.id.clone())
        .ok_or_else(|| Failure::new("export_unavailable", "Export format is unavailable"))
}

pub(crate) fn encode(host: &mut Host, format: &str, cx: &dyn ExportCx) -> Result<Vec<u8>> {
    host.dispatch_export(format, cx)
        .map_err(|e| Failure::new("export_failed", e.to_string()))?
        .ok_or_else(|| Failure::new("export_unavailable", "Export format is unavailable"))
}

pub(crate) fn snapshot(host: &mut Host, extension: &str, snapshot: &Snapshot) -> Result<Vec<u8>> {
    let format = format(host, extension)?;
    encode(
        host,
        &format,
        &SnapshotExport {
            snapshot,
            options: &Default::default(),
        },
    )
}

pub(crate) fn tools(host: &Host) -> Vec<rmcp::model::Tool> {
    crate::catalog::CATALOG
        .iter()
        .filter(|spec| {
            if let Some(ext) = spec.tool.name.strip_prefix("voxelite_export_") {
                format(host, ext).is_ok()
            } else if spec.tool.name == "voxelite_palette_export" {
                host.registry()
                    .formats()
                    .any(|(_, f)| f.export && f.content == FormatContent::Palette)
            } else if spec.tool.name.starts_with("voxelite_palette_") {
                crate::palettes::enabled(host)
            } else if spec.tool.name.starts_with("voxelite_comments_") {
                crate::comments::enabled(host)
            } else {
                true
            }
        })
        .map(|spec| spec.tool.clone())
        .collect()
}
