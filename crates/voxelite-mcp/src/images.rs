//! Sequential CPU renders and bounded, labeled multi-angle PNG contact sheets.
use crate::{
    error::{Args, Failure, Result},
    files,
};
use serde_json::{Value, json};
use voxelite_core::Snapshot;
use voxelite_render::{
    AmbientOcclusionStrength, Camera, PresetView, Projection, RenderOptions, View,
};

pub(crate) struct Images {
    pub pngs: Vec<Vec<u8>>,
    pub metadata: Value,
}

pub(crate) fn render(
    s: &Snapshot,
    a: &Args<'_>,
    passes: &voxelite_render::RenderPlan,
) -> Result<Images> {
    render_with_encoder(s, a, passes, |width, height, rgba| {
        Ok(voxelite_export::png_rgba(width, height, rgba)?)
    })
}

pub(crate) fn export(
    s: &Snapshot,
    a: &Args<'_>,
    host: &mut voxelite_plugin_host::Host,
) -> Result<Images> {
    let format = crate::exports::format(host, "png")?;
    let passes = host.render_plan();
    render_with_encoder(s, a, &passes, |width, height, rgba| {
        crate::exports::encode(
            host,
            &format,
            &Pixels {
                snapshot: s,
                width,
                height,
                rgba,
            },
        )
    })
}

struct Pixels<'a> {
    snapshot: &'a Snapshot,
    width: u32,
    height: u32,
    rgba: &'a [u8],
}
impl voxelite_plugin_api::ExportCx for Pixels<'_> {
    fn snapshot(&self) -> &Snapshot {
        self.snapshot
    }
    fn render_view(&self) -> voxelite_plugin_api::ExportResult<voxelite_plugin_api::ExportImage> {
        Ok(voxelite_plugin_api::ExportImage {
            width: self.width,
            height: self.height,
            rgba: self.rgba.to_vec(),
        })
    }
}

fn render_with_encoder(
    s: &Snapshot,
    a: &Args<'_>,
    passes: &voxelite_render::RenderPlan,
    mut encode: impl FnMut(u32, u32, &[u8]) -> Result<Vec<u8>>,
) -> Result<Images> {
    let width: u32 = a.default("width", 512)?;
    let height: u32 = a.default("height", 512)?;
    let mode: String = a.default("image_mode", "contact_sheet".into())?;
    let perspective = a.default("perspective", false)?;
    let background = a.default("background", [0u8; 4])?;
    let ambient_occlusion_strength: AmbientOcclusionStrength = a.default(
        "ambient_occlusion_strength",
        AmbientOcclusionStrength::default(),
    )?;
    let mut names: Vec<String> = a.default(
        "views",
        vec!["iso".into(), "front".into(), "right".into(), "back".into()],
    )?;
    let explicit = if let Some(c) = a.0.get("camera") {
        if a.0.get("views").is_some() || a.0.get("perspective").is_some() {
            return Err(Failure::invalid(
                "camera is mutually exclusive with views and perspective",
            ));
        }
        let c = Args(c);
        let projection: String = c.get("projection")?;
        let projection = match projection.as_str() {
            "orthographic" if c.0.get("fov_y_degrees").is_none() => Projection::Orthographic {
                height: c.get("ortho_height")?,
            },
            "perspective" if c.0.get("ortho_height").is_none() => Projection::Perspective {
                fov_y_degrees: c.get("fov_y_degrees")?,
            },
            _ => {
                return Err(Failure::invalid(
                    "camera requires exactly its projection's height or FOV",
                ));
            }
        };
        let camera = Camera {
            eye: c.get("eye")?,
            target: c.get("target")?,
            up: c.get("up")?,
            projection,
        };
        camera.validate()?;
        names = vec!["camera".into()];
        Some(camera)
    } else {
        None
    };
    if !(1..=1024).contains(&width)
        || !(1..=1024).contains(&height)
        || names.is_empty()
        || names.len() > 6
    {
        return Err(Failure::invalid("render dimensions/views exceed limits"));
    }
    let sheet = mode != "individual";
    let individual = mode != "contact_sheet";
    let columns = if names.len() == 1 { 1 } else { 2 };
    let rows = (names.len() as u32).div_ceil(columns);
    let sw = width * columns;
    let sh = height * rows;
    let pixels = u64::from(width) * u64::from(height) * names.len() as u64
        + if sheet {
            u64::from(sw) * u64::from(sh)
        } else {
            0
        };
    if pixels > 4_194_304 || (sheet && (sw > 2048 || sh > 2048)) {
        return Err(Failure::new(
            "limit_exceeded",
            "render and contact sheet exceed pixel budget; reduce dimensions or views",
        ));
    }
    let mut canvas = if sheet {
        background.repeat((sw * sh) as usize)
    } else {
        Vec::new()
    };
    let mut pngs = Vec::new();
    let mut views = Vec::new();
    let mut output = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let view = if let Some(c) = explicit {
            View::Camera(c)
        } else {
            let direction = match name.as_str() {
                "iso" => PresetView::Iso,
                "front" => PresetView::Front,
                "back" => PresetView::Back,
                "left" => PresetView::Left,
                "right" => PresetView::Right,
                "top" => PresetView::Top,
                _ => return Err(Failure::invalid("unknown view")),
            };
            View::Preset {
                direction,
                perspective,
            }
        };
        let image = voxelite_render::render(
            s,
            &RenderOptions {
                passes: passes.clone(),
                width,
                height,
                view,
                background,
                ambient_occlusion_strength,
                ..Default::default()
            },
        )?;
        let png = encode(width, height, &image.rgba)?;
        let projection = match image.camera.projection {
            Projection::Orthographic { height } => {
                json!({"projection":"orthographic","ortho_height":height})
            }
            Projection::Perspective { fov_y_degrees } => {
                json!({"projection":"perspective","fov_y_degrees":fov_y_degrees})
            }
        };
        let mut camera =
            json!({"eye":image.camera.eye,"target":image.camera.target,"up":image.camera.up});
        camera
            .as_object_mut()
            .unwrap()
            .extend(projection.as_object().unwrap().clone());
        views.push(json!({"view":name,"width":width,"height":height,"camera":camera,"sha256":files::hash(&png),"bytes":png.len()}));
        if sheet {
            let ox = (i as u32 % columns) * width;
            let oy = (i as u32 / columns) * height;
            for y in 0..height as usize {
                let start = (((oy as usize + y) * sw as usize) + ox as usize) * 4;
                canvas[start..start + width as usize * 4].copy_from_slice(
                    &image.rgba[y * width as usize * 4..(y + 1) * width as usize * 4],
                );
            }
            label(&mut canvas, sw, [ox, oy, width, height], name);
        }
        if individual {
            output.push(json!({"view":name,"width":width,"height":height,"sha256":files::hash(&png),"bytes":png.len()}));
            pngs.push(png);
        }
    }
    if sheet {
        let png = encode(sw, sh, &canvas)?;
        output.insert(0,json!({"view":"contact_sheet","width":sw,"height":sh,"sha256":files::hash(&png),"bytes":png.len()}));
        pngs.insert(0, png);
    }
    Ok(Images {
        pngs,
        metadata: json!({"backend":"cpu","bounds":voxelite_render::visible_bounds(s),"views":views,"images":output,"image_mode":mode}),
    })
}
// Minimal embedded 5×7 capitals for view labels. Labels stay inside each tile,
// so tile dimensions and the advertised contact-sheet pixel budget are exact.
fn label(canvas: &mut [u8], stride: u32, tile: [u32; 4], text: &str) {
    let [ox, oy, w, h] = tile;
    if w < 80 || h < 24 {
        return;
    }
    let scale = if w >= 256 { 2 } else { 1 };
    let glyph = |c| -> [u8; 7] {
        match c {
            'A' => [14, 17, 17, 31, 17, 17, 17],
            'B' => [30, 17, 17, 30, 17, 17, 30],
            'C' => [14, 17, 16, 16, 16, 17, 14],
            'E' => [31, 16, 16, 30, 16, 16, 31],
            'F' => [31, 16, 16, 30, 16, 16, 16],
            'G' => [14, 17, 16, 23, 17, 17, 15],
            'H' => [17, 17, 17, 31, 17, 17, 17],
            'I' => [14, 4, 4, 4, 4, 4, 14],
            'K' => [17, 18, 20, 24, 20, 18, 17],
            'L' => [16, 16, 16, 16, 16, 16, 31],
            'M' => [17, 27, 21, 21, 17, 17, 17],
            'N' => [17, 25, 25, 21, 19, 19, 17],
            'O' => [14, 17, 17, 17, 17, 17, 14],
            'P' => [30, 17, 17, 30, 16, 16, 16],
            'R' => [30, 17, 17, 30, 20, 18, 17],
            'S' => [15, 16, 16, 14, 1, 1, 30],
            'T' => [31, 4, 4, 4, 4, 4, 4],
            _ => [0; 7],
        }
    };
    let label_w = text.len() as u32 * 6 * scale + 12;
    for y in 0..(7 * scale + 8) {
        for x in 0..label_w.min(w) {
            let i = (((oy + y) * stride + ox + x) * 4) as usize;
            canvas[i..i + 4].copy_from_slice(&[0, 0, 0, 220]);
        }
    }
    for (n, c) in text.to_ascii_uppercase().chars().enumerate() {
        for (y, bits) in glyph(c).into_iter().enumerate() {
            for x in 0..5 {
                if bits & (1 << (4 - x)) != 0 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = ox + 6 + n as u32 * 6 * scale + x * scale + dx;
                            let py = oy + 4 + y as u32 * scale + dy;
                            if px < ox + w && py < oy + h {
                                let i = ((py * stride + px) * 4) as usize;
                                canvas[i..i + 4].copy_from_slice(&[235, 235, 235, 255]);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_and_export_use_the_same_disabled_lighting_plan() {
        let snapshot = serde_json::from_slice(include_bytes!(
            "../../voxelite-export/tests/golden/scene.json"
        ))
        .unwrap();
        let args = json!({"width":80,"height":64,"views":["iso"]});
        let mut host = crate::exports::host();
        let enabled = render(&snapshot, &Args(&args), &host.render_plan()).unwrap();
        host.disable(voxelite_plugin_shadows_lighting::PLUGIN_ID)
            .unwrap();
        let disabled = render(&snapshot, &Args(&args), &host.render_plan()).unwrap();
        assert_ne!(disabled.pngs, enabled.pngs);
        assert_eq!(
            export(&snapshot, &Args(&args), &mut host).unwrap().pngs,
            disabled.pngs
        );
        host.enable(voxelite_plugin_shadows_lighting::PLUGIN_ID)
            .unwrap();
        assert_eq!(
            render(&snapshot, &Args(&args), &host.render_plan())
                .unwrap()
                .pngs,
            enabled.pngs
        );
    }

    #[test]
    fn contact_sheet_export_characterization() {
        let snapshot = serde_json::from_slice(include_bytes!(
            "../../voxelite-export/tests/golden/scene.json"
        ))
        .unwrap();
        let args = json!({"width":80,"height":64,"views":["iso","front"]});
        let images = render(
            &snapshot,
            &Args(&args),
            &crate::exports::host().render_plan(),
        )
        .unwrap();
        assert_eq!(images.pngs[0], include_bytes!("../tests/contact-sheet.png"));
        let exported = export(&snapshot, &Args(&args), &mut crate::exports::host()).unwrap();
        assert_eq!(exported.pngs, images.pngs);
        assert_eq!(exported.metadata, images.metadata);
    }

    #[test]
    fn ao_override_is_explicit_and_shared_by_render_and_export() {
        let snapshot = serde_json::from_slice(include_bytes!(
            "../../voxelite-export/tests/golden/scene.json"
        ))
        .unwrap();
        let mut args = json!({"width":80,"height":64,"views":["iso"],"image_mode":"individual"});
        let plan = crate::exports::host().render_plan();
        let original = render(&snapshot, &Args(&args), &plan).unwrap();
        args["ambient_occlusion_strength"] = json!(1);
        assert_eq!(original.pngs, render(&snapshot, &Args(&args), &plan).unwrap().pngs);
        for strength in [0.0, 0.5, 2.0] {
            args["ambient_occlusion_strength"] = json!(strength);
            let rendered = render(&snapshot, &Args(&args), &plan).unwrap();
            assert_ne!(original.pngs, rendered.pngs);
            assert_eq!(
                rendered.pngs,
                export(&snapshot, &Args(&args), &mut crate::exports::host())
                    .unwrap()
                    .pngs
            );
        }
        for value in [json!(-0.1), json!(2.1), json!(null), json!("1")] {
            args["ambient_occlusion_strength"] = value;
            assert!(render(&snapshot, &Args(&args), &plan).is_err());
        }
    }
}
