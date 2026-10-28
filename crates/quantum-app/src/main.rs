//! Quantum native GPUI editor and display-free viewport capture.
mod actions;
mod assets;
mod shortcuts;
mod text_field;
mod theme;
mod ui;
#[cfg(target_os = "macos")]
pub mod updater;
use gpui::*;
use quantum_app::editor::Editor;
use std::{path::PathBuf, time::Duration};
use theme::Theme;

struct Options {
    path: Option<PathBuf>,
    example: bool,
    theme: Option<Theme>,
    exit_after: Option<u64>,
    viewport_png: Option<PathBuf>,
    session_socket: Option<PathBuf>,
    show: Option<ui::launch::LaunchState>,
}
#[expect(clippy::print_stdout, reason = "CLI output")]
fn options(args: impl IntoIterator<Item = String>) -> anyhow::Result<Options> {
    let mut options = Options {
        path: None,
        example: false,
        theme: None,
        exit_after: None,
        viewport_png: None,
        session_socket: None,
        show: None,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--example" => options.example = true,
            "--show" => {
                anyhow::ensure!(options.show.is_none(), "--show may only be specified once");
                options.show = Some(ui::launch::LaunchState::parse(&args.next().ok_or_else(
                    || {
                        anyhow::anyhow!(
                            "--show requires a capture state; see crates/quantum-app/README.md"
                        )
                    },
                )?)?);
            }
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
                    "Quantum native voxel editor\n\nquantum [project.voxl] [--example] [--theme dark|light] [--exit-after SECONDS] [--session-socket PATH]\nquantum [project.voxl] --viewport-png output.png\n\nNo path opens Welcome, or a blank editor when Show at launch is off.\n--example opens the bundled autumn-workshop diorama for exploration and captures.\n--viewport-png without a path renders that diorama.\n--viewport-png renders without initializing GPUI or a display.\n--session-socket selects the private Unix socket for live MCP attachment.\n--screenshot-delay is an alias for --exit-after (external capture).\nSee crates/quantum-app/README.md for keyboard and mouse controls."
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
    anyhow::ensure!(
        options.show.is_none() || (options.path.is_none() && options.viewport_png.is_none()),
        "--show uses the bundled example and cannot be combined with a project path or --viewport-png"
    );
    Ok(options)
}
#[expect(clippy::expect_used, reason = "internal invariant: bundled fonts")]
fn main() -> anyhow::Result<()> {
    quantum_app::local_settings::initialize_data_dir()?;
    install_diagnostics()?;
    let options = options(std::env::args().skip(1))?;
    if let Some(path) = options.viewport_png {
        let editor = match options.path.as_ref() {
            Some(path) => Editor::open(path)?,
            None => Editor::example()?,
        };
        let format = editor.export_format("png")?;
        let job = editor.prepare_export(&format.id)?;
        let options = quantum_render::RenderOptions {
            passes: editor.render_plan(),
            width: 1024,
            height: 900,
            background: if options.theme == Some(Theme::Light) {
                [255; 4]
            } else {
                [0, 0, 0, 255]
            },
            ..Default::default()
        };
        let bytes = editor.finish_export(job.run(&quantum_plugin_host::SnapshotExport {
            snapshot: editor.snapshot(),
            render_view: &quantum_app::editor::render_view(&options),
        }))?;
        quantum_format::storage::safe_write(path, bytes)?;
        return Ok(());
    }
    let (editor, open_error) = if options.example || options.show.is_some() {
        (Some(Editor::example()?), None)
    } else {
        quantum_app::startup::open(options.path.as_deref())
    };
    // Load local preferences before starting GPUI's event loop.
    let preferences = quantum_app::preferences::Preferences::load(
        &quantum_app::local_settings::data_dir().join("settings.json"),
    );
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
            ui::welcome::init(options.theme, options.session_socket, preferences, cx);
            let result = if let Some(editor) = editor {
                ui::welcome::open_editor(editor, cx).and_then(|()| {
                    if let Some(state) = options.show {
                        ui::welcome::show_capture(state, cx)?;
                    }
                    Ok(())
                })
            } else if open_error.is_some() {
                // A failed launch path always lands on Welcome so the error is
                // visible, even when "Show at launch" is off.
                ui::welcome::show(cx).and_then(|handle| {
                    handle.update(cx, |view, _, cx| view.show_error(open_error, cx))?;
                    Ok(())
                })
            } else {
                ui::welcome::startup(cx)
            };
            if let Err(error) = result {
                eprintln!("Cannot open Quantum: {error}");
                cx.quit();
            }
            if options.show.is_none() {
                ui::feedback::offer_pending(cx);
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

fn install_diagnostics() -> anyhow::Result<()> {
    use std::io::Write;
    let tail = quantum_crash::LogTail::new(20);
    let logs = tail.clone();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format(move |buffer, record| {
            let line = format!("{}: {}", record.level(), record.args());
            logs.push(&line);
            writeln!(buffer, "{line}")
        })
        .init();
    let redactor = quantum_app::diagnostics::redactor().map_err(anyhow::Error::new)?;
    if quantum_crash::install_panic_hook(quantum_crash::HookConfig {
        directory: quantum_app::diagnostics::directory(),
        environment: quantum_app::diagnostics::environment(),
        redactor,
        log_tail: tail,
    })
    .is_err()
    {
        eprintln!("Could not enable local crash reports; the editor will still start.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Options, Theme, options, ui};
    use std::path::PathBuf;

    fn parse(args: &[&str]) -> anyhow::Result<Options> {
        options(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn shell_panic_hook_persists_redacted_report_for_next_launch() {
        const CHILD: &str = "QUANTUM_SHELL_PANIC_TEST";
        if std::env::var_os(CHILD).is_some() {
            super::install_diagnostics().unwrap();
            panic!("forced panic /Users/private/model.voxl private@example.com");
        }
        let dir = tempfile::tempdir().unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::shell_panic_hook_persists_redacted_report_for_next_launch",
            ])
            .env(CHILD, "1")
            .env("QUANTUM_DATA_DIR", dir.path())
            .output()
            .unwrap();
        assert!(!output.status.success());
        let pending = quantum_crash::list_pending(&dir.path().join("crashes")).unwrap();
        assert_eq!(pending.len(), 1);
        let redactor = quantum_app::diagnostics::redactor().unwrap();
        let report = quantum_crash::load(&pending[0], &redactor).unwrap();
        let quantum_crash::Report::Crash(crash) = report else {
            panic!("expected crash report")
        };
        assert!(!crash.panic_message.contains("/Users/private"));
        assert!(!crash.panic_message.contains("private@example.com"));
        assert!(!crash.backtrace.is_empty());
        assert_eq!(crash.environment.app_version, env!("CARGO_PKG_VERSION"));
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
    fn capture_states_are_explicit_and_reject_ambiguous_launches() {
        assert!(parse(&[]).unwrap().show.is_none());
        for (name, state) in ui::launch::LaunchState::ALL {
            assert_eq!(
                parse(&["--show", name, "--theme", "light"]).unwrap().show,
                Some(*state)
            );
        }
        for args in [
            vec!["--show"],
            vec!["--show", "unknown"],
            vec!["--show", "dialogs-about", "scene.voxl"],
            vec!["--show", "dialogs-about", "--viewport-png", "preview.png"],
            vec!["--show", "dialogs-about", "--show", "panels-history"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
        assert!(
            parse(&["--show", "unknown"])
                .err()
                .unwrap()
                .to_string()
                .contains("unknown --show state")
        );
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
