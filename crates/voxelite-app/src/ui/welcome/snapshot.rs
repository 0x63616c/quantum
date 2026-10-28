//! Fixed clock and home for the opt-in golden harness, never normal startup.
use super::*;
use std::time::{Duration, SystemTime};

struct FixtureHome(PathBuf);
impl Global for FixtureHome {}

pub(crate) fn prepare_snapshot(
    handle: WindowHandle<WelcomeView>,
    home: PathBuf,
    cx: &mut App,
) -> anyhow::Result<()> {
    cx.set_global(FixtureHome(home));
    handle.update(cx, |view, _, _| view.reduced_motion = true)?;
    Ok(())
}

pub(super) fn home(cx: &App) -> Option<PathBuf> {
    cx.try_global::<FixtureHome>().map(|home| home.0.clone())
}

pub(super) fn clock(cx: &App) -> Option<SystemTime> {
    cx.try_global::<FixtureHome>()
        .map(|_| SystemTime::UNIX_EPOCH + Duration::from_secs(1_782_048_000))
}

pub(crate) fn snapshot_ready(view: Entity<WelcomeView>, cx: &App) -> anyhow::Result<()> {
    let view = view.read(cx);
    anyhow::ensure!(view.error.is_none(), "Welcome fixture reported an error");
    anyhow::ensure!(
        view.samples.len() == Sample::ALL.len(),
        "sample previews did not load"
    );
    let state = cx.global::<Launcher>();
    anyhow::ensure!(state.error.is_none(), "recent fixtures did not load");
    anyhow::ensure!(
        state
            .projects
            .iter()
            .all(|p| view.previews.get(&p.path).is_some_and(Option::is_some)),
        "recent previews did not load"
    );
    Ok(())
}
