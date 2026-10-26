//! Voxelite native GPUI editor and display-free viewport capture.
mod actions;
mod assets;
mod shortcuts;
mod text_field;
mod theme;
mod ui;
#[cfg(target_os = "macos")]
pub mod updater;
use gpui::{prelude::*, *};
use std::{path::PathBuf, time::Duration};
use theme::Theme;
use voxelite_app::editor::Editor;

struct Options {
    path: Option<PathBuf>,
    theme: Option<Theme>,
    exit_after: Option<u64>,
    viewport_png: Option<PathBuf>,
    session_socket: Option<PathBuf>,
}
fn options() -> anyhow::Result<Options> {
    let mut options = Options {
        path: None,
        theme: None,
        exit_after: None,
        viewport_png: None,
        session_socket: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--session-socket" => {
                options.session_socket = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("missing socket path"))?
                        .into(),
                );
            }
            "--theme" => {
                options.theme = Some(match args.next().as_deref() {
                    Some("dark") => Theme::Dark,
                    Some("light") => Theme::Light,
                    _ => anyhow::bail!("--theme requires dark or light"),
                })
            }
            "--exit-after" | "--screenshot-delay" => {
                let seconds: u64 = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing seconds"))?
                    .parse()?;
                anyhow::ensure!(
                    (1..=3600).contains(&seconds),
                    "exit delay must be 1..=3600 seconds"
                );
                options.exit_after = Some(seconds);
            }
            "--viewport-png" => {
                options.viewport_png = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("missing PNG path"))?
                        .into(),
                )
            }
            "--help" | "-h" => {
                println!(
                    "Voxelite native voxel editor\n\nvoxelite [project.voxl] [--theme dark|light] [--exit-after SECONDS] [--session-socket PATH]\nvoxelite [project.voxl] --viewport-png output.png\n\nNo path opens the bundled autumn-workshop diorama.\n--viewport-png renders without initializing GPUI or a display.\n--session-socket selects the private Unix socket for live MCP attachment.\n--screenshot-delay is an alias for --exit-after (external capture).\nSee crates/voxelite-app/README.md for keyboard and mouse controls."
                );
                std::process::exit(0);
            }
            _ if arg.starts_with('-') => anyhow::bail!("unknown option: {arg}"),
            _ => {
                anyhow::ensure!(options.path.is_none(), "only one project path is accepted");
                options.path = Some(arg.into());
            }
        }
    }
    Ok(options)
}
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let options = options()?;
    let editor = match options.path {
        Some(path) => Editor::open(&path)?,
        None => Editor::example()?,
    };
    if let Some(path) = options.viewport_png {
        let frame = voxelite_render::render(
            editor.snapshot(),
            &voxelite_render::RenderOptions {
                width: 1024,
                height: 900,
                background: if options.theme == Some(Theme::Light) {
                    [255; 4]
                } else {
                    [0, 0, 0, 255]
                },
                ..Default::default()
            },
        )?;
        std::fs::write(
            path,
            voxelite_export::png_rgba(frame.width, frame.height, &frame.rgba)?,
        )?;
        return Ok(());
    }
    gpui_platform::application()
        .with_assets(assets::Assets)
        .run(move |cx| {
            #[cfg(target_os = "macos")]
            updater::init();
            cx.text_system()
                .add_fonts(assets::fonts())
                .expect("bundled fonts");
            cx.set_text_rendering_mode(TextRenderingMode::Grayscale);
            actions::install(cx);
            let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
            if let Err(error) = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: if cfg!(target_os = "macos") {
                        Some(TitlebarOptions {
                            title: Some("Voxelite".into()),
                            appears_transparent: true,
                            traffic_light_position: Some(point(px(20.0), px(24.0))),
                        })
                    } else {
                        None
                    },
                    app_owns_titlebar_drag: cfg!(target_os = "macos"),
                    window_min_size: Some(size(px(1024.0), px(640.0))),
                    app_id: Some("dev.voxelite.editor".into()),
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(|cx| {
                        ui::EditorView::new(
                            editor,
                            options.theme,
                            options.session_socket,
                            window,
                            cx,
                        )
                    })
                },
            ) {
                eprintln!("Cannot open Voxelite: {error}");
                cx.quit();
            }
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            if let Some(seconds) = options.exit_after {
                let timer = cx.background_executor().timer(Duration::from_secs(seconds));
                cx.spawn(async move |cx| {
                    timer.await;
                    cx.update(|cx| cx.quit());
                })
                .detach();
            }
            cx.activate(true);
        });
    Ok(())
}
