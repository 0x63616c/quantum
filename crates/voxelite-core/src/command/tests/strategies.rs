use super::*;
use proptest::prelude::*;

fn color() -> impl Strategy<Value = Swatch> {
    any::<[u8; 3]>().prop_map(|[r, g, b]| Swatch::rgb(r, g, b))
}

pub(super) fn document() -> impl Strategy<Value = Document> {
    (
        prop::array::uniform3(prop::sample::select(vec![-17, -2, 0, 15])),
        2i32..=4,
        prop::collection::vec(color(), 1..=4),
        prop::collection::vec(
            (
                any::<bool>(),
                any::<bool>(),
                prop::collection::vec((any::<[u8; 3]>(), any::<u8>()), 0..16),
            ),
            1..=3,
        ),
        any::<bool>(),
    )
        .prop_map(|(min, side, colors, layers, reverse)| {
            let mut doc = Document::new(id(1), id(2));
            doc.content.bounds = Region::new(min, min.map(|v| v + side)).unwrap();
            doc.content.palette = Palette::from_colors(colors).unwrap();
            if reverse {
                doc.content.palette.order.reverse();
            }
            doc.content.layers = layers
                .into_iter()
                .enumerate()
                .map(|(i, (visible, locked, cells))| {
                    let mut layer = Layer::new(id(i as u128 + 2), format!("Layer {i}"));
                    layer.visible = visible;
                    layer.locked = locked;
                    for (p, value) in cells {
                        let at = std::array::from_fn(|a| min[a] + i32::from(p[a]) % side);
                        let slot = u16::from(value) % (doc.content.palette.slots.len() as u16 + 1);
                        layer.voxels.set(at, slot);
                    }
                    layer
                        .extra
                        .insert("layer_note".into(), serde_json::json!(i));
                    layer
                })
                .collect();
            doc.content
                .extra
                .insert("note".into(), serde_json::json!("kept"));
            doc.extensions
                .insert(
                    "org.example",
                    "state",
                    DataSection::bytes(1, &[0, 255]).unwrap(),
                )
                .unwrap();
            doc.lineage = Some(Lineage {
                document: id(90),
                version: id(91),
            });
            doc
        })
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Set,
    Erase,
    Fill,
    Box,
    Ellipsoid,
    Cylinder,
    Line,
    Transform,
    AddColor,
    SetColor,
    RemoveColor,
    ReorderColors,
    CreateLayer,
    UpdateLayer,
    DuplicateLayer,
    ReorderLayers,
    DeleteLayer,
    Rename,
    Resize,
    CreateComment,
    Reply,
    Reanchor,
    Resolve,
    SaveVersion,
    RestoreVersion,
    Batch,
}

const KINDS: [Kind; 26] = [
    Kind::Set,
    Kind::Erase,
    Kind::Fill,
    Kind::Box,
    Kind::Ellipsoid,
    Kind::Cylinder,
    Kind::Line,
    Kind::Transform,
    Kind::AddColor,
    Kind::SetColor,
    Kind::RemoveColor,
    Kind::ReorderColors,
    Kind::CreateLayer,
    Kind::UpdateLayer,
    Kind::DuplicateLayer,
    Kind::ReorderLayers,
    Kind::DeleteLayer,
    Kind::Rename,
    Kind::Resize,
    Kind::CreateComment,
    Kind::Reply,
    Kind::Reanchor,
    Kind::Resolve,
    Kind::SaveVersion,
    Kind::RestoreVersion,
    Kind::Batch,
];

#[derive(Clone, Debug)]
pub(super) struct Step {
    kind: Kind,
    pick: usize,
    from: [u8; 3],
    to: [u8; 3],
    flags: [bool; 3],
    mode: Mode,
    color: Swatch,
}

pub(super) fn step() -> impl Strategy<Value = Step> {
    (
        prop::sample::select(KINDS.to_vec()),
        0usize..16,
        any::<[u8; 3]>(),
        any::<[u8; 3]>(),
        any::<[bool; 3]>(),
        prop::sample::select(vec![Mode::Place, Mode::Paint, Mode::Erase]),
        color(),
    )
        .prop_map(|(kind, pick, from, to, flags, mode, color)| Step {
            kind,
            pick,
            from,
            to,
            flags,
            mode,
            color,
        })
}

// Resolve identities and preconditions from the evolving model. Never discard an
// execute error: every generated command must succeed, including after shrinking.
impl Step {
    pub(super) fn command(&self, e: &Engine, serial: u128) -> Command {
        let s = &e.document().content;
        let bounds = s.bounds;
        let layer = &s.layers[self.pick % s.layers.len()];
        let slot = s.palette.order[self.pick % s.palette.order.len()];
        let point = |p: [u8; 3]| {
            std::array::from_fn(|a| {
                bounds.min[a] + i32::from(p[a]) % (bounds.max[a] - bounds.min[a])
            })
        };
        let from = point(self.from);
        let to = point(self.to);
        let region = Region::new(
            std::array::from_fn(|a| from[a].min(to[a])),
            std::array::from_fn(|a| from[a].max(to[a]) + 1),
        )
        .unwrap();
        let mirror = Mirror::centered(bounds, self.flags);
        let name = format!("Edit {serial}");
        let fresh = id(1000 + serial * 2);
        let message = id(1001 + serial * 2);
        let axis = [Axis::X, Axis::Y, Axis::Z][self.pick % 3];
        let create_layer = || {
            Command::Layer(LayerEdit::Create {
                id: fresh,
                name: name.clone(),
                index: self.pick % (s.layers.len() + 1),
            })
        };
        let create_comment = || Command::CreateComment {
            id: fresh,
            anchor: Anchor::Voxel { at: from },
            layer: Some(layer.id),
            message,
            text: name.clone(),
        };
        let save_version = || Command::SaveVersion {
            id: fresh,
            name: name.clone(),
            parent: e.document().versions.keys().next_back().copied(),
        };
        // Unlock/show only where required, as part of the same atomic edit. This
        // also exercises reversal of edits to initially hidden/locked layers.
        let editable = |ids: &[Uuid], command: Command| {
            let mut commands: Vec<_> = s
                .layers
                .iter()
                .filter(|l| ids.contains(&l.id) && (l.locked || !l.visible))
                .map(|l| {
                    Command::Layer(LayerEdit::Update {
                        id: l.id,
                        name: None,
                        visible: Some(true),
                        locked: Some(false),
                    })
                })
                .collect();
            if commands.is_empty() {
                command
            } else {
                commands.push(command);
                Command::Batch(commands)
            }
        };
        match self.kind {
            Kind::Set => editable(
                &[layer.id],
                Command::SetVoxels {
                    layer: layer.id,
                    voxels: vec![Voxel { at: from, slot }, Voxel { at: to, slot }],
                    mode: self.mode,
                    mirror,
                },
            ),
            Kind::Erase => editable(
                &[layer.id],
                Command::EraseRegion {
                    layer: layer.id,
                    region,
                    mirror,
                },
            ),
            Kind::Fill => editable(
                &[layer.id],
                Command::FloodFill {
                    layer: layer.id,
                    seed: from,
                    slot,
                    region: self.flags[0].then_some(region),
                    cap: bounds.volume().unwrap(),
                    mirror,
                },
            ),
            Kind::Box | Kind::Ellipsoid | Kind::Cylinder | Kind::Line => {
                let center2 = std::array::from_fn(|a| bounds.min[a] + bounds.max[a]);
                let radii2 = std::array::from_fn(|a| bounds.max[a] - bounds.min[a]);
                let hollow = self.flags[0];
                let shape = match self.kind {
                    Kind::Box => Shape::Box { region, hollow },
                    Kind::Ellipsoid => Shape::Ellipsoid {
                        center2,
                        radii2,
                        hollow,
                    },
                    Kind::Cylinder => Shape::Cylinder {
                        axis,
                        center2,
                        radii2,
                        range: [bounds.min[axis.index()], bounds.max[axis.index()]],
                        hollow,
                    },
                    Kind::Line => Shape::Line { from, to },
                    _ => unreachable!(),
                };
                editable(
                    &[layer.id],
                    Command::Primitive {
                        layer: layer.id,
                        shape,
                        mode: self.mode,
                        slot: (self.mode != Mode::Erase).then_some(slot),
                        mirror,
                    },
                )
            }
            Kind::Transform => {
                let destination = s.layers[(self.pick + 1) % s.layers.len()].id;
                let move_cells = self.flags[0];
                editable(
                    &[layer.id, destination],
                    Command::TransformRegion(Transform {
                        source: layer.id,
                        destination,
                        region: bounds,
                        move_cells,
                        translate: [0; 3],
                        pivot2: std::array::from_fn(|a| bounds.min[a] + bounds.max[a]),
                        reflect: self.flags,
                        rotate: Some((axis, (self.pick % 4) as u8)),
                        collision: if !move_cells && self.flags[1] {
                            Collision::Skip
                        } else {
                            Collision::Overwrite
                        },
                    }),
                )
            }
            Kind::AddColor => Command::Palette(PaletteEdit::Add {
                slot: None,
                color: self.color.clone(),
            }),
            Kind::SetColor => Command::Palette(PaletteEdit::Set {
                slot,
                color: self.color.clone(),
            }),
            Kind::RemoveColor if s.palette.order.len() > 1 => {
                let replacement = s
                    .palette
                    .order
                    .iter()
                    .copied()
                    .find(|s| *s != slot)
                    .unwrap();
                editable(
                    &s.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
                    Command::Palette(PaletteEdit::Remove {
                        slot,
                        replacement: Some(replacement),
                    }),
                )
            }
            Kind::RemoveColor => Command::Palette(PaletteEdit::Add {
                slot: None,
                color: self.color.clone(),
            }),
            Kind::ReorderColors => {
                let mut order = s.palette.order.clone();
                order.reverse();
                Command::Palette(PaletteEdit::Reorder(order))
            }
            Kind::CreateLayer => create_layer(),
            Kind::UpdateLayer => Command::Layer(LayerEdit::Update {
                id: layer.id,
                name: Some(name),
                visible: Some(self.flags[0]),
                locked: Some(self.flags[1]),
            }),
            Kind::DuplicateLayer => Command::Layer(LayerEdit::Duplicate {
                source: layer.id,
                id: fresh,
                name,
            }),
            Kind::ReorderLayers => Command::Layer(LayerEdit::Reorder(
                s.layers.iter().rev().map(|l| l.id).collect(),
            )),
            Kind::DeleteLayer if s.layers.len() > 1 => {
                editable(&[layer.id], Command::Layer(LayerEdit::Delete(layer.id)))
            }
            Kind::DeleteLayer => create_layer(),
            Kind::Rename => Command::EditDocument {
                name: Some(name),
                bounds: None,
            },
            Kind::Resize => Command::EditDocument {
                name: None,
                bounds: Some(Region {
                    min: bounds.min.map(|v| v - 1),
                    max: bounds.max.map(|v| v + 1),
                }),
            },
            Kind::CreateComment => create_comment(),
            Kind::Reply | Kind::Reanchor | Kind::Resolve => {
                if s.comments.is_empty() {
                    return create_comment();
                }
                let id = s.comments[self.pick % s.comments.len()].id;
                match self.kind {
                    Kind::Reply => Command::Reply {
                        id,
                        message,
                        text: name,
                    },
                    Kind::Reanchor => Command::Reanchor {
                        id,
                        anchor: Anchor::Region {
                            min: region.min,
                            max: region.max,
                        },
                        layer: None,
                    },
                    Kind::Resolve => Command::Resolve {
                        id,
                        resolved: self.flags[0],
                    },
                    _ => unreachable!(),
                }
            }
            Kind::SaveVersion => save_version(),
            Kind::RestoreVersion => {
                if s.layers.iter().any(|l| l.locked) {
                    return Command::Batch(
                        s.layers
                            .iter()
                            .map(|l| {
                                Command::Layer(LayerEdit::Update {
                                    id: l.id,
                                    name: None,
                                    visible: None,
                                    locked: Some(false),
                                })
                            })
                            .collect(),
                    );
                }
                match e.document().versions.keys().next() {
                    Some(id) => Command::RestoreVersion {
                        id: *id,
                        expected_head: e.head(),
                    },
                    None => save_version(),
                }
            }
            Kind::Batch => Command::Batch(vec![
                create_layer(),
                Command::Palette(PaletteEdit::Set {
                    slot,
                    color: self.color.clone(),
                }),
                Command::SetVoxels {
                    layer: fresh,
                    voxels: vec![Voxel { at: from, slot }],
                    mode: Mode::Place,
                    mirror,
                },
            ]),
        }
    }
}
