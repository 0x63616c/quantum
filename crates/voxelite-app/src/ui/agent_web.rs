//! Standalone browser session state. Companion pairing is deferred.
use super::*;
#[derive(Default)]
pub(super) struct Agent {
    pub events: Vec<()>,
    pub open: bool,
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
        div().absolute().top(px(metrics::AGENT_POPOVER_TOP)).right(px(space::XXL)).child(
            self.floating().w(px(metrics::WEB_AGENT_POPOVER_WIDTH)).p(px(space::PANEL)).flex().flex_col().gap(px(space::LG))
                .child(section_title("Build with an agent"))
                .child(div().text_color(self.theme.muted()).child("Connect an agent in the desktop app. Browser companion pairing is coming soon; you can edit and download this project here."))
        ).into_any_element()
    }
    pub(super) fn agent_feed(&self, compact: bool, cx: &Context<Self>) -> Div {
        Panel::section(self.theme)
            .px(px(PANEL_GUTTER))
            .py(px(if compact { space::XL } else { space::PANEL }))
            .child(section_title("Build with an agent"))
            .child(
                div()
                    .mt(px(space::SM))
                    .text_color(self.theme.muted())
                    .child("Agent sessions are available in the desktop app."),
            )
            .when(self.editor.engine.history().next().is_some(), |panel| {
                panel.child(
                    self.history_panel(true, cx)
                        .h(px(metrics::AGENT_HISTORY_HEIGHT)),
                )
            })
    }
}
