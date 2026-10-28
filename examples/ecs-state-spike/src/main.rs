//! Throwaway state-storage experiment; no product entry points depend on this crate.
#[cfg(any(not(feature = "ecs"), test))]
mod direct;
#[cfg(feature = "ecs")]
mod ecs;
mod model;
#[cfg(test)]
mod tests;

use model::*;
use std::{hint::black_box, time::Instant};
use voxelite_core::{Actor, Document, Transaction};
use voxelite_plugin_api::*;

#[cfg(not(feature = "ecs"))]
type App = direct::Direct;
#[cfg(feature = "ecs")]
type App = ecs::Ecs;

fn tx(revision: u64, id: u128, command: Command, origin: Origin) -> Transaction {
    Transaction {
        expected_revision: revision,
        id: Uuid::from_u128(id),
        actor: Actor::new(origin),
        label: "Spike edit".into(),
        command,
    }
}

fn query() -> CommentsQuery {
    CommentsQuery {
        state: "all".into(),
        offset: 0,
        limit: 32,
        message_offset: 0,
        message_limit: 32,
    }
}

fn blank() -> Document {
    Document::new(Uuid::from_u128(1), Uuid::from_u128(2))
}

fn fixture(count: usize) -> Result<Document> {
    let mut engine = voxelite_core::Engine::new(blank())?;
    for i in 0..count {
        engine.execute(tx(
            engine.revision(),
            3 + i as u128,
            Command::CreateComment {
                id: Uuid::from_u128(100 + i as u128),
                anchor: Anchor::Voxel { at: [0, 0, 0] },
                layer: None,
                message: Uuid::from_u128(10_000 + i as u128),
                text: "Measured thread".into(),
            },
            Origin::Human,
        ))?;
    }
    Ok(engine.document().clone())
}

// Exercise the same features in both release executables, preventing unfair
// dead-code elimination in the binary-size comparison.
fn scenario<B: Backend>() -> Result<B> {
    let mut app = B::new(blank())?;
    app.dispatch(Request::Mirror([true, false, false]))?;
    app.dispatch(Request::Select(Uuid::from_u128(2)))?;
    let slot = app.read().live.content.palette.order[0];
    app.dispatch(Request::Tool(tx(
        0,
        3,
        Command::SetVoxels {
            layer: Uuid::from_u128(2),
            voxels: vec![Voxel {
                at: [0, 0, 0],
                slot,
            }],
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
        Origin::Human,
    )))?;
    app.dispatch(Request::Comments {
        action: CommentsAction::Create {
            id: Uuid::from_u128(4),
            anchor: Anchor::Voxel { at: [0, 0, 0] },
            layer: CommentLayer::Automatic,
            message: Uuid::from_u128(5),
            text: "Hello".into(),
        },
        actor: Actor::new(Origin::Human),
        revision: 1,
        id: Uuid::from_u128(6),
    })?;
    app.dispatch(Request::Undo {
        revision: 2,
        head: Uuid::from_u128(6),
    })?;
    app.dispatch(Request::Redo {
        revision: 3,
        head: Uuid::from_u128(6),
    })?;
    app.dispatch(Request::Preview(None))?;
    black_box(app.read().comments_view());
    black_box(app.read().mcp_comments(query())?);
    app.dispatch(Request::Live)?;
    app.dispatch(Request::Enable {
        mirror: false,
        comments: false,
    })?;
    app.dispatch(Request::Enable {
        mirror: true,
        comments: true,
    })?;
    app.dispatch(Request::Agent(tx(
        4,
        7,
        Command::Resolve {
            id: Uuid::from_u128(4),
            resolved: true,
        },
        Origin::Agent,
    )))?;
    let read = app.read();
    black_box((
        read.selection.layer,
        read.mirror,
        read.revision,
        app.open_threads(),
    ));
    Ok(app)
}

fn measure(name: &str, iterations: usize, mut f: impl FnMut()) {
    for _ in 0..100 {
        f();
    }
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    println!(
        "{name},{iterations},{:.2}",
        start.elapsed().as_nanos() as f64 / iterations as f64
    );
}

fn benchmark() -> Result<()> {
    let mut app = scenario::<App>()?;
    println!("operation,iterations,ns_per_operation");
    measure("idle", 100_000, || {
        black_box(&mut app).idle();
    });
    measure("read_publication", 200_000, || {
        black_box(black_box(&app).read());
    });
    measure("select_layer", 30_000, || {
        app.dispatch(Request::Select(black_box(Uuid::from_u128(2))))
            .unwrap();
    });
    let slot = app.read().live.content.palette.order[0];
    measure("mirrored_tool_noop", 3_000, || {
        app.dispatch(Request::Tool(tx(
            app.read().revision,
            20,
            Command::SetVoxels {
                layer: Uuid::from_u128(2),
                voxels: vec![Voxel {
                    at: [0, 0, 0],
                    slot,
                }],
                mode: Mode::Place,
                mirror: Mirror::default(),
            },
            Origin::Human,
        )))
        .unwrap();
    });
    for count in [1, 128, 1024] {
        let mut app = App::new(fixture(count)?)?;
        measure(&format!("thread_query_{count}"), 100_000, || {
            black_box(app.open_threads());
        });
        measure(&format!("comments_view_{count}"), 1_000, || {
            black_box(app.read().comments_view());
        });
        measure(&format!("mcp_list_{count}"), 1_000, || {
            black_box(app.read().mcp_comments(query()).unwrap());
        });
        let mut resolved = false;
        let mut id = 100_000;
        measure(&format!("resolve_commit_{count}"), 200, || {
            resolved = !resolved;
            id += 1;
            app.dispatch(Request::Comments {
                action: CommentsAction::Resolve {
                    id: Uuid::from_u128(100),
                    resolved,
                },
                actor: Actor::new(Origin::Human),
                revision: app.read().revision,
                id: Uuid::from_u128(id),
            })
            .unwrap();
        });
        measure(&format!("undo_redo_{count}"), 200, || {
            app.dispatch(Request::Undo {
                revision: app.read().revision,
                head: Uuid::from_u128(id),
            })
            .unwrap();
            app.dispatch(Request::Redo {
                revision: app.read().revision,
                head: Uuid::from_u128(id),
            })
            .unwrap();
        });
    }
    Ok(())
}

fn main() -> Result<()> {
    benchmark()
}
