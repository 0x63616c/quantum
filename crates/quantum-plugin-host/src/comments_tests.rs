use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Failure {
    None,
    Unbound,
    Bind,
    View,
    Interact,
    List,
    Error,
}
struct PanelPlugin {
    metadata: TestPlugin,
    failure: Failure,
}
impl PanelPlugin {
    fn new(failure: Failure) -> Self {
        let mut metadata = TestPlugin::new("dev.comments");
        metadata
            .declarations
            .push(Contribution::Panel(PanelContribution {
                id: "dev.comments.panel".into(),
                label_key: "Comments".into(),
                icon: None,
            }));
        Self { metadata, failure }
    }
}
impl Plugin for PanelPlugin {
    fn manifest(&self) -> Manifest {
        self.metadata.manifest()
    }
    fn contribute(&self, registrar: &mut Registrar) {
        self.metadata.contribute(registrar);
    }
    fn comments_panel(&self, _: &str) -> Option<&dyn CommentsPanel> {
        assert!(self.failure != Failure::Bind, "binding panic");
        (self.failure != Failure::Unbound).then_some(self)
    }
}
impl CommentsPanel for PanelPlugin {
    fn view(&self, cx: &dyn CommentsCx) -> CommentsView {
        assert!(self.failure != Failure::View, "view panic");
        CommentsView {
            state: cx.state().clone(),
            comments: vec![],
            empty: true,
            thread: None,
            region: None,
        }
    }
    fn interact(
        &self,
        cx: &dyn CommentsCx,
        _: CommentsAction,
    ) -> Result<CommentsOutcome, ToolError> {
        assert!(self.failure != Failure::Interact, "interaction panic");
        if self.failure == Failure::Error {
            return Err(ToolError::Gesture("rejected".into()));
        }
        Ok(CommentsOutcome {
            state: cx.state().clone(),
            ..Default::default()
        })
    }
    fn list(&self, _: &dyn CommentsCx, _: CommentsQuery) -> Value {
        assert!(self.failure != Failure::List, "listing panic");
        Value::Null
    }
}
#[derive(Default)]
struct Context {
    document: quantum_core::Document,
    state: CommentsState,
}
impl CommentsCx for Context {
    fn snapshot(&self) -> &Snapshot {
        &self.document.content
    }
    fn state(&self) -> &CommentsState {
        &self.state
    }
}
fn query() -> CommentsQuery {
    CommentsQuery {
        state: "all".into(),
        offset: 0,
        limit: 50,
        message_offset: 0,
        message_limit: 50,
    }
}

#[test]
fn discussion_bindings_are_enabled_only_and_reject_foreign_commands() {
    let mut host = Host::new(vec![Box::new(PanelPlugin::new(Failure::None))]);
    let cx = Context::default();
    let panel = "dev.comments.panel";
    assert!(host.has_comments_panel(panel).unwrap());
    assert!(
        host.dispatch_comments(panel, &cx, CommentsAction::Command("dev.other.run".into()))
            .unwrap()
            .is_none()
    );
    assert!(
        host.dispatch_comments(
            panel,
            &cx,
            CommentsAction::Command("dev.comments.run".into())
        )
        .unwrap()
        .is_some()
    );
    for enabled in [false, true] {
        if enabled {
            host.enable("dev.comments").unwrap();
        } else {
            host.disable("dev.comments").unwrap();
        }
        assert_eq!(host.has_comments_panel(panel).unwrap(), enabled);
        assert_eq!(host.comments_view(panel, &cx).unwrap().is_some(), enabled);
        assert_eq!(
            host.dispatch_comments(panel, &cx, CommentsAction::Cancel)
                .unwrap()
                .is_some(),
            enabled
        );
        assert_eq!(
            host.list_comments(panel, &cx, query()).unwrap().is_some(),
            enabled
        );
    }
    // Disabled bindings and foreign commands never invoke plugin code.
    assert_eq!(
        host.timings()
            .get("dev.comments", "comments")
            .unwrap()
            .count,
        6
    );
    conformance::check(|| Box::new(PanelPlugin::new(Failure::None)));
}

#[test]
fn discussion_errors_are_inert_and_panics_quarantine_every_entrypoint() {
    for failure in [
        Failure::Unbound,
        Failure::Bind,
        Failure::View,
        Failure::Interact,
        Failure::List,
        Failure::Error,
    ] {
        let mut host = Host::new(vec![Box::new(PanelPlugin::new(failure))]);
        let cx = Context::default();
        let panel = "dev.comments.panel";
        match failure {
            Failure::Unbound => {
                assert!(!host.has_comments_panel(panel).unwrap());
                assert!(host.comments_view(panel, &cx).unwrap().is_none());
                assert!(
                    host.dispatch_comments(panel, &cx, CommentsAction::Cancel)
                        .unwrap()
                        .is_none()
                );
                assert!(host.list_comments(panel, &cx, query()).unwrap().is_none());
            }
            Failure::Bind => assert!(host.has_comments_panel(panel).is_err()),
            Failure::View => assert!(host.comments_view(panel, &cx).is_err()),
            Failure::Interact | Failure::Error => assert!(
                host.dispatch_comments(panel, &cx, CommentsAction::Cancel)
                    .is_err()
            ),
            Failure::List => assert!(host.list_comments(panel, &cx, query()).is_err()),
            Failure::None => unreachable!(),
        }
        let faulted = !matches!(failure, Failure::Unbound | Failure::Error);
        assert_eq!(
            matches!(
                host.state("dev.comments"),
                Some(PluginState::Faulted { .. })
            ),
            faulted
        );
        assert_eq!(host.registry().panels().next().is_none(), faulted);
        assert!(cx.document.content.comments.is_empty());
        assert_eq!(cx.state, CommentsState::default());
    }
}
