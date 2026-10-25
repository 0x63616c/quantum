//! Local file sharing. Hosted links and permissions are not available in this build.
use super::*;

pub(super) struct Sharing {
    pub open: bool,
}
impl Sharing {
    pub fn new(_: &mut App) -> Self {
        Self { open: false }
    }
}
impl EditorView {
    pub(super) fn open_share(&mut self, cx: &mut Context<Self>) {
        self.sharing.open = true;
        cx.notify();
    }
    pub(super) fn share_popover(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        div()
            .absolute().top(px(88.0)).right(px(24.0))
            .child(self.floating().w(px(360.0)).p(px(20.0)).flex().flex_col().gap(px(16.0))
                .child(row().justify_between().child(section_title("Share a project file"))
                    .child(self.control("close-share", "close", "", cx, |this,window,cx| {
                        this.sharing.open=false;window.focus(&this.focus,cx);cx.notify();
                    }).tooltip(hint("Close sharing",t))))
                .child(div().line_height(px(21.0)).text_color(t.muted()).child("Save a .voxl copy to send through your preferred app. The file includes the model, comments and named versions."))
                .child(self.control("share-copy", "file-output", "Save a copy…", cx, |this,_,cx| {
                    this.sharing.open=false; this.choose_save_path(true,cx);
                }))
                .when_some(self.editor.path.clone(), |d,path| {
                    let shown=path.display().to_string();
                    d.child(div().text_size(px(12.0)).text_color(t.muted()).overflow_hidden().text_ellipsis().child(shown.clone()))
                     .child(self.control("share-path", "copy", "Copy file path", cx, move |this,_,cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(shown.clone()));this.status="File path copied".into();cx.notify();
                     }))
                })
                .child(div().pt(px(12.0)).border_t_1().border_color(t.line()).text_size(px(12.0)).text_color(t.muted()).child("Hosted links and online permissions are not available yet.")))
            .into_any_element()
    }
}
