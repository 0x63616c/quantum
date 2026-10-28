//! Equivalent direct orchestration control, using the same production callbacks.
use crate::model::*;
use std::sync::Arc;
use voxelite_core::Document;

pub(crate) struct Direct {
    authority: Authority,
    selection: Selection,
    mirror: MirrorState,
    discussion: Discussion,
    published: Arc<ReadModel>,
}

impl Backend for Direct {
    fn new(document: Document) -> Result<Self> {
        let (authority, selection) = initial(document)?;
        let mirror = MirrorState::default();
        let discussion = Discussion::default();
        let published = Arc::new(ReadModel::capture(
            &authority,
            &selection,
            &mirror,
            &discussion,
        ));
        Ok(Self {
            authority,
            selection,
            mirror,
            discussion,
            published,
        })
    }

    fn dispatch(&mut self, request: Request) -> Result<()> {
        let mut work = Work {
            request: Some(request),
            ..Default::default()
        };
        mirror(&self.authority, &self.mirror, &mut work);
        comments(&self.authority, &self.discussion, &mut work);
        admit(
            &mut self.authority,
            &mut self.selection,
            &mut self.mirror,
            &mut self.discussion,
            &mut work,
        );
        if work.publish {
            self.published = Arc::new(ReadModel::capture(
                &self.authority,
                &self.selection,
                &self.mirror,
                &self.discussion,
            ));
        }
        work.result.expect("admission always records a result")
    }

    fn read(&self) -> Arc<ReadModel> {
        self.published.clone()
    }
    fn idle(&mut self) {}
    fn open_threads(&mut self) -> usize {
        self.authority
            .engine
            .document()
            .content
            .comments
            .iter()
            .filter(|c| !c.resolved)
            .count()
    }
}
