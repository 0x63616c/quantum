//! Settings window lifecycle and editor preference synchronization.
use super::*;
use quantum_ui::Space;

impl EditorView {
    pub(in crate::ui) fn open_settings(
        &mut self,
        section: Section,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let owner = cx.weak_entity();
        let owner_window = window.window_handle();
        {
            let existing = self.settings_window;
            // open_window paints its first frame immediately. Defer until this
            // EditorView update ends: settings reads the owner's live registry.
            cx.defer(move |cx| {
                if let Some(handle) = existing
                    && handle
                        .update(cx, |view, window, cx| {
                            view.navigate(section, cx);
                            window.activate_window();
                            cx.notify();
                        })
                        .is_ok()
                {
                    return;
                }
                let bounds = Bounds::centered(
                    None,
                    size(px(metrics::SETTINGS_WIDTH), px(metrics::SETTINGS_HEIGHT)),
                    cx,
                );
                let view_owner = owner.clone();
                let result = cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        app_owns_titlebar_drag: cfg!(target_os = "macos"),
                        window_min_size: Some(size(
                            px(metrics::SETTINGS_MIN_WIDTH),
                            px(metrics::SETTINGS_MIN_HEIGHT),
                        )),
                        titlebar: Some(TitlebarOptions {
                            title: Some("Quantum Settings".into()),
                            appears_transparent: true,
                            traffic_light_position: Some(point(
                                px(Space::S5.px()),
                                px(metrics::TRAFFIC_TOP),
                            )),
                        }),
                        window_background: if cfg!(target_os = "macos") {
                            WindowBackgroundAppearance::Blurred
                        } else {
                            WindowBackgroundAppearance::Opaque
                        },
                        app_id: Some(quantum_app::APP_ID.into()),
                        ..Default::default()
                    },
                    |window, cx| {
                        cx.new(|cx| {
                            SettingsView::new(section, view_owner, owner_window, window, cx)
                        })
                    },
                );
                let _ = owner.update(cx, |this, cx| {
                    match result {
                        Ok(handle) => this.settings_window = Some(handle),
                        Err(error) => {
                            this.status = format!("Could not open settings: {error}").into();
                            this.error = true;
                        }
                    }
                    cx.notify();
                });
            });
        }
        cx.notify();
    }
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: declared boolean grid setting"
    )]
    pub(in crate::ui) fn sync_preferences(&mut self, window: &Window, cx: &mut Context<Self>) {
        let next = cx.global::<SettingsState>().values.clone();
        let next_theme = theme(next.appearance, window);
        let rerender = self.preferences.antialiasing != next.antialiasing
            || self.preferences.viewport_ambient_occlusion() != next.viewport_ambient_occlusion()
            || (self.viewport.gpu_enabled()
                && (self.theme != next_theme
                    || self.preferences.grid != next.grid
                    || self.preferences.show_debug_overlay != next.show_debug_overlay));
        self.theme = next_theme;
        self.editor
            .set_grid(next.grid)
            .expect("declared boolean grid setting");
        if self.preferences.show_debug_overlay != next.show_debug_overlay {
            self.viewport.clear_diagnostics();
        }
        self.preferences = next;
        if rerender {
            self.request_render(cx);
        }
        cx.notify();
    }
}
