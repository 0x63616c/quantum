use super::*;
use crate::direct::Direct;
use std::sync::Arc;
use voxelite_core::LayerEdit;

fn check_contract<B: Backend>() {
    let mut app = scenario::<B>().unwrap();
    let original = app.read();
    assert_eq!(original.live.content.layers[0].voxels.len(), 2);
    assert_eq!(original.revision, 5);
    assert_eq!(original.mirror, [true, false, false]);
    assert_eq!(
        original.live.content.comments[0].messages[0].origin,
        Origin::Human
    );

    // Failure must roll back earlier commands, preserve publication/selection,
    // and leave revision/history unchanged.
    let bad = Request::Agent(tx(
        5,
        50,
        Command::Batch(vec![
            Command::Layer(LayerEdit::Create {
                id: Uuid::from_u128(49),
                name: "Must roll back".into(),
                index: 1,
            }),
            Command::Layer(LayerEdit::Delete(Uuid::from_u128(999))),
        ]),
        Origin::Agent,
    ));
    assert!(app.dispatch(bad).is_err());
    assert!(Arc::ptr_eq(&original, &app.read()));
    assert!(
        app.dispatch(Request::Agent(tx(
            4,
            51,
            Command::Resolve {
                id: Uuid::from_u128(4),
                resolved: false,
            },
            Origin::Agent
        )))
        .is_err()
    );
    assert!(
        app.dispatch(Request::Undo {
            revision: 5,
            head: Uuid::from_u128(6)
        })
        .is_err()
    );
    assert_eq!(app.read().revision, 5);

    // Empty text is rejected by core after the panel proposed clearing its draft.
    app.dispatch(Request::Comments {
        action: CommentsAction::Anchor([0, 0, 0]),
        actor: Actor::new(Origin::Human),
        revision: 5,
        id: Uuid::from_u128(52),
    })
    .unwrap();
    let draft = app.read();
    assert!(
        app.dispatch(Request::Comments {
            action: CommentsAction::Post(" ".into()),
            actor: Actor::new(Origin::Human),
            revision: 5,
            id: Uuid::from_u128(53),
        })
        .is_err()
    );
    assert!(Arc::ptr_eq(&draft, &app.read()));
    assert!(app.read().discussion.state.draft.is_some());

    // Mirror and panel state stay transient. Disable/enable retains axes and
    // removes active drafts without changing document history.
    app.dispatch(Request::Enable {
        mirror: false,
        comments: false,
    })
    .unwrap();
    assert_eq!(app.read().mirror, [false; 3]);
    assert!(app.read().discussion.state.draft.is_none());
    assert!(app.read().comments_view().is_none());
    assert!(app.read().mcp_comments(query()).is_err());
    app.dispatch(Request::Mirror([false; 3])).unwrap();
    app.dispatch(Request::Enable {
        mirror: true,
        comments: true,
    })
    .unwrap();
    assert_eq!(app.read().mirror, [true, false, false]);
    assert_eq!(app.read().revision, 5);

    app.dispatch(Request::Preview(None)).unwrap();
    assert!(app.read().displayed.content.comments.is_empty());
    assert_eq!(app.read().mcp_comments(query()).unwrap()["total"], 1);
    assert!(
        app.dispatch(Request::Tool(tx(
            5,
            54,
            Command::Resolve {
                id: Uuid::from_u128(4),
                resolved: false,
            },
            Origin::Human
        )))
        .is_err()
    );
    assert!(app.read().preview);

    // Attached agent commit exits preview, bypasses UI mirror settings, and
    // publishes immutable state to both consumers in the same dispatch.
    let slot = app.read().live.content.palette.order[0];
    app.dispatch(Request::Agent(tx(
        5,
        55,
        Command::SetVoxels {
            layer: Uuid::from_u128(2),
            voxels: vec![Voxel {
                at: [1, 0, 0],
                slot,
            }],
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
        Origin::Agent,
    )))
    .unwrap();
    assert!(!app.read().preview);
    assert_eq!(app.read().live.content.layers[0].voxels.len(), 3);
    assert_eq!(original.live.content.layers[0].voxels.len(), 2);

    // Active-layer selection is reconciled when undo removes its target.
    app.dispatch(Request::Agent(tx(
        6,
        56,
        Command::Layer(LayerEdit::Create {
            id: Uuid::from_u128(8),
            name: "Second".into(),
            index: 1,
        }),
        Origin::Agent,
    )))
    .unwrap();
    app.dispatch(Request::Select(Uuid::from_u128(8))).unwrap();
    app.dispatch(Request::Undo {
        revision: 7,
        head: Uuid::from_u128(56),
    })
    .unwrap();
    assert_eq!(app.read().selection.layer, Uuid::from_u128(2));
    app.dispatch(Request::Redo {
        revision: 8,
        head: Uuid::from_u128(56),
    })
    .unwrap();
    assert_eq!(app.read().selection.layer, Uuid::from_u128(2));

    // Existing no-op and redo rules remain Engine rules.
    app.dispatch(Request::Agent(tx(
        9,
        57,
        Command::Resolve {
            id: Uuid::from_u128(4),
            resolved: true,
        },
        Origin::Agent,
    )))
    .unwrap();
    assert_eq!(app.read().revision, 9);
    app.dispatch(Request::Undo {
        revision: 9,
        head: Uuid::from_u128(56),
    })
    .unwrap();
    app.dispatch(Request::Agent(tx(
        10,
        58,
        Command::Resolve {
            id: Uuid::from_u128(4),
            resolved: false,
        },
        Origin::Agent,
    )))
    .unwrap();
    assert!(
        app.dispatch(Request::Redo {
            revision: 11,
            head: Uuid::from_u128(56)
        })
        .is_err()
    );
}

#[test]
fn direct_preserves_transactions_and_transient_state() {
    check_contract::<Direct>();
}

#[cfg(feature = "ecs")]
#[test]
fn ecs_preserves_transactions_and_transient_state() {
    check_contract::<ecs::Ecs>();
}

#[cfg(feature = "ecs")]
#[test]
fn ecs_index_tracks_undo_redo_and_agent_edits_without_exposing_entity_ids() {
    let mut app = scenario::<ecs::Ecs>().unwrap();
    assert_eq!(app.indexed_threads(), vec![(Uuid::from_u128(4), 0, true)]);
    app.dispatch(Request::Undo {
        revision: 5,
        head: Uuid::from_u128(7),
    })
    .unwrap();
    assert_eq!(app.open_threads(), 1);
    app.dispatch(Request::Undo {
        revision: 6,
        head: Uuid::from_u128(6),
    })
    .unwrap();
    assert!(app.indexed_threads().is_empty());
    app.dispatch(Request::Redo {
        revision: 7,
        head: Uuid::from_u128(6),
    })
    .unwrap();
    assert_eq!(app.indexed_threads(), vec![(Uuid::from_u128(4), 0, false)]);
    assert_eq!(
        app.read().mcp_comments(query()).unwrap()["comments"][0]["thread"]["id"],
        Uuid::from_u128(4).to_string()
    );
}

#[cfg(feature = "ecs")]
#[test]
fn both_backends_produce_identical_documents_views_and_mcp_pages() {
    let mut direct = scenario::<Direct>().unwrap();
    let mut ecs = scenario::<ecs::Ecs>().unwrap();
    for i in 0..30 {
        let request = Request::Comments {
            action: CommentsAction::Resolve {
                id: Uuid::from_u128(4),
                resolved: i % 2 == 0,
            },
            actor: Actor::new(Origin::Agent),
            revision: direct.read().revision,
            id: Uuid::from_u128(100 + i),
        };
        direct.dispatch(request.clone()).unwrap();
        ecs.dispatch(request).unwrap();
        let a = direct.read();
        let b = ecs.read();
        assert_eq!(a.revision, b.revision);
        assert_eq!(a.live, b.live);
        assert_eq!(a.selection, b.selection);
        assert_eq!(a.comments_view(), b.comments_view());
        assert_eq!(
            a.mcp_comments(query()).unwrap(),
            b.mcp_comments(query()).unwrap()
        );
        assert_eq!(direct.open_threads(), ecs.open_threads());
    }
}
