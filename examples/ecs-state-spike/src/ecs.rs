//! Owner-thread World with three typed feature components and an ordered Schedule.
use crate::model::{Result, *};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::SingleThreadedExecutor;
use std::{collections::BTreeMap, sync::Arc};
use voxelite_core::{Document, Uuid};

#[derive(Resource)]
struct Published(Arc<ReadModel>);

// Derived scene index only. UUIDs remain portable; Entity never enters .voxl/MCP.
#[derive(Component)]
struct Thread {
    id: Uuid,
    index: usize,
    resolved: bool,
}

#[derive(Resource, Default)]
struct ThreadIndex {
    revision: Option<u64>,
    entities: BTreeMap<Uuid, Entity>,
}

pub(crate) struct Ecs {
    world: World,
    schedule: Schedule,
    threads: QueryState<&'static Thread>,
}

impl Backend for Ecs {
    fn new(document: Document) -> Result<Self> {
        let (authority, selection) = initial(document)?;
        let mirror = MirrorState::default();
        let discussion = Discussion::default();
        let published = ReadModel::capture(&authority, &selection, &mirror, &discussion);
        let mut world = World::new();
        world.spawn((selection, mirror, discussion));
        world.insert_resource(authority);
        world.insert_resource(Published(Arc::new(published)));
        world.init_resource::<Work>();
        world.init_resource::<ThreadIndex>();
        let mut schedule = Schedule::default();
        schedule.set_executor(SingleThreadedExecutor::default());
        schedule.add_systems(
            (
                mirror_system,
                comments_system,
                admit_system,
                index_system,
                publish_system,
            )
                .chain(),
        );
        schedule.run(&mut world);
        let threads = world.query();
        Ok(Self {
            world,
            schedule,
            threads,
        })
    }

    fn dispatch(&mut self, request: Request) -> Result<()> {
        *self.world.resource_mut::<Work>() = Work {
            request: Some(request),
            ..Default::default()
        };
        self.schedule.run(&mut self.world);
        self.world
            .resource_mut::<Work>()
            .result
            .take()
            .expect("admission records a result")
    }

    fn read(&self) -> Arc<ReadModel> {
        self.world.resource::<Published>().0.clone()
    }
    fn idle(&mut self) {
        *self.world.resource_mut::<Work>() = Work::default();
        self.schedule.run(&mut self.world);
    }
    fn open_threads(&mut self) -> usize {
        self.threads
            .iter(&self.world)
            .filter(|t| !t.resolved)
            .count()
    }
}

fn mirror_system(a: Res<Authority>, m: Single<&MirrorState>, mut work: ResMut<Work>) {
    mirror(&a, &m, &mut work);
}

fn comments_system(a: Res<Authority>, d: Single<&Discussion>, mut work: ResMut<Work>) {
    comments(&a, &d, &mut work);
}

fn admit_system(
    mut a: ResMut<Authority>,
    state: Single<(&mut Selection, &mut MirrorState, &mut Discussion)>,
    mut work: ResMut<Work>,
) {
    let (mut s, mut m, mut d) = state.into_inner();
    admit(&mut a, &mut s, &mut m, &mut d, &mut work);
}

fn index_system(
    a: Res<Authority>,
    mut index: ResMut<ThreadIndex>,
    mut threads: Query<(Entity, &mut Thread)>,
    mut commands: Commands,
) {
    if index.revision == Some(a.engine.revision()) {
        return;
    }
    let comments = &a.engine.document().content.comments;
    let positions: BTreeMap<_, _> = comments
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id, i))
        .collect();
    for (entity, mut thread) in &mut threads {
        if let Some(&position) = positions.get(&thread.id) {
            thread.index = position;
            thread.resolved = comments[position].resolved;
        } else {
            commands.entity(entity).despawn();
            index.entities.remove(&thread.id);
        }
    }
    for (position, comment) in comments.iter().enumerate() {
        index.entities.entry(comment.id).or_insert_with(|| {
            commands
                .spawn(Thread {
                    id: comment.id,
                    index: position,
                    resolved: comment.resolved,
                })
                .id()
        });
    }
    index.revision = Some(a.engine.revision());
}

fn publish_system(
    a: Res<Authority>,
    state: Single<(&Selection, &MirrorState, &Discussion)>,
    work: Res<Work>,
    mut published: ResMut<Published>,
) {
    if work.publish {
        let (s, m, d) = *state;
        published.0 = Arc::new(ReadModel::capture(&a, s, m, d));
    }
}

#[cfg(test)]
impl Ecs {
    pub(crate) fn indexed_threads(&mut self) -> Vec<(Uuid, usize, bool)> {
        let mut rows: Vec<_> = self
            .threads
            .iter(&self.world)
            .map(|t| (t.id, t.index, t.resolved))
            .collect();
        rows.sort_by_key(|row| row.1);
        rows
    }
}
