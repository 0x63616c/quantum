use super::*;

#[test]
fn legacy_enablement_and_setting_ids_resolve_to_current_declarations() {
    let saved = EnabledSet::from(vec![
        "dev.voxelite.tools".into(),
        "dev.quantum.tools".into(),
        "org.unknown".into(),
    ]);
    assert_eq!(saved.to_vec(), ["dev.quantum.tools", "org.unknown"]);
    assert!(saved.contains("dev.voxelite.tools"));
    assert_eq!(
        EnabledSet::from(BTreeSet::from(["dev.voxelite.tools".into()])).to_vec(),
        ["dev.quantum.tools"]
    );
    let mut plugin = TestPlugin::new("dev.quantum.tools");
    plugin.manifest.default_enabled = false;
    plugin
        .declarations
        .push(Contribution::Setting(SettingContribution {
            key: "dev.quantum.tools.setting".into(),
            label_key: "setting".into(),
            kind: SettingKind::Bool,
            default: SettingValue::Bool(false),
        }));
    let mut host = Host::with_enabled(vec![Box::new(plugin)], saved);
    assert_eq!(host.state("dev.quantum.tools"), Some(&PluginState::Enabled));
    assert!(
        host.set_setting("dev.voxelite.tools.setting", SettingValue::Bool(true))
            .unwrap()
    );
    assert_eq!(
        host.setting("dev.quantum.tools.setting"),
        Some(&SettingValue::Bool(true))
    );
    assert_eq!(
        host.setting("dev.voxelite.tools.setting"),
        Some(&SettingValue::Bool(true))
    );
    assert_eq!(host.enabled_set().to_vec(), ["dev.quantum.tools"]);
}
