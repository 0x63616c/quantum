//! Standalone browser session state. Companion pairing is deferred.
use super::*;
#[derive(Default)]
pub(super) struct Agent {
    pub events: Vec<()>,
    pub open: bool,
    pub clients: usize,
    pub capture: Option<CapturedViews>,
    pub capture_generation: u64,
    pub capturing: bool,
    pub follow: bool,
    pub retired: Vec<Arc<RenderImage>>,
}
pub(super) struct CapturedViews {
    pub views: Vec<(String, PresetView, Arc<RenderImage>)>,
}
impl Agent {
    pub fn new(_: &Editor, _: Option<PathBuf>) -> std::io::Result<Self> {
        Ok(Self::default())
    }
}
impl EditorView {
    pub(super) fn agent_popover(&self, _: &Context<Self>) -> AnyElement {
        div().absolute().top(px(100.0)).right(px(24.0)).child(
            self.floating().w(px(420.0)).p(px(20.0)).flex().flex_col().gap(px(12.0))
                .child(section_title("Build with an agent"))
                .child(div().text_color(self.theme.muted()).child("Connect an agent in the desktop app. Browser companion pairing is coming soon; you can edit and download this project here."))
        ).into_any_element()
    }
    pub(super) fn agent_feed(&self, compact: bool, cx: &Context<Self>) -> Div {
        div()
            .px(px(PANEL_GUTTER))
            .py(px(if compact { 16.0 } else { 20.0 }))
            .child(section_title("Build with an agent"))
            .child(
                div()
                    .mt(px(6.0))
                    .text_color(self.theme.muted())
                    .child("Agent sessions are available in the desktop app."),
            )
            .when(self.editor.engine.history().next().is_some(), |panel| {
                panel.child(self.history_panel(true, cx).h(px(156.0)))
            })
    }
}
