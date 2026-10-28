use super::*;
use quantum_plugin_api::{Mode, PaletteEdit, Region, Shape, Swatch, Uuid, Voxel};
use quantum_plugin_host::Host;

quantum_plugin_host::plugin_conformance_test!(conforms_to_plugin_contract, || Box::new(
    MirrorPlugin
));

#[test]
fn declares_plugin_specific_capabilities() {
    let mut host = Host::new(vec![Box::new(MirrorPlugin)]);
    assert_eq!(host.registry().command_transforms().count(), 1);
    assert_eq!(host.registry().settings().count(), 3);
    assert!(host.has_command_transform(TRANSFORM_ID).unwrap());
    assert!(
        !host
            .has_command_transform("dev.quantum.mirror.unknown")
            .unwrap()
    );
    for key in AXIS_SETTINGS {
        assert_eq!(host.setting(key), Some(&SettingValue::Bool(false)));
    }
    let commands: Vec<_> = host
        .registry()
        .commands()
        .map(|(_, c)| (c.id.as_str(), c.default_keybinding.as_deref(), c.order))
        .collect();
    assert_eq!(
        commands,
        [
            ("dev.quantum.mirror.mirror_x", Some("shift-x"), 230),
            ("dev.quantum.mirror.mirror_y", Some("shift-y"), 240),
            ("dev.quantum.mirror.mirror_z", Some("shift-z"), 250),
        ]
    );
}

fn commands(mirror: Mirror) -> Vec<Command> {
    let layer = Uuid::nil();
    let region = Region::new([1; 3], [3; 3]).unwrap();
    let mut edits: Vec<_> = [Mode::Place, Mode::Paint, Mode::Erase]
        .into_iter()
        .map(|mode| Command::SetVoxels {
            layer,
            voxels: vec![Voxel {
                at: [1; 3],
                slot: 2,
            }],
            mode,
            mirror,
        })
        .collect();
    edits.push(Command::Batch(vec![
        Command::FloodFill {
            layer,
            seed: [1; 3],
            slot: 2,
            region: None,
            cap: 1024,
            mirror,
        },
        Command::Primitive {
            layer,
            shape: Shape::Box {
                region,
                hollow: false,
            },
            mode: Mode::Place,
            slot: Some(2),
            mirror,
        },
    ]));
    vec![
        Command::Palette(PaletteEdit::Add {
            slot: Some(2),
            color: Swatch::rgb(1, 2, 3),
        }),
        Command::Batch(edits),
        // The original tool transform did not rewrite EraseRegion or other commands.
        Command::EraseRegion {
            layer,
            region,
            mirror: Mirror {
                plane2: [Some(7); 3],
            },
        },
        Command::EditDocument {
            name: Some("Unchanged".into()),
            bounds: Some(region),
        },
    ]
}

#[test]
fn transforms_nested_tool_commands_only_for_every_axis_combination() {
    let mut host = Host::new(vec![Box::new(MirrorPlugin)]);
    for (min, max) in [
        ([0; 3], [32; 3]),
        ([-4; 3], [4; 3]),
        ([-7, 10, -3], [2, 19, 6]),
    ] {
        let bounds = Region::new(min, max).unwrap();
        for mask in 0..8 {
            for (axis, key) in AXIS_SETTINGS.into_iter().enumerate() {
                host.set_setting(key, SettingValue::Bool(mask & (1 << axis) != 0))
                    .unwrap();
            }
            let expected = Mirror {
                plane2: std::array::from_fn(|axis| {
                    (mask & (1 << axis) != 0).then_some(min[axis] + max[axis])
                }),
            };
            // Existing planes are replaced even when every plugin axis is off.
            let input = commands(Mirror {
                plane2: [Some(9); 3],
            });
            assert_eq!(
                host.transform_commands(bounds, input).unwrap(),
                commands(expected)
            );
            assert!(host.transform_commands(bounds, vec![]).unwrap().is_empty());
        }
    }
}

#[test]
fn disabled_mirror_is_inert_and_axis_state_survives_reactivation() {
    let mut host = Host::new(vec![Box::new(MirrorPlugin)]);
    host.set_setting(AXIS_SETTINGS[0], SettingValue::Bool(true))
        .unwrap();
    let bounds = Region::new([-4; 3], [4; 3]).unwrap();
    let input = commands(Mirror::default());
    host.disable("dev.quantum.mirror").unwrap();
    assert!(host.registry().iter().next().is_none());
    assert!(
        !host
            .set_setting(AXIS_SETTINGS[0], SettingValue::Bool(false))
            .unwrap()
    );
    assert_eq!(
        host.transform_commands(bounds, input.clone()).unwrap(),
        input
    );
    host.enable("dev.quantum.mirror").unwrap();
    assert_eq!(
        host.transform_commands(bounds, input).unwrap(),
        commands(Mirror {
            plane2: [Some(0), None, None]
        })
    );
    let other = Host::new(vec![Box::new(MirrorPlugin)]);
    assert_eq!(
        other.setting(AXIS_SETTINGS[0]),
        Some(&SettingValue::Bool(false))
    );
}
