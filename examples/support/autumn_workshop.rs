//! Original MIT-licensed Autumn Workshop, staged through real core edits.
use std::collections::BTreeMap;
use voxelite_core::*;
use voxelite_format::{BuildHistory, BuildStep, FormatResult, Project};

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x5f26_1025_0000_4000_8000_0000_0000_0000 + n)
}
struct Builder {
    cells: BTreeMap<Pos, u16>,
}
impl Builder {
    fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }
    fn cell(&mut self, p: Pos, s: u16) {
        if s == 0 {
            self.cells.remove(&p);
        } else {
            self.cells.insert(p, s);
        }
    }
    fn box_at(&mut self, min: Pos, max: Pos, s: u16) {
        for p in (Region { min, max }).cells() {
            self.cell(p, s);
        }
    }
    fn blob(&mut self, c: Pos, r: Pos, colors: &[u16]) {
        for x in c[0] - r[0]..=c[0] + r[0] {
            for y in c[1] - r[1]..=c[1] + r[1] {
                for z in c[2] - r[2]..=c[2] + r[2] {
                    let p = [x, y, z];
                    // Clear the ellipsoid denominators for platform-independent occupancy.
                    let r2 = r.map(|radius| i64::from(radius).pow(2));
                    let denominator: i64 = r2.iter().product();
                    let distance: i64 = (0..3)
                        .map(|a| i64::from(p[a] - c[a]).pow(2) * (denominator / r2[a]))
                        .sum();
                    if distance <= denominator {
                        let h = hash(p);
                        let light = usize::from(y > c[1]);
                        self.cell(
                            p,
                            colors[(h as usize % colors.len() + light) % colors.len()],
                        );
                    }
                }
            }
        }
    }
}
fn hash(p: Pos) -> u32 {
    let mut n = (p[0] as u32).wrapping_mul(374761393)
        ^ (p[1] as u32).wrapping_mul(668265263)
        ^ (p[2] as u32).wrapping_mul(2246822519);
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
// Scratch cells describe geometry only. flush() sends differences through Engine;
// no Document, history snapshot, or serialized voxel array is mutated directly.
impl Builder {
    fn flush(&self, build: &mut Build, layer: Uuid, label: &str, mode: Mode) -> Result<()> {
        let current = &build.engine.document().content.layer(layer)?.voxels;
        let mut changes: BTreeMap<_, _> = self
            .cells
            .iter()
            .filter(|(at, slot)| current.get(**at) != **slot)
            .map(|(at, slot)| (*at, *slot))
            .collect();
        for (at, _) in current.iter() {
            if !self.cells.contains_key(&at) {
                changes.insert(at, 0);
            }
        }
        if mode == Mode::Paint
            && changes
                .iter()
                .any(|(at, slot)| *slot == 0 || current.get(*at) == 0)
        {
            return Err(Error::Invalid(
                "paint pass must only recolor occupied cells".into(),
            ));
        }
        build.step(
            label,
            Command::SetVoxels {
                layer,
                voxels: changes
                    .into_iter()
                    .map(|(at, slot)| Voxel { at, slot })
                    .collect(),
                mode,
                mirror: Mirror::default(),
            },
        )
    }
}
struct Build {
    engine: Engine,
    history: BuildHistory,
}
impl Build {
    fn step(&mut self, label: &str, command: Command) -> Result<()> {
        let index = self.history.steps.len();
        let step = BuildStep {
            id: id(101 + index as u128),
            label: label.into(),
            origin: Origin::Agent,
            // A logical build clock: one minute per edit, no claimed wall time.
            elapsed_ms: index as u32 * 60_000,
            command,
        };
        let receipt = self.engine.execute(Transaction {
            expected_revision: self.engine.revision(),
            id: step.id,
            actor: Actor::new(step.origin),
            label: step.label.clone(),
            command: step.command.clone(),
        })?;
        if receipt.entry != Some(step.id) || !self.engine.initial_retained() {
            return Err(Error::Invalid(format!(
                "empty or evicted build step: {label}"
            )));
        }
        self.history.steps.push(step);
        Ok(())
    }
    fn layer(&mut self, layer: Uuid, name: &str) -> Result<()> {
        self.step(
            &format!("Create {name} layer"),
            Command::Layer(LayerEdit::Create {
                id: layer,
                name: name.into(),
                index: self.engine.document().content.layers.len(),
            }),
        )
    }
}

pub fn build() -> FormatResult<Project> {
    let mut build = Build {
        engine: Engine::new(Document::new(id(1), id(2)))?,
        history: BuildHistory {
            version: 1,
            base_layer: id(2),
            steps: vec![],
        },
    };
    build.step(
        "Set up Autumn Workshop",
        Command::EditDocument {
            name: Some("Autumn Workshop".into()),
            bounds: Some(Region::new([-40, 0, -40], [40, 64, 40])?),
        },
    )?;
    // Earth, turf, stone, timber, plaster, roof, amber, autumn leaves, garden.
    let palette = Palette::from_hex(
        "3a3033\n59423a\n775642\n987759\n526343\n718250\n97a164\n48505a\n727c80\na5aaa0\n4b3330\n785044\na57551\nd9c5a0\nf4dfb5\n263a44\n365564\n50737b\n769497\n392e30\nf0a750\nffd78b\n8a4636\nbc5c38\ndf7d3f\nf3a44d\nf4bf65\n304d42\n49745a\n75936a\nb55259\ne58b73\nd0d7c2\n9eae83\n223942\n456a78",
    )?;
    let mut colors = Vec::new();
    for swatch in &palette.slots {
        // Glazing starts muted; the final lighting pass warms it to the original colors.
        let color = match swatch.index {
            21 => Swatch::rgb(190, 140, 75),
            22 => Swatch::rgb(220, 185, 125),
            _ => swatch.color.clone(),
        };
        colors.push(Command::Palette(if swatch.index == 1 {
            PaletteEdit::Set { slot: 1, color }
        } else {
            PaletteEdit::Add {
                slot: Some(swatch.index),
                color,
            }
        }));
    }
    build.step("Mix earth, slate and autumn colors", Command::Batch(colors))?;
    build.step(
        "Name the landscape layer",
        Command::Layer(LayerEdit::Update {
            id: id(2),
            name: Some("Landscape".into()),
            visible: None,
            locked: None,
        }),
    )?;
    let mut ground = Builder::new();
    for (low, high, label) in [
        (2, 4, "Lay the chamfered earth base"),
        (4, 7, "Raise the island terrain"),
        (7, 8, "Cover the island with turf"),
    ] {
        for x in -31i32..=31 {
            for z in -27i32..=27 {
                // Rounded chamfered island, with a rough stone/earth cutaway.
                if x.abs() * x.abs() + z.abs() * z.abs() > 33 * 33 || x.abs() + z.abs() > 46 {
                    continue;
                }
                let base = 2 + (hash([x, 0, z]) % 2) as i32;
                for y in base.max(low)..high {
                    let s = if y == 7 {
                        6
                    } else if y == 6 {
                        5
                    } else if y == base {
                        2
                    } else if hash([x, y, z]) % 9 < 2 {
                        4
                    } else {
                        3
                    };
                    ground.cell([x, y, z], s);
                }
            }
        }
        ground.flush(&mut build, id(2), label, Mode::Place)?;
    }
    for (at, slot) in &mut ground.cells {
        if at[1] == 7 && hash(*at) % 11 < 2 {
            *slot = 7;
        }
    }
    ground.flush(
        &mut build,
        id(2),
        "Paint sunlit patches in the meadow",
        Mode::Paint,
    )?;
    // Meandering gravel path with inset cobblestones and grass interruptions.
    for z in 10..27 {
        let center = if z < 17 { -2 } else { (z - 17) / 2 - 2 };
        for x in center - 3..=center + 3 {
            if ground.cells.contains_key(&[x, 7, z]) {
                ground.cell(
                    [x, 7, z],
                    if hash([x, 7, z]).is_multiple_of(4) {
                        4
                    } else {
                        9
                    },
                );
                if (x + z).rem_euclid(3) != 0 {
                    ground.cell(
                        [x, 8, z],
                        if hash([x, 8, z]).is_multiple_of(3) {
                            10
                        } else {
                            9
                        },
                    );
                }
            }
        }
    }
    ground.flush(
        &mut build,
        id(2),
        "Lay the winding cobblestone path",
        Mode::Place,
    )?;
    // Tiny stream along the back of the island with a stone bank.
    for x in 8..27 {
        for z in -23..-18 {
            if ground.cells.contains_key(&[x, 7, z]) {
                ground.cell([x, 7, z], if (x + z) % 4 == 0 { 36 } else { 35 });
            }
        }
    }
    ground.flush(
        &mut build,
        id(2),
        "Paint the stream behind the cottage",
        Mode::Paint,
    )?;
    build.layer(id(3), "Cottage")?;
    let mut house = Builder::new();
    house.box_at([-13, 8, -11], [11, 10, 12], 8);
    house.box_at([-12, 9, -10], [10, 11, 11], 10);
    house.flush(
        &mut build,
        id(3),
        "Lay the cottage stone foundation",
        Mode::Place,
    )?;
    house.box_at([-12, 10, -10], [10, 24, 11], 14);
    house.flush(&mut build, id(3), "Raise the plaster walls", Mode::Place)?;
    // Interior void, solid framed walls and tall front gable.
    house.box_at([-10, 11, -8], [8, 23, 9], 0);
    house.flush(
        &mut build,
        id(3),
        "Hollow out the cottage interior",
        Mode::Place,
    )?;
    for x in -12i32..10 {
        let top = 34 - (x + 1).abs();
        for y in 24..top {
            for z in [-10, 10] {
                house.cell([x, y, z], if (x + y + z) % 8 == 0 { 15 } else { 14 });
            }
        }
    }
    house.flush(
        &mut build,
        id(3),
        "Raise the front and rear gables",
        Mode::Place,
    )?;
    // Exposed oak frame, including horizontal rails and corner posts.
    for x in [-12, -5, 3, 9] {
        house.box_at([x, 10, 10], [x + 1, 24, 12], 11);
        house.box_at([x, 10, -11], [x + 1, 24, -10], 11);
    }
    for z in [-10, -3, 4, 10] {
        house.box_at([9, 10, z], [11, 24, z + 1], 11);
        house.box_at([-13, 10, z], [-12, 24, z + 1], 11);
    }
    house.flush(&mut build, id(3), "Set the oak corner posts", Mode::Place)?;
    for y in [11, 22, 24] {
        house.box_at([-13, y, 10], [11, y + 1, 12], 12);
        house.box_at([9, y, -11], [11, y + 1, 11], 12);
    }
    house.box_at([-2, 23, 10], [-1, 34, 12], 11);
    for i in 0..8 {
        house.cell([-10 + i, 24 + i, 11], 12);
        house.cell([7 - i, 24 + i, 11], 12);
    }
    house.flush(
        &mut build,
        id(3),
        "Join the timber rails and gable frame",
        Mode::Place,
    )?;
    // Two amber windows facing front, with shutters and planter boxes.
    for x in [-9, 4] {
        house.box_at([x, 15, 10], [x + 4, 20, 12], 20);
        house.box_at([x + 1, 16, 11], [x + 3, 19, 12], 22);
        house.box_at([x - 1, 14, 11], [x + 5, 15, 13], 13);
        house.box_at([x - 1, 20, 11], [x + 5, 21, 12], 13);
        house.box_at([x - 1, 15, 12], [x, 20, 13], 17);
        house.box_at([x + 4, 15, 12], [x + 5, 20, 13], 17);
        house.box_at([x, 12, 12], [x + 4, 14, 14], 12);
        for dx in 0..4 {
            house.cell([x + dx, 14, 13], 29);
            if dx % 2 == 0 {
                house.cell([x + dx, 15, 13], 31);
            }
        }
        house.flush(
            &mut build,
            id(3),
            if x < 0 {
                "Fit the left window and planter"
            } else {
                "Fit the right window and planter"
            },
            Mode::Place,
        )?;
    }
    // Recessed door, hinges, brass handle, stone steps and timber lintel.
    house.box_at([-4, 10, 10], [1, 19, 12], 20);
    house.box_at([-3, 10, 11], [0, 18, 12], 12);
    for y in 11..18 {
        house.cell([-2, y, 11], 13);
    }
    house.cell([-1, 14, 12], 21);
    house.box_at([-5, 19, 11], [2, 21, 13], 11);
    house.box_at([-5, 8, 12], [2, 10, 15], 9);
    house.box_at([-4, 9, 12], [1, 10, 14], 10);
    house.flush(
        &mut build,
        id(3),
        "Hang the door and build the front steps",
        Mode::Place,
    )?;
    // Side window on +X wall.
    house.box_at([9, 15, -6], [11, 20, -1], 20);
    house.box_at([10, 16, -5], [11, 19, -2], 22);
    house.box_at([11, 15, -7], [12, 20, -6], 17);
    house.box_at([11, 15, -1], [12, 20, 0], 17);
    house.box_at([10, 14, -7], [12, 15, 0], 13);
    house.flush(
        &mut build,
        id(3),
        "Fit the east window and shutters",
        Mode::Place,
    )?;
    // Deep slate eaves; individual shingles form staggered seams.
    let mut shingles = BTreeMap::new();
    for x in -15i32..=12 {
        let y = 35 - (x + 1).abs();
        for z in -13..=13 {
            house.cell([x, y - 1, z], 11);
            let seam = (z + (x.abs() % 2) * 2).rem_euclid(5) == 0;
            let s = if seam {
                16
            } else {
                match hash([x, y, z]) % 9 {
                    0 => 19,
                    1..=3 => 18,
                    _ => 17,
                }
            };
            house.cell([x, y, z], 17);
            shingles.insert([x, y, z], s);
            if z == -13 || z == 13 {
                shingles.insert([x, y, z], 18);
            }
        }
    }
    house.flush(
        &mut build,
        id(3),
        "Lay slate over the timber eaves",
        Mode::Place,
    )?;
    for (at, slot) in shingles {
        house.cell(at, slot);
    }
    house.flush(
        &mut build,
        id(3),
        "Paint staggered slate seams and highlights",
        Mode::Paint,
    )?;
    house.box_at([-2, 35, -14], [0, 36, 15], 19);
    house.flush(&mut build, id(3), "Cap the roof ridge", Mode::Place)?;
    // Brick chimney, stepped crown and dark opening.
    for y in 28..42 {
        for x in 4..8 {
            for z in -7..-3 {
                house.cell(
                    [x, y, z],
                    if (x + z + y).rem_euclid(4) == 0 {
                        24
                    } else {
                        23
                    },
                );
            }
        }
    }
    house.box_at([3, 40, -8], [9, 42, -2], 8);
    house.box_at([4, 42, -7], [8, 43, -3], 10);
    house.box_at([5, 42, -6], [7, 43, -4], 20);
    house.flush(
        &mut build,
        id(3),
        "Build the brick chimney and stone crown",
        Mode::Place,
    )?;
    // Small round attic opening, indicated with a deep inset and gold center.
    house.box_at([-4, 26, 11], [1, 30, 12], 11);
    house.box_at([-3, 27, 12], [0, 29, 13], 21);
    house.cell([-2, 28, 12], 22);
    house.flush(
        &mut build,
        id(3),
        "Add the amber attic opening",
        Mode::Place,
    )?;
    build.layer(id(4), "Garden")?;
    let mut plants = Builder::new();
    // A sculpted, asymmetrical autumn crown, with a visible forked trunk.
    plants.box_at([-24, 8, 4], [-21, 26, 7], 11);
    plants.box_at([-23, 8, 6], [-21, 23, 8], 12);
    plants.flush(
        &mut build,
        id(4),
        "Plant the western oak trunk",
        Mode::Place,
    )?;
    for y in 18..30 {
        let drift = (y - 18) / 3;
        plants.box_at([-23 - drift, y, 5], [-21 - drift, y + 1, 7], 12);
    }
    for y in 20..30 {
        let drift = (y - 20) / 3;
        plants.box_at([-22 + drift, y, 5], [-20 + drift, y + 1, 7], 13);
    }
    plants.flush(&mut build, id(4), "Fork the oak branches", Mode::Place)?;
    for (label, c, r) in [
        ("Build the western oak crown", [-25, 31, 5], [7, 7, 7]),
        ("Build the eastern oak crown", [-18, 30, 5], [6, 6, 6]),
        ("Lift the upper oak canopy", [-23, 37, 4], [6, 5, 6]),
        ("Round out the outer oak canopy", [-28, 31, 10], [5, 5, 5]),
        ("Fill the rear oak canopy", [-19, 34, 0], [5, 5, 5]),
    ] {
        plants.blob(c, r, &[24, 25, 25, 26, 26, 27]);
        plants.flush(&mut build, id(4), label, Mode::Place)?;
    }
    // Clusters of sage bushes and garden blooms around the stone foundation.
    for (c, r) in [
        ([13, 10, 9], [3, 3, 3]),
        ([-15, 9, -8], [3, 2, 4]),
        ([14, 9, -10], [4, 2, 3]),
        ([23, 9, 15], [3, 2, 3]),
    ] {
        plants.blob(c, r, &[28, 29, 29, 30]);
    }
    plants.flush(
        &mut build,
        id(4),
        "Plant sage bushes around the foundation",
        Mode::Place,
    )?;
    for x in -29..30 {
        for z in -25..25 {
            let h = hash([x, 8, z]);
            if h.is_multiple_of(77)
                && x * x + z * z < 30 * 30
                && (!(-16..15).contains(&x) || !(-14..16).contains(&z))
            {
                plants.cell([x, 8, z], 5);
                plants.cell([x, 9, z], if h.is_multiple_of(3) { 32 } else { 27 });
            }
        }
    }
    plants.flush(
        &mut build,
        id(4),
        "Scatter wildflowers around the meadow",
        Mode::Place,
    )?;
    // Windfall leaves gather beside the trunk and along the path.
    for x in -29..-12 {
        for z in 2..20 {
            if hash([x, 8, z]).is_multiple_of(8) {
                plants.cell([x, 8, z], if (x + z) % 2 == 0 { 25 } else { 26 });
            }
        }
    }
    plants.flush(
        &mut build,
        id(4),
        "Gather fallen leaves beside the oak",
        Mode::Place,
    )?;
    build.layer(id(5), "Workshop details")?;
    let mut props = Builder::new();
    // Split-rail fence with chamfered-looking post caps.
    for z in [-16, -10, -4, 2, 8] {
        props.box_at([23, 8, z], [25, 14, z + 2], 12);
        props.box_at([23, 14, z], [25, 15, z + 2], 13);
    }
    for y in [10, 12] {
        props.box_at([24, y, -16], [25, y + 1, 10], 13);
    }
    props.flush(
        &mut build,
        id(5),
        "Build the eastern split-rail fence",
        Mode::Place,
    )?;
    for x in [12, 18, 24] {
        props.box_at([x, 8, -16], [x + 2, 14, -14], 12);
    }
    for y in [10, 12] {
        props.box_at([12, y, -15], [25, y + 1, -14], 13);
    }
    props.flush(
        &mut build,
        id(5),
        "Return the fence along the north garden",
        Mode::Place,
    )?;
    // Vegetable beds, orange pumpkins and neatly stacked firewood.
    props.box_at([14, 8, -7], [21, 9, 1], 2);
    for z in [-6, -3, 0] {
        for x in [15, 18, 20] {
            props.cell([x, 9, z], 29);
            props.cell([x, 10, z], 30);
        }
    }
    props.flush(&mut build, id(5), "Plant the vegetable beds", Mode::Place)?;
    for (c, r) in [([13, 9, 15], [2, 2, 2]), ([17, 9, 14], [2, 2, 2])] {
        props.blob(c, r, &[24, 25, 26]);
        props.cell([c[0], 12, c[2]], 11);
    }
    props.flush(
        &mut build,
        id(5),
        "Place pumpkins by the front garden",
        Mode::Place,
    )?;
    for y in [9, 11] {
        for z in [2, 5, 8] {
            props.box_at([11, y, z], [15, y + 2, z + 2], 11);
            props.box_at([14, y, z], [15, y + 2, z + 2], 13);
        }
    }
    props.flush(
        &mut build,
        id(5),
        "Stack firewood against the east wall",
        Mode::Place,
    )?;
    // Lantern post with honey glass, dark cage and pitched cap.
    props.box_at([11, 8, 20], [12, 18, 21], 11);
    props.box_at([10, 17, 20], [14, 18, 21], 11);
    props.box_at([12, 14, 19], [15, 17, 22], 21);
    props.box_at([13, 15, 20], [14, 17, 22], 22);
    for x in [12, 14] {
        for z in [19, 21] {
            props.box_at([x, 14, z], [x + 1, 18, z + 1], 16);
        }
    }
    props.box_at([11, 17, 18], [16, 18, 23], 16);
    props.box_at([12, 18, 19], [15, 19, 22], 17);
    props.cell([13, 19, 20], 19);
    props.flush(
        &mut build,
        id(5),
        "Hang the lantern beside the path",
        Mode::Place,
    )?;
    // Bench on the far right and a few stepping stones beside the stream.
    props.box_at([16, 11, 13], [22, 12, 15], 13);
    for x in [16, 20] {
        props.box_at([x, 8, 13], [x + 1, 11, 15], 11);
    }
    props.flush(
        &mut build,
        id(5),
        "Set a bench in the front garden",
        Mode::Place,
    )?;
    for x in [10, 14, 18, 22] {
        props.box_at([x, 8, -21], [x + 2, 9, -19], 10);
    }
    props.flush(
        &mut build,
        id(5),
        "Lay stepping stones across the stream",
        Mode::Place,
    )?;
    build.step(
        "Warm the window and lantern glass",
        Command::Batch(
            [21, 22]
                .into_iter()
                .map(|slot| {
                    Command::Palette(PaletteEdit::Set {
                        slot,
                        color: palette.get(slot).expect("defined glazing").clone(),
                    })
                })
                .collect(),
        ),
    )?;
    build.step(
        "Name the autumn garden layer",
        Command::Layer(LayerEdit::Update {
            id: id(4),
            name: Some("Autumn garden".into()),
            visible: None,
            locked: None,
        }),
    )?;
    build.step("Review the finished composition", Command::CreateComment {
        id: id(20), anchor: Anchor::Voxel { at: [-2, 35, 0] }, layer: Some(id(3)), message: id(21),
        text: "Slate roof, amber windows, and an autumn crown. Check the silhouette from all six views.".into(),
    })?;
    build.step(
        "Save Autumn Workshop",
        Command::SaveVersion {
            id: id(30),
            name: "Autumn Workshop · finished".into(),
            parent: None,
        },
    )?;
    let mut project = Project::new(build.engine.document().clone())?;
    project.set_build_history(build.history)?;
    Ok(project)
}
