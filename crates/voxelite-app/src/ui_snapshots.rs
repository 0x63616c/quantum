//! Opt-in real GPUI pixel capture. Invoked only by `cargo xtask ui-snapshots`.
use gpui::*;
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Copy)]
pub(crate) enum Screen {
    Welcome,
    Editor(Option<crate::ui::launch::LaunchState>),
}

fn parse(name: &str) -> anyhow::Result<(Screen, crate::theme::Theme)> {
    use crate::theme::Theme;
    let (state, theme) = name
        .strip_suffix("-dark")
        .map(|s| (s, Theme::Dark))
        .or_else(|| name.strip_suffix("-light").map(|s| (s, Theme::Light)))
        .unwrap_or((name, Theme::Dark));
    const BASE: &[(&str, Screen)] = &[
        ("welcome", Screen::Welcome),
        ("welcome-recents", Screen::Welcome),
        ("editor", Screen::Editor(None)),
    ];
    let screen = match BASE.iter().find(|(key, _)| *key == state) {
        Some((_, screen)) => *screen,
        None => Screen::Editor(Some(crate::ui::launch::LaunchState::parse(state)?)),
    };
    Ok((screen, theme))
}

pub(crate) fn run() -> anyhow::Result<()> {
    anyhow::ensure!(
        cfg!(target_os = "linux"),
        "UI goldens require Linux lavapipe"
    );
    anyhow::ensure!(
        std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none(),
        "UI snapshots must run without a display server"
    );
    let args: Vec<_> = std::env::args().skip(2).collect();
    anyhow::ensure!(
        args.len() == 5,
        "expected NAME WIDTH HEIGHT OUTPUT FIXTURE_HOME"
    );
    let width: u32 = args[1].parse()?;
    let height: u32 = args[2].parse()?;
    anyhow::ensure!(
        (1..=2048).contains(&width) && (1..=2048).contains(&height),
        "invalid logical capture dimensions"
    );
    let mut cx = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(crate::assets::Assets),
        gpui_platform::current_headless_renderer,
    );
    cx.update(|cx| -> anyhow::Result<()> {
        cx.text_system().add_fonts(crate::assets::fonts())?;
        cx.set_text_rendering_mode(TextRenderingMode::Grayscale);
        crate::actions::install(cx);
        Ok(())
    })?;
    let (screen, theme) = parse(&args[0])?;
    let window = crate::ui::snapshots::open(
        &mut cx,
        screen,
        theme,
        size(px(width as f32), px(height as f32)),
        PathBuf::from(&args[4]),
    )?;
    // The deterministic dispatcher drains real CPU rendering and asset decoding.
    // Draw again after each drain so newly loaded assets reach the GPUI scene.
    for _ in 0..4 {
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })?;
    }
    crate::ui::snapshots::check_ready(&mut cx, window)?;
    let image = cx.capture_screenshot(window)?;
    // TestPlatform has a fixed 2x scale. Never resample captured pixels.
    anyhow::ensure!(
        image.dimensions() == (width * 2, height * 2),
        "wrong capture dimensions"
    );
    let file = std::io::BufWriter::new(std::fs::File::create(&args[3])?);
    let mut encoder = png::Encoder::new(file, image.width(), image.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    encoder.write_header()?.write_image_data(image.as_raw())?;
    Ok(())
}
