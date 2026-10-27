//! Small native single-line field with clipboard, selection, and IME support.
use crate::{actions::ClipboardOp, shortcuts::ShortcutPlatform, theme::Theme};
use gpui::{prelude::*, *};
use std::ops::Range;

pub struct TextField {
    pub text: String,
    focus: FocusHandle,
    selection: Range<usize>,
    marked: Option<Range<usize>>,
    reversed: bool,
    scroll_x: Pixels,
    drag_anchor: Option<usize>,
    placeholder: &'static str,
    theme: Theme,
    embedded: bool,
    layout: Option<ShapedLine>,
    bounds: Bounds<Pixels>,
}
impl TextField {
    pub fn new(placeholder: &'static str, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            text: String::new(),
            focus: cx.focus_handle(),
            selection: 0..0,
            marked: None,
            reversed: false,
            scroll_x: px(0.0),
            drag_anchor: None,
            placeholder,
            theme: Theme::Dark,
            embedded: false,
            layout: None,
            bounds: Bounds::default(),
        })
    }
    pub fn set(&mut self, text: String, cx: &mut Context<Self>) {
        self.selection = text.len()..text.len();
        self.text = text;
        self.marked = None;
        self.reversed = false;
        cx.notify();
    }
    pub fn appearance(&mut self, theme: Theme, cx: &mut Context<Self>) {
        if self.theme != theme {
            self.theme = theme;
            cx.notify();
        }
    }
    /// Let a surrounding search control provide the background and focus border.
    pub fn embed(&mut self) {
        self.embedded = true;
    }
    pub fn focused(&self, window: &Window) -> bool {
        self.focus.is_focused(window)
    }
    fn byte(&self, offset: usize) -> usize {
        let mut utf16 = 0;
        for (byte, c) in self.text.char_indices() {
            if utf16 >= offset {
                return byte;
            }
            utf16 += c.len_utf16();
        }
        self.text.len()
    }
    fn range_from_utf16(&self, r: Range<usize>) -> Range<usize> {
        self.byte(r.start.min(r.end))..self.byte(r.start.max(r.end))
    }
    fn to_utf16(&self, r: Range<usize>) -> Range<usize> {
        self.text[..r.start].encode_utf16().count()..self.text[..r.end].encode_utf16().count()
    }
    fn replace(&mut self, range: Option<Range<usize>>, text: &str, cx: &mut Context<Self>) -> bool {
        let r = range
            .map(|r| self.range_from_utf16(r))
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone());
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if self.text.len() - r.len() + text.len() > 8192 {
            return false;
        }
        self.text.replace_range(r.clone(), &text);
        self.selection = r.start + text.len()..r.start + text.len();
        self.marked = None;
        self.reversed = false;
        cx.notify();
        true
    }
    fn clipboard(&mut self, op: ClipboardOp, cx: &mut Context<Self>) {
        match op {
            ClipboardOp::SelectAll => {
                self.selection = 0..self.text.len();
                self.reversed = false;
            }
            ClipboardOp::Copy | ClipboardOp::Cut => {
                if !self.selection.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        self.text[self.selection.clone()].into(),
                    ));
                    if op == ClipboardOp::Cut {
                        self.replace(None, "", cx);
                    }
                }
            }
            ClipboardOp::Paste => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.replace(None, &text, cx);
                }
            }
        }
        cx.notify();
    }
    fn key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = e.keystroke.key.as_str();
        let cmd = ShortcutPlatform::current().is_primary(e.keystroke.modifiers);
        let handled = match key {
            "a" | "c" | "x" | "v" if cmd => {
                self.clipboard(
                    match key {
                        "a" => ClipboardOp::SelectAll,
                        "c" => ClipboardOp::Copy,
                        "x" => ClipboardOp::Cut,
                        _ => ClipboardOp::Paste,
                    },
                    cx,
                );
                true
            }
            "backspace" | "delete" => {
                if self.selection.is_empty() {
                    if key == "backspace" {
                        self.selection.start = self.text[..self.selection.start]
                            .char_indices()
                            .next_back()
                            .map_or(0, |(i, _)| i);
                    } else {
                        self.selection.end += self.text[self.selection.end..]
                            .chars()
                            .next()
                            .map_or(0, char::len_utf8);
                    }
                }
                self.replace(None, "", cx);
                true
            }
            "left" | "right" | "home" | "end" => {
                (self.selection, self.reversed) = move_selection(
                    &self.text,
                    self.selection.clone(),
                    self.reversed,
                    key,
                    e.keystroke.modifiers.shift,
                );
                self.marked = None;
                true
            }
            "enter" => true,
            _ => false,
        };
        if handled {
            cx.stop_propagation();
            cx.notify();
        }
    }
}
impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        r: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.range_from_utf16(r);
        *actual = Some(self.to_utf16(r.clone()));
        Some(self.text[r].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.to_utf16(self.selection.clone()),
            reversed: self.reversed,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.clone().map(|r| self.to_utf16(r))
    }
    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked = None;
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace(r, text, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let start = r
            .clone()
            .map(|r| self.range_from_utf16(r).start)
            .or_else(|| self.marked.as_ref().map(|r| r.start))
            .unwrap_or(self.selection.start);
        if !self.replace(r, text, cx) {
            return;
        }
        let end = self.selection.end;
        self.marked = Some(start..end);
        if let Some(r) = selected {
            let base = self.to_utf16(start..start).start;
            let selection =
                self.range_from_utf16(base.saturating_add(r.start)..base.saturating_add(r.end));
            self.selection = selection.start.clamp(start, end)..selection.end.clamp(start, end);
        }
    }
    fn bounds_for_range(
        &mut self,
        r: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let r = self.range_from_utf16(r);
        let l = self.layout.as_ref()?;
        Some(Bounds::from_corners(
            point(
                self.bounds.left() - self.scroll_x + l.x_for_index(r.start),
                self.bounds.top(),
            ),
            point(
                self.bounds.left() - self.scroll_x + l.x_for_index(r.end),
                self.bounds.bottom(),
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let byte = self
            .layout
            .as_ref()?
            .closest_index_for_x(p.x - self.bounds.left() + self.scroll_x);
        Some(self.to_utf16(byte..byte).start)
    }
}
impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let entity = cx.entity();
        let focus = self.focus.clone();
        let focused = self.focused(window);
        div()
            .id("text-field")
            .tab_index(0)
            .track_focus(&self.focus)
            .key_context("TextField")
            .h(px(38.0))
            .w_full()
            .px(px(10.0))
            .py(px(9.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(if focused { t.accent() } else { t.line() })
            .bg(t.background())
            .when(self.embedded, |d| {
                d.h(px(30.0))
                    .px(px(0.0))
                    .py(px(5.0))
                    .border_0()
                    .bg(transparent_black())
            })
            .overflow_hidden()
            .cursor_text()
            .on_key_down(cx.listener(Self::key))
            .on_action(cx.listener(|this, op: &ClipboardOp, _, cx| {
                this.clipboard(*op, cx);
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    let p = this
                        .layout
                        .as_ref()
                        .map_or(this.text.len(), |l| {
                            l.closest_index_for_x(e.position.x - this.bounds.left() + this.scroll_x)
                        })
                        .min(this.text.len());
                    let anchor = if e.modifiers.shift {
                        if this.reversed {
                            this.selection.end
                        } else {
                            this.selection.start
                        }
                    } else {
                        p
                    };
                    this.selection = if e.click_count > 1 {
                        0..this.text.len()
                    } else {
                        anchor.min(p)..anchor.max(p)
                    };
                    this.reversed = p < anchor;
                    this.drag_anchor = (e.click_count == 1).then_some(anchor);
                    this.marked = None;
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                if e.pressed_button == Some(MouseButton::Left)
                    && let Some(anchor) = this.drag_anchor
                {
                    let p = this
                        .layout
                        .as_ref()
                        .map_or(this.text.len(), |line| {
                            line.closest_index_for_x(
                                e.position.x - this.bounds.left() + this.scroll_x,
                            )
                        })
                        .min(this.text.len());
                    this.selection = anchor.min(p)..anchor.max(p);
                    this.reversed = p < anchor;
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.drag_anchor = None),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.drag_anchor = None),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        window.handle_input(
                            &focus,
                            ElementInputHandler::new(bounds, entity.clone()),
                            cx,
                        );
                        entity.update(cx, |this, cx| {
                            let content: SharedString = if this.text.is_empty() {
                                this.placeholder.into()
                            } else {
                                this.text.clone().into()
                            };
                            let style = window.text_style();
                            let color = if this.text.is_empty() {
                                t.muted()
                            } else {
                                t.text()
                            };
                            let line = window.text_system().shape_line(
                                content.clone(),
                                px(13.0),
                                &[TextRun {
                                    len: content.len(),
                                    font: style.font(),
                                    color,
                                    background_color: None,
                                    underline: None,
                                    strikethrough: None,
                                }],
                                None,
                            );
                            let caret = if this.reversed {
                                this.selection.start
                            } else {
                                this.selection.end
                            };
                            let caret_x = line.x_for_index(caret);
                            if focused {
                                let width = (bounds.size.width - px(2.0)).max(px(1.0));
                                this.scroll_x =
                                    this.scroll_x.min(caret_x).max(caret_x - width).max(px(0.0));
                            }
                            if this.text.is_empty() {
                                this.scroll_x = px(0.0);
                            }
                            let origin = point(bounds.left() - this.scroll_x, bounds.top());
                            if focused {
                                let a = line.x_for_index(this.selection.start);
                                let b = line.x_for_index(this.selection.end);
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(origin.x + a, bounds.top()),
                                        size(
                                            if a == b { px(1.0) } else { b - a },
                                            bounds.size.height,
                                        ),
                                    ),
                                    t.accent().opacity(if a == b { 1.0 } else { 0.25 }),
                                ));
                            }
                            let _ = line.paint(origin, px(19.0), TextAlign::Left, None, window, cx);
                            this.layout = Some(line);
                            this.bounds = bounds;
                        });
                    },
                )
                .size_full(),
            )
    }
}

/// Move the active end while preserving the selection anchor, including reversals.
fn move_selection(
    text: &str,
    selection: Range<usize>,
    reversed: bool,
    key: &str,
    shift: bool,
) -> (Range<usize>, bool) {
    let head = if reversed {
        selection.start
    } else {
        selection.end
    };
    let anchor = if reversed {
        selection.end
    } else {
        selection.start
    };
    let next = match key {
        "home" => 0,
        "end" => text.len(),
        "left" if !shift && !selection.is_empty() => selection.start,
        "right" if !shift && !selection.is_empty() => selection.end,
        "left" => text[..head]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i),
        _ => head + text[head..].chars().next().map_or(0, char::len_utf8),
    };
    if shift {
        (anchor.min(next)..anchor.max(next), next < anchor)
    } else {
        (next..next, false)
    }
}
#[cfg(test)]
mod tests {
    use super::move_selection;
    #[test]
    fn selection_reverses_and_collapses_without_skipping() {
        let (range, reversed) = move_selection("aé猫z", 6..6, false, "left", true);
        assert_eq!(range, 3..6);
        assert!(reversed);
        assert_eq!(
            move_selection("aé猫z", range.clone(), reversed, "right", true),
            (6..6, false)
        );
        assert_eq!(
            move_selection("aé猫z", range.clone(), reversed, "left", false),
            (3..3, false)
        );
        assert_eq!(
            move_selection("aé猫z", range, reversed, "right", false),
            (6..6, false)
        );
        assert_eq!(
            move_selection("aé猫z", 1..6, false, "home", true),
            (0..1, true)
        );
    }
}
