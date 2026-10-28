//! Sidebar composition and plugin panel selection.
use super::*;

impl EditorView {
    pub(super) fn sidebar(&self, compact: bool, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        Panel::sidebar(t)
            .id("sidebar")
            .child(self.panel_tabs(cx))
            .child(
                div()
                    .id("sidebar-scroll")
                    .when(self.panel != SidebarTab::Session, |d| d.flex().flex_col())
                    .flex_1()
                    .min_h(px(quantum_ui::Space::S0.px()))
                    .overflow_y_scroll()
                    .track_scroll(&self.sidebar_scroll)
                    .child(match self.panel {
                        SidebarTab::Comments => self.comments_panel(cx),
                        SidebarTab::History => self.timeline_panel(cx),
                        SidebarTab::Session => self.agent_feed(cx),
                    })
                    .when(self.panel == SidebarTab::Session, |d| {
                        d.child(self.layers_panel(compact || self.agent.capture.is_some(), cx))
                            .children({
                                let panels: Vec<_> = self
                                    .editor
                                    .plugin_host()
                                    .registry()
                                    .panels()
                                    .map(|(_, p)| p.id.clone())
                                    .collect();
                                panels
                                    .into_iter()
                                    .filter(|id| id == quantum_plugin_palette::PANEL_ID)
                                    .map(|_| {
                                        self.palette_panel(
                                            compact || self.agent.capture.is_some(),
                                            cx,
                                        )
                                    })
                            })
                    }),
            )
    }
}
