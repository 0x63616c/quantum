//! Shared behavior for the direct control and ECS experiment.
use std::sync::Arc;
use voxelite_core::{Actor, Document, Engine, Transaction};
use voxelite_plugin_api::*;
use voxelite_plugin_comments::CommentsPlugin;
use voxelite_plugin_mirror::{AXIS_SETTINGS, MirrorPlugin};

pub(crate) type Result<T> = std::result::Result<T, ToolError>;

#[cfg_attr(feature = "ecs", derive(bevy_ecs::resource::Resource))]
pub(crate) struct Authority {
    pub engine: Engine,
    pub preview: Option<Arc<Document>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ecs", derive(bevy_ecs::component::Component))]
pub(crate) struct Selection {
    pub layer: Uuid,
    pub box_start: Option<Pos>,
}

#[derive(Clone)]
#[cfg_attr(feature = "ecs", derive(bevy_ecs::component::Component))]
pub(crate) struct MirrorState {
    pub enabled: bool,
    pub axes: [SettingValue; 3],
}

impl Default for MirrorState {
    fn default() -> Self {
        Self {
            enabled: true,
            axes: std::array::from_fn(|_| SettingValue::Bool(false)),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "ecs", derive(bevy_ecs::component::Component))]
pub(crate) struct Discussion {
    pub enabled: bool,
    pub state: CommentsState,
}

impl Default for Discussion {
    fn default() -> Self {
        Self {
            enabled: true,
            state: CommentsState::default(),
        }
    }
}

#[derive(Clone)]
pub(crate) enum Request {
    Tool(Transaction),
    Agent(Transaction),
    Comments {
        action: CommentsAction,
        actor: Actor,
        revision: u64,
        id: Uuid,
    },
    Undo {
        revision: u64,
        head: Uuid,
    },
    Redo {
        revision: u64,
        head: Uuid,
    },
    Select(Uuid),
    Mirror([bool; 3]),
    Enable {
        mirror: bool,
        comments: bool,
    },
    Preview(Option<Uuid>),
    Live,
}

#[derive(Default)]
#[cfg_attr(feature = "ecs", derive(bevy_ecs::resource::Resource))]
pub(crate) struct Work {
    pub request: Option<Request>,
    pub proposed: Option<CommentsOutcome>,
    pub result: Option<Result<()>>,
    pub publish: bool,
}

/// Owned publication: GPUI and an MCP worker never borrow a running World.
#[derive(Clone)]
pub(crate) struct ReadModel {
    pub live: Arc<Document>,
    pub displayed: Arc<Document>,
    pub revision: u64,
    pub selection: Selection,
    pub mirror: [bool; 3],
    pub discussion: Discussion,
    pub preview: bool,
}

impl ReadModel {
    pub fn capture(a: &Authority, s: &Selection, m: &MirrorState, d: &Discussion) -> Self {
        Self {
            live: a.engine.snapshot(),
            displayed: a.preview.clone().unwrap_or_else(|| a.engine.snapshot()),
            revision: a.engine.revision(),
            selection: s.clone(),
            mirror: std::array::from_fn(|i| m.enabled && m.axes[i] == SettingValue::Bool(true)),
            discussion: d.clone(),
            preview: a.preview.is_some(),
        }
    }

    pub fn comments_view(&self) -> Option<CommentsView> {
        self.discussion.enabled.then(|| CommentsPlugin.view(self))
    }

    pub fn mcp_comments(&self, query: CommentsQuery) -> Result<Value> {
        if !self.discussion.enabled {
            return Err(ToolError::Gesture("Comments plugin is unavailable".into()));
        }
        // Attached MCP sees live state even while GPUI inspects a history preview.
        Ok(CommentsPlugin.list(
            &CommentContext {
                live: &self.live.content,
                displayed: &self.live.content,
                state: &CommentsState::default(),
                preview: false,
            },
            query,
        ))
    }
}

impl CommentsCx for ReadModel {
    fn snapshot(&self) -> &Snapshot {
        &self.displayed.content
    }
    fn live_snapshot(&self) -> &Snapshot {
        &self.live.content
    }
    fn state(&self) -> &CommentsState {
        &self.discussion.state
    }
    fn preview(&self) -> bool {
        self.preview
    }
}

struct CommentContext<'a> {
    live: &'a Snapshot,
    displayed: &'a Snapshot,
    state: &'a CommentsState,
    preview: bool,
}

impl CommentsCx for CommentContext<'_> {
    fn snapshot(&self) -> &Snapshot {
        self.displayed
    }
    fn live_snapshot(&self) -> &Snapshot {
        self.live
    }
    fn state(&self) -> &CommentsState {
        self.state
    }
    fn preview(&self) -> bool {
        self.preview
    }
}

struct MirrorContext<'a>(Region, &'a MirrorState);
impl TransformCx for MirrorContext<'_> {
    fn bounds(&self) -> Region {
        self.0
    }
    fn setting(&self, key: &str) -> Option<&SettingValue> {
        AXIS_SETTINGS
            .iter()
            .position(|k| *k == key)
            .map(|i| &self.1.axes[i])
    }
}

pub(crate) fn mirror(a: &Authority, m: &MirrorState, work: &mut Work) {
    if let Some(Request::Tool(tx)) = &mut work.request
        && m.enabled
    {
        let commands = MirrorPlugin.transform(
            &MirrorContext(a.engine.document().content.bounds, m),
            vec![tx.command.clone()],
        );
        // Native tool proposals are committed as an ordered batch today.
        tx.command = Command::Batch(commands);
    }
}

pub(crate) fn comments(a: &Authority, d: &Discussion, work: &mut Work) {
    if let Some(Request::Comments { action, actor, .. }) = &work.request {
        if !d.enabled {
            work.result = Some(Err(ToolError::Gesture(
                "Comments plugin is unavailable".into(),
            )));
            return;
        }
        let live = &a.engine.document().content;
        let agent = actor.origin == Origin::Agent;
        let empty = CommentsState::default();
        let cx = CommentContext {
            live,
            displayed: if agent {
                live
            } else {
                a.preview.as_ref().map_or(live, |p| &p.content)
            },
            state: if agent { &empty } else { &d.state },
            preview: !agent && a.preview.is_some(),
        };
        match CommentsPanel::interact(&CommentsPlugin, &cx, action.clone()) {
            Ok(outcome) => work.proposed = Some(outcome),
            Err(error) => work.result = Some(Err(error)),
        }
    }
}

pub(crate) fn admit(
    a: &mut Authority,
    s: &mut Selection,
    m: &mut MirrorState,
    d: &mut Discussion,
    work: &mut Work,
) {
    let Some(request) = work.request.take() else {
        return;
    };
    if work.result.is_some() {
        return;
    }
    let result = (|| -> Result<()> {
        let mut reconcile = false;
        match request {
            Request::Tool(tx) | Request::Agent(tx) => {
                // Agent activity returns the editor to live only after admission.
                let agent = tx.actor.origin == Origin::Agent;
                if !agent {
                    ensure_live(a)?;
                }
                a.engine.execute(tx)?;
                if agent {
                    a.preview = None;
                }
                reconcile = true;
            }
            Request::Comments {
                actor,
                revision,
                id,
                ..
            } => {
                let agent = actor.origin == Origin::Agent;
                let out = work
                    .proposed
                    .take()
                    .expect("comments system precedes admission");
                if !out.commands.is_empty() {
                    if !agent {
                        ensure_live(a)?;
                    }
                    let command = match out.commands.as_slice() {
                        [command] => command.clone(),
                        _ => Command::Batch(out.commands),
                    };
                    a.engine.execute(Transaction {
                        expected_revision: revision,
                        id,
                        actor,
                        label: out.label,
                        command,
                    })?;
                    if agent {
                        a.preview = None;
                    }
                    reconcile = true;
                }
                if !agent {
                    d.state = out.state;
                }
            }
            Request::Undo { revision, head } => {
                ensure_live(a)?;
                a.engine.undo(revision, head)?;
                reconcile = true;
            }
            Request::Redo { revision, head } => {
                ensure_live(a)?;
                a.engine.redo(revision, head)?;
                reconcile = true;
            }
            Request::Select(id) => {
                a.engine.document().content.layer(id)?;
                s.layer = id;
            }
            Request::Mirror(axes) => {
                if m.enabled {
                    m.axes = axes.map(SettingValue::Bool);
                }
            }
            Request::Enable { mirror, comments } => {
                m.enabled = mirror;
                d.enabled = comments;
                if !comments {
                    d.state.placing = false;
                    d.state.first = None;
                    d.state.draft = None;
                }
            }
            Request::Preview(head) => a.preview = Some(a.engine.preview(head)?),
            Request::Live => a.preview = None,
        }
        if reconcile {
            let snapshot = &a.engine.document().content;
            if snapshot.layer(s.layer).is_err() {
                s.layer = snapshot.layers[0].id;
            }
            s.box_start = None;
        }
        Ok(())
    })();
    work.publish = result.is_ok();
    work.result = Some(result);
}

fn ensure_live(a: &Authority) -> Result<()> {
    if a.preview.is_some() {
        return Err(ToolError::Gesture("Return to live before editing".into()));
    }
    Ok(())
}

pub(crate) fn initial(document: Document) -> Result<(Authority, Selection)> {
    let engine = Engine::new(document)?;
    let layers = &engine.document().content.layers;
    let layer = layers
        .iter()
        .rev()
        .find(|l| l.visible && !l.locked)
        .unwrap_or(&layers[0])
        .id;
    Ok((
        Authority {
            engine,
            preview: None,
        },
        Selection {
            layer,
            box_start: None,
        },
    ))
}

pub(crate) trait Backend: Sized {
    fn new(document: Document) -> Result<Self>;
    fn dispatch(&mut self, request: Request) -> Result<()>;
    fn read(&self) -> Arc<ReadModel>;
    fn idle(&mut self);
    fn open_threads(&mut self) -> usize;
}
