//! Small native single-line field with clipboard, selection, and IME support.
use crate::theme::Theme;
use gpui::{prelude::*, *};
use std::ops::Range;

pub struct TextField {
    pub text: String,
    focus: FocusHandle,
    selection: Range<usize>,
    marked: Option<Range<usize>>,
    placeholder: &'static str,
    theme: Theme,
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
            placeholder,
            theme: Theme::Dark,
            layout: None,
            bounds: Bounds::default(),
        })
    }
    pub fn set(&mut self, text: String, cx: &mut Context<Self>) {
        self.selection = text.len()..text.len();
        self.text = text;
        self.marked = None;
        cx.notify();
    }
    pub fn appearance(&mut self, theme: Theme, cx: &mut Context<Self>) {
        if self.theme != theme {
            self.theme = theme;
            cx.notify();
        }
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
        self.byte(r.start)..self.byte(r.end)
    }
    fn to_utf16(&self, r: Range<usize>) -> Range<usize> {
        self.text[..r.start].encode_utf16().count()..self.text[..r.end].encode_utf16().count()
    }
    fn replace(&mut self, range: Option<Range<usize>>, text: &str, cx: &mut Context<Self>) {
        let r = range
            .map(|r| self.range_from_utf16(r))
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone());
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if self.text.len() - r.len() + text.len() > 8192 {
            return;
        }
        self.text.replace_range(r.clone(), &text);
        self.selection = r.start + text.len()..r.start + text.len();
        self.marked = None;
        cx.notify();
    }
    fn key(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = e.keystroke.key.as_str();
        let cmd = e.keystroke.modifiers.control || e.keystroke.modifiers.platform;
        let handled = match key {
            "a" if cmd => {
                self.selection = 0..self.text.len();
                true
            }
            "c" | "x" if cmd => {
                cx.write_to_clipboard(ClipboardItem::new_string(
                    self.text[self.selection.clone()].into(),
                ));
                if key == "x" {
                    self.replace(None, "", cx);
                }
                true
            }
            "v" if cmd => {
                if let Some(s) = cx.read_from_clipboard().and_then(|c| c.text()) {
                    self.replace(None, &s, cx);
                }
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
                let p = match key {
                    "home" => 0,
                    "end" => self.text.len(),
                    "left" => self.text[..self.selection.start]
                        .char_indices()
                        .next_back()
                        .map_or(0, |(i, _)| i),
                    _ => {
                        self.selection.end
                            + self.text[self.selection.end..]
                                .chars()
                                .next()
                                .map_or(0, char::len_utf8)
                    }
                };
                if e.keystroke.modifiers.shift {
                    self.selection = self.selection.start.min(p)..self.selection.end.max(p);
                } else {
                    self.selection = p..p;
                }
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
            reversed: false,
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
            .map(|r| self.byte(r.start))
            .or_else(|| self.marked.as_ref().map(|r| r.start))
            .unwrap_or(self.selection.start);
        self.replace(r, text, cx);
        let end = self.selection.end;
        self.marked = Some(start..end);
        if let Some(r) = selected {
            let base = self.to_utf16(start..start).start;
            self.selection = self.range_from_utf16(base + r.start..base + r.end);
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
                self.bounds.left() + l.x_for_index(r.start),
                self.bounds.top(),
            ),
            point(
                self.bounds.left() + l.x_for_index(r.end),
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
            .closest_index_for_x(p.x - self.bounds.left());
        Some(self.to_utf16(byte..byte).start)
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
            .overflow_hidden()
            .cursor_text()
            .on_key_down(cx.listener(Self::key))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    let p = this
                        .layout
                        .as_ref()
                        .map_or(this.text.len(), |l| {
                            l.closest_index_for_x(e.position.x - this.bounds.left())
                        })
                        .min(this.text.len());
                    this.selection = if e.click_count > 1 {
                        0..this.text.len()
                    } else {
                        p..p
                    };
                    cx.stop_propagation();
                    cx.notify();
                }),
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
                            if focused {
                                let a = line.x_for_index(this.selection.start);
                                let b = line.x_for_index(this.selection.end);
                                window.paint_quad(fill(
                                    Bounds::new(
                                        point(bounds.left() + a, bounds.top()),
                                        size(
                                            if a == b { px(1.0) } else { b - a },
                                            bounds.size.height,
                                        ),
                                    ),
                                    t.accent().opacity(if a == b { 1.0 } else { 0.25 }),
                                ));
                            }
                            let _ = line.paint(
                                bounds.origin,
                                px(19.0),
                                TextAlign::Left,
                                None,
                                window,
                                cx,
                            );
                            this.layout = Some(line);
                            this.bounds = bounds;
                        });
                    },
                )
                .size_full(),
            )
    }
}
