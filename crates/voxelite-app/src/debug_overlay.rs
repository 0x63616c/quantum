//! Pure text presentation of renderer diagnostics, usable without GPUI or a GPU.
use crate::frame_meter::{FrameAverage, RendererKind};
use voxelite_render::diagnostics::FrameStats;

/// Lines for the last completed viewport frame. Presentation cadence comes from
/// FrameMeter, independently of the renderer's CPU/GPU work durations.
pub fn lines(
    renderer: RendererKind,
    average: Option<FrameAverage>,
    stats: Option<&FrameStats>,
) -> Vec<String> {
    let cadence = average.map_or_else(
        || "n/a FPS / n/a ms".into(),
        |a| format!("{:.1} FPS / {:.2} ms", a.fps, a.frame_ms),
    );
    let mut lines = vec![format!("{}  {cadence}", renderer.label())];
    let Some(s) = stats.filter(|_| renderer == RendererKind::Gpu) else {
        lines.push(if renderer == RendererKind::Cpu {
            "Renderer diagnostics: n/a (CPU viewport)".into()
        } else {
            "Renderer diagnostics: awaiting frame".into()
        });
        return lines;
    };
    lines.extend([
        format!("CPU wall {:.2} ms", s.wall_ms),
        format!(
            "  sync    occ {:.2} / mesh {:.2} ms",
            s.occupancy_sync_ms, s.mesh_sync_ms
        ),
        format!(
            "  upload  occ {:.2} / mesh {:.2} ms",
            s.occupancy_upload_ms, s.mesh_upload_ms
        ),
        format!(
            "  encode {:.2} / submit+complete {:.2} ms",
            s.encode_ms, s.submit_complete_ms
        ),
    ]);
    if let Some(gpu) = s.gpu {
        lines.push(format!(
            "GPU main {:.2} / resolve {:.2} ms",
            gpu.main_ms, gpu.resolve_ms
        ));
        lines.push(format!(
            "  shadow {} / total {:.2} ms",
            gpu.shadow_ms
                .map_or_else(|| "n/a".into(), |ms| format!("{ms:.2} ms")),
            gpu.total_ms
        ));
    } else {
        lines.push("GPU main n/a / resolve n/a".into());
        lines.push("  shadow n/a / total n/a".into());
    }
    lines.extend([
        format!(
            "Chunks {} resident / {} updated",
            s.resident_chunks, s.chunks_updated
        ),
        format!(
            "Bricks {} occupied / {} total (8³)",
            s.occupied_bricks, s.total_bricks
        ),
        format!(
            "Draws {} main / 1 resolve / {} shadow",
            s.draw_calls, s.shadow_draw_calls
        ),
        format!("Vertices {} / triangles {}", s.vertices, s.triangles),
        format!(
            "Memory vertices {} / occ {}",
            bytes(s.vertex_bytes),
            bytes(s.occupancy_bytes)
        ),
        format!(
            "  targets {} / device {}",
            bytes(s.target_bytes),
            s.device_allocated_bytes.map_or_else(|| "n/a".into(), bytes)
        ),
        format!("Uploaded {}", bytes(s.upload_bytes)),
        format!(
            "Target {} × {} / shadow {}",
            s.target_size[0],
            s.target_size[1],
            s.shadow_path.as_str()
        ),
    ]);
    lines
}

fn bytes(value: u64) -> String {
    if value < 1024 {
        format!("{value} B")
    } else if value < 1024 * 1024 {
        format!("{:.1} KiB", value as f64 / 1024.0)
    } else {
        format!("{:.1} MiB", value as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxelite_render::diagnostics::{GpuTimes, ShadowPath};

    #[test]
    fn warmup_and_cpu_fallback_do_not_show_stale_gpu_stats() {
        assert_eq!(
            lines(RendererKind::Gpu, None, None),
            [
                "GPU  n/a FPS / n/a ms",
                "Renderer diagnostics: awaiting frame",
            ]
        );
        assert_eq!(
            lines(RendererKind::Cpu, None, Some(&FrameStats::default())),
            [
                "CPU  n/a FPS / n/a ms",
                "Renderer diagnostics: n/a (CPU viewport)",
            ]
        );
    }

    #[test]
    fn formats_each_measurement_and_distinguishes_unavailable_from_zero() {
        let stats = FrameStats {
            wall_ms: 10.0,
            occupancy_sync_ms: 1.0,
            mesh_sync_ms: 2.0,
            occupancy_upload_ms: 3.0,
            mesh_upload_ms: 4.0,
            encode_ms: 5.0,
            submit_complete_ms: 6.0,
            gpu: Some(GpuTimes {
                main_ms: 7.0,
                resolve_ms: 8.0,
                shadow_ms: Some(0.0),
                total_ms: 9.0,
            }),
            resident_chunks: 11,
            chunks_updated: 12,
            occupied_bricks: 13,
            total_bricks: 14,
            draw_calls: 11,
            shadow_draw_calls: 11,
            vertices: 90,
            triangles: 30,
            vertex_bytes: 1024,
            occupancy_bytes: 0,
            target_bytes: 2 * 1024 * 1024,
            device_allocated_bytes: Some(3 * 1024 * 1024),
            upload_bytes: 16,
            target_size: [1280, 800],
            shadow_path: ShadowPath::Fallback,
        };
        assert_eq!(
            lines(
                RendererKind::Gpu,
                Some(FrameAverage {
                    fps: 50.0,
                    frame_ms: 20.0
                }),
                Some(&stats)
            ),
            [
                "GPU  50.0 FPS / 20.00 ms",
                "CPU wall 10.00 ms",
                "  sync    occ 1.00 / mesh 2.00 ms",
                "  upload  occ 3.00 / mesh 4.00 ms",
                "  encode 5.00 / submit+complete 6.00 ms",
                "GPU main 7.00 / resolve 8.00 ms",
                "  shadow 0.00 ms / total 9.00 ms",
                "Chunks 11 resident / 12 updated",
                "Bricks 13 occupied / 14 total (8³)",
                "Draws 11 main / 1 resolve / 11 shadow",
                "Vertices 90 / triangles 30",
                "Memory vertices 1.0 KiB / occ 0 B",
                "  targets 2.0 MiB / device 3.0 MiB",
                "Uploaded 16 B",
                "Target 1280 × 800 / shadow fallback",
            ]
        );
        let missing = lines(RendererKind::Gpu, None, Some(&FrameStats::default()));
        assert_eq!(missing[5], "GPU main n/a / resolve n/a");
        assert_eq!(missing[6], "  shadow n/a / total n/a");
        assert_eq!(missing[12], "  targets 0 B / device n/a");
        assert_eq!(missing[14], "Target 0 × 0 / shadow exact");
        let no_shadow = FrameStats {
            gpu: Some(GpuTimes::default()),
            ..stats
        };
        assert_eq!(
            lines(RendererKind::Gpu, None, Some(&no_shadow))[6],
            "  shadow n/a / total 0.00 ms"
        );
    }
}
