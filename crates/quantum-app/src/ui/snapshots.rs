//! Harness-only fixture admission; all painting uses the production views.
use super::*;

pub(crate) fn open(
    cx: &mut HeadlessAppContext,
    state: crate::ui_snapshots::Screen,
    theme: Theme,
    size: Size<Pixels>,
    home: PathBuf,
) -> anyhow::Result<AnyWindowHandle> {
    cx.update(|cx| welcome::init(Some(theme), None, Ok(Default::default()), cx));
    if matches!(state, crate::ui_snapshots::Screen::Welcome) {
        return cx.update(|cx| {
            let handle = welcome::show(cx)?;
            welcome::prepare_snapshot(handle, home, cx)?;
            Ok(handle.into())
        });
    }
    let editor = Editor::example()?;
    let handle = cx.open_window(size, |window, cx| {
        cx.new(|cx| {
            let mut view = EditorView::new(editor, Some(theme), None, window, cx);
            view.reduced_motion = true;
            view
        })
    })?;
    let settings = matches!(
        state,
        crate::ui_snapshots::Screen::Editor(Some(launch::LaunchState::Settings(_)))
    );
    cx.update(|cx| {
        handle.update(cx, |view, window, cx| -> anyhow::Result<()> {
            if let crate::ui_snapshots::Screen::Editor(Some(state)) = state {
                view.show_capture(state, window, cx)?;
            }
            Ok(())
        })?
    })?;
    // Settings opens in a deferred effect after the owner update has ended.
    cx.run_until_parked();
    cx.update(|cx| {
        handle.update(cx, |view, _, _| {
            if settings {
                view.settings_window
                    .map(Into::into)
                    .ok_or_else(|| anyhow::anyhow!("Settings fixture did not open its window"))
            } else {
                Ok(handle.into())
            }
        })
    })?
}

pub(crate) fn check_ready(
    cx: &mut HeadlessAppContext,
    target: AnyWindowHandle,
) -> anyhow::Result<()> {
    cx.update_window(target, |view, _, cx| -> anyhow::Result<()> {
        if let Ok(editor) = view.clone().downcast::<EditorView>() {
            let view = editor.read(cx);
            anyhow::ensure!(!view.error, "editor fixture reported an error");
            anyhow::ensure!(
                view.viewport.has_frame() && !view.viewport.rendering(),
                "viewport did not finish rendering"
            );
        } else if let Ok(welcome) = view.clone().downcast::<welcome::WelcomeView>() {
            welcome::snapshot_ready(welcome, cx)?;
        } else {
            anyhow::ensure!(
                view.downcast::<settings::SettingsView>().is_ok(),
                "unexpected snapshot root view"
            );
            anyhow::ensure!(
                cx.global::<settings::SettingsState>().error.is_none(),
                "Settings fixture reported an error"
            );
        }
        Ok(())
    })?
}
