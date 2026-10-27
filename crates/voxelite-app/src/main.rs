//! Voxelite native GPUI editor and display-free viewport capture.
mod actions;
mod assets;
mod shortcuts;
mod text_field;
mod theme;
mod ui;
#[cfg(target_os = "macos")]
pub mod updater;
use gpui::*;
use std::{path::PathBuf, time::Duration};
use theme::Theme;
use voxelite_app::editor::Editor;

struct Options {
    path: Option<PathBuf>,
    example: bool,
    theme: Option<Theme>,
    exit_after: Option<u64>,
    viewport_png: Option<PathBuf>,
    session_socket: Option<PathBuf>,
}
fn options(args: impl IntoIterator<Item = String>) -> anyhow::Result<Options> {
    let mut options = Options {
        path: None,
        example: false,
        theme: None,
        exit_after: None,
        viewport_png: None,
        session_socket: None,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--example" => options.example = true,
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
                    "Quantum native voxel editor\n\nvoxelite [project.voxl] [--example] [--theme dark|light] [--exit-after SECONDS] [--session-socket PATH]\nvoxelite [project.voxl] --viewport-png output.png\n\nNo path opens the welcome window.\n--example opens the bundled autumn-workshop diorama for exploration and captures.\n--viewport-png without a path renders that diorama.\n--viewport-png renders without initializing GPUI or a display.\n--session-socket selects the private Unix socket for live MCP attachment.\n--screenshot-delay is an alias for --exit-after (external capture).\nSee crates/voxelite-app/README.md for keyboard and mouse controls."
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
    anyhow::ensure!(
        !options.example || options.path.is_none(),
        "--example cannot be combined with a project path"
    );
    Ok(options)
}
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let options = options(std::env::args().skip(1))?;
    if let Some(path) = options.viewport_png {
        let editor = match options.path.as_ref() {
            Some(path) => Editor::open(path)?,
            None => Editor::example()?,
        };
        let format = editor.export_format("png")?;
        let job = editor.prepare_export(&format.id)?;
        let options = voxelite_render::RenderOptions {
            width: 1024,
            height: 900,
            background: if options.theme == Some(Theme::Light) {
                [255; 4]
            } else {
                [0, 0, 0, 255]
            },
            ..Default::default()
        };
        let bytes = editor.finish_export(job.run(&voxelite_plugin_host::SnapshotExport {
            snapshot: editor.snapshot(),
            render_view: &voxelite_app::editor::render_view(&options),
        }))?;
        std::fs::write(path, bytes)?;
        return Ok(());
    }
    let editor = if options.example {
        Some(Editor::example()?)
    } else {
        options
            .path
            .as_ref()
            .map(|path| Editor::open(path))
            .transpose()?
    };
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
            ui::welcome::init(options.theme, options.session_socket, cx);
            let result = if let Some(editor) = editor {
                ui::welcome::open_editor(editor, cx)
            } else {
                ui::welcome::show(cx).map(|_| ())
            };
            if let Err(error) = result {
                eprintln!("Cannot open Quantum: {error}");
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

#[cfg(test)]
mod tests {
    use super::{Options, Theme, options};
    use std::path::PathBuf;

    fn parse(args: &[&str]) -> anyhow::Result<Options> {
        options(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn startup_distinguishes_welcome_path_and_explicit_capture_fixture() {
        let welcome = parse(&[]).unwrap();
        assert!(welcome.path.is_none() && !welcome.example);
        let path = parse(&["scene.voxl"]).unwrap();
        assert_eq!(path.path, Some(PathBuf::from("scene.voxl")));
        assert!(!path.example);
        let example = parse(&["--example", "--theme", "dark"]).unwrap();
        assert!(example.example && example.path.is_none());
        assert_eq!(example.theme, Some(Theme::Dark));
        assert!(parse(&["--example", "scene.voxl"]).is_err());
        assert!(parse(&["scene.voxl", "other.voxl"]).is_err());
    }

    #[test]
    fn display_free_capture_keeps_its_optional_path() {
        let capture = parse(&["--viewport-png", "preview.png"]).unwrap();
        assert_eq!(capture.viewport_png, Some(PathBuf::from("preview.png")));
        assert!(capture.path.is_none());
        let capture = parse(&["scene.voxl", "--viewport-png", "preview.png"]).unwrap();
        assert_eq!(capture.path, Some(PathBuf::from("scene.voxl")));
    }
}
