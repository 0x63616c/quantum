//! Track field ownership without consulting GPUI's previous-frame dispatch tree.
use super::TextField;
use gpui::{App, Context, EntityId, FocusHandle, Global, Window};
use std::collections::HashMap;

#[derive(Default)]
struct Fields(HashMap<EntityId, FocusHandle>);
impl Global for Fields {}

impl TextField {
    pub(super) fn register_focus(cx: &mut Context<Self>) -> FocusHandle {
        // GPUI uses the tracked handle's tab policy, not the Div's flags.
        let focus = cx.focus_handle().tab_stop(true);
        let id = cx.entity_id();
        cx.default_global::<Fields>().0.insert(id, focus.clone());
        cx.on_release(move |_, cx| {
            if cx.has_global::<Fields>() {
                cx.global_mut::<Fields>().0.remove(&id);
            }
        })
        .detach();
        focus
    }

    /// Whether a live shared field owns keyboard focus in this window.
    ///
    /// Safe before and during the first draw: focus handles exist before the
    /// dispatch tree does. Includes fields created by controlled numeric inputs.
    /// Released fields are removed even if their old handle is still focused.
    pub fn any_focused(window: &Window, cx: &App) -> bool {
        cx.try_global::<Fields>()
            .is_some_and(|fields| fields.0.values().any(|focus| focus.is_focused(window)))
    }
}
