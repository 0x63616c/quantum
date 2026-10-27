//! Controlled numeric presentation backed by the shared TextField entity.
use crate::{TextField, TextFieldEvent, Theme, editing::NumericPolicy};
use gpui::{prelude::*, *};
use std::rc::Rc;
type Change = Rc<dyn Fn(&f64, &mut Window, &mut App)>;
struct NumericState {
    field: Entity<TextField>,
    change: Change,
    policy: NumericPolicy,
    value: f64,
}
/// A numeric field with persistent selection, local history and Option-drag scrub.
#[derive(IntoElement)]
pub struct NumericField {
    id: ElementId,
    label: SharedString,
    theme: Theme,
    policy: NumericPolicy,
    value: f64,
    change: Change,
    scope: Option<FocusHandle>,
    return_focus: Option<FocusHandle>,
}
impl NumericField {
    /// Create a controlled number box. IDs must be stable across parent renders.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        theme: Theme,
        policy: NumericPolicy,
        value: f64,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            theme,
            policy,
            value,
            change: Rc::new(|_, _, _| {}),
            scope: None,
            return_focus: None,
        }
    }
    /// Receive valid numeric draft changes, including stepping and scrubbing.
    pub fn on_change(mut self, change: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.change = Rc::new(change);
        self
    }
    /// Set the modal tab scope and Escape focus destination.
    pub fn focus_policy(mut self, scope: Option<FocusHandle>, return_focus: FocusHandle) -> Self {
        self.scope = scope;
        self.return_focus = Some(return_focus);
        self
    }
}
impl RenderOnce for NumericField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id, cx, |window, cx| {
            let field = TextField::new(self.label, cx);
            field.update(cx, |field, cx| field.number(self.policy, self.value, cx));
            cx.subscribe_in(
                &field,
                window,
                |this: &mut NumericState, field, event: &TextFieldEvent, window, cx| {
                    if matches!(event, TextFieldEvent::Changed | TextFieldEvent::Committed)
                        && let Some(value) = this.policy.parse(field.read(cx).text())
                    {
                        (this.change)(&value, window, cx);
                    }
                },
            )
            .detach();
            NumericState {
                field,
                change: self.change.clone(),
                policy: self.policy,
                value: self.policy.clamp(self.value),
            }
        });
        state.update(cx, |state, cx| {
            state.change = self.change;
            let policy_changed = state.policy != self.policy;
            state.policy = self.policy;
            let value = self.policy.clamp(self.value);
            let value_changed = state.value != value;
            state.value = value;
            state.field.update(cx, |field, cx| {
                field.appearance(self.theme, cx);
                if let Some(return_focus) = self.return_focus {
                    field.focus_policy(self.scope, return_focus);
                }
                if policy_changed {
                    field.number(self.policy, self.value, cx);
                } else if value_changed && self.policy.parse(field.text()) != Some(value) {
                    // Echoes of valid draft changes must not erase history or
                    // replace an invalid draft before its blur policy runs.
                    field.set(self.policy.format(value), cx);
                }
            });
            state.field.clone()
        })
    }
}
