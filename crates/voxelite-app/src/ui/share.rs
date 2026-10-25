//! Local Share popover.
use super::*;
use voxelite_app::share::{Access, LocalShareBackend, Person, Share, ShareBackend};
use voxelite_core::Uuid;
pub(super) struct Sharing {
    pub open: bool,
    pub record: Option<Share>,
    pub person: Entity<crate::text_field::TextField>,
}
impl Sharing {
    pub fn new(cx: &mut App) -> Self {
        Self {
            open: false,
            record: None,
            person: crate::text_field::TextField::new("Person or role", cx),
        }
    }
}
impl EditorView {
    pub(super) fn open_share(&mut self, cx: &mut Context<Self>) {
        let result = LocalShareBackend::new(voxelite_app::palettes::data_dir().join("shares.json"))
            .get_or_create(self.editor.engine.document().id);
        match result {
            Ok(record) => {
                self.sharing.record = Some(record);
                self.sharing.open = true;
            }
            Err(e) => self.report::<()>(Err(e), "", cx),
        };
        cx.notify();
    }
    fn edit_share(&mut self, cx: &mut Context<Self>, edit: impl FnOnce(&mut Share)) {
        let Some(mut share) = self.sharing.record.clone() else {
            return;
        };
        edit(&mut share);
        let result = LocalShareBackend::new(voxelite_app::palettes::data_dir().join("shares.json"))
            .save(share.clone());
        if result.is_ok() {
            self.sharing.record = Some(share);
        }
        self.report(result, "Local share settings saved", cx);
    }
    pub(super) fn share_popover(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        let Some(s) = &self.sharing.record else {
            return div().into_any_element();
        };
        div().absolute().top(px(88.0)).right(px(24.0)).child(self.floating().w(px(410.0)).p(px(20.0)).flex().flex_col().gap(px(16.0))
            .child(row().justify_between().child(section_title("Share project")).child(mono("LOCAL").text_color(t.muted())))
            .child(div().text_size(px(12.0)).text_color(t.muted()).child("Local preview only. Links and access settings stay on this device; no invitations are sent."))
            .child(row().gap(px(8.0)).child(div().flex_1().overflow_hidden().text_ellipsis().px(px(10.0)).py(px(9.0)).border_1().border_color(t.line()).rounded(px(6.0)).child(mono(s.link())))
                .child(self.control("copy-share","copy","Copy link",cx,|this,_,cx| {if let Some(s)=&this.sharing.record {cx.write_to_clipboard(ClipboardItem::new_string(s.link()));this.status="Local link copied".into();cx.notify();}})))
            .child(div().text_size(px(12.0)).text_color(t.muted()).child("Link access"))
            .child(row().gap(px(6.0)).children(Access::ALL.into_iter().enumerate().map(|(i,a)|self.control(("share-access",i),["eye", "comment", "git-fork"][i],a.label(),cx,move |this,_,cx|this.edit_share(cx,|s|s.access=a)).flex_1().justify_center().when(s.access==a,|d|d.bg(t.selected()).text_color(t.accent())))))
            .child(row().pt(px(8.0)).border_t_1().border_color(t.line()).justify_between().child("You").child(div().text_color(t.muted()).child("Owner")))
            .child(div().id("share-people").max_h(px(180.0)).overflow_y_scroll().children(s.people.iter().enumerate().map(|(i,p)| {
                let id=p.id;let next=match p.access{Access::View=>Access::Comment,Access::Comment=>Access::Remix,Access::Remix=>Access::View};
                row().gap(px(8.0)).mb(px(6.0)).child(div().flex_1().overflow_hidden().text_ellipsis().child(p.name.clone()))
                    .child(self.control(("person-access",i),"chevron",p.access.label(),cx,move |this,_,cx|this.edit_share(cx,|s|{if let Some(p)=s.people.iter_mut().find(|p|p.id==id){p.access=next;}})).tooltip(hint("Cycle View / Comment / Remix",t)))
                    .child(self.control(("person-remove",i),"close","×",cx,move |this,_,cx|this.edit_share(cx,|s|s.people.retain(|p|p.id!=id))).tooltip(hint("Remove person",t)))
            })))
            .child(row().gap(px(8.0)).child(div().flex_1().child(self.sharing.person.clone())).child(self.control("add-person","plus","Add",cx,|this,_,cx| {let name=this.sharing.person.read(cx).text.trim().to_string();this.edit_share(cx,|s|s.people.push(Person{id:Uuid::new_v4(),name,access:Access::View}));if !this.error{this.sharing.person.update(cx,|f,cx|f.set(String::new(),cx));}}))))
            .into_any_element()
    }
}
