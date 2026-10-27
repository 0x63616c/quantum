use super::*;
use voxelite_core::{Actor, Document, Engine, Origin};
use voxelite_format::{Project, from_bytes};

const OWNER: &str = "dev.example.data";
const RAW: &str = r#"{ "payload":{"count":2}, "schema_version":1, "encoding":"json" }"#;

struct DataPlugin {
    id: &'static str,
    migrations: Vec<DataMigration>,
    reject: bool,
    panic: bool,
}
impl Default for DataPlugin {
    fn default() -> Self {
        Self {
            id: OWNER,
            migrations: vec![
                DataMigration {
                    from_version: 1,
                    to_version: 2,
                    migrate: one_to_two,
                },
                DataMigration {
                    from_version: 2,
                    to_version: 3,
                    migrate: two_to_three,
                },
            ],
            reject: false,
            panic: false,
        }
    }
}
impl Plugin for DataPlugin {
    fn manifest(&self) -> Manifest {
        TestPlugin::new(self.id).manifest
    }
    fn contribute(&self, registrar: &mut Registrar) {
        registrar.data_section(DataSectionContribution {
            section_name: "state".into(),
            schema_version: 3,
        });
    }
    fn data_migrations(&self, _: &str) -> Vec<DataMigration> {
        assert!(!self.panic, "migration binding panic");
        self.migrations.clone()
    }
    fn load_data(&self, cx: &mut dyn DataCx) -> DataResult<()> {
        if let Some(section) = cx.read("state")? {
            assert_eq!(section.schema_version(), 3);
            if self.reject {
                return Err(DataError("corrupt count".into()));
            }
        }
        Ok(())
    }
}
fn one_to_two(section: &DataSection) -> DataResult<DataSection> {
    let mut value = section.json_payload()?;
    value["count"] = json::json!(value["count"].as_i64().unwrap() + 10);
    Ok(DataSection::json(2, &value)?)
}
fn two_to_three(section: &DataSection) -> DataResult<DataSection> {
    let value = section.json_payload()?;
    Ok(DataSection::bytes(
        3,
        &[value["count"].as_u64().unwrap() as u8],
    )?)
}
fn engine() -> Engine {
    let mut document = Document::default();
    document
        .extensions
        .insert(OWNER, "state", DataSection::from_raw(RAW).unwrap())
        .unwrap();
    document
        .extensions
        .insert(OWNER, "unknown", DataSection::from_raw(RAW).unwrap())
        .unwrap();
    document
        .extensions
        .insert("org.absent", "state", DataSection::from_raw(RAW).unwrap())
        .unwrap();
    Engine::new(document).unwrap()
}
fn host(plugin: DataPlugin) -> Host {
    Host::new(vec![Box::new(plugin)])
}
fn saved(engine: &Engine) -> Vec<u8> {
    Project::new(engine.document().clone())
        .unwrap()
        .to_bytes()
        .unwrap()
}

#[test]
fn migration_chain_loads_atomically_is_undoable_and_preserves_unowned_data() {
    let mut host = host(DataPlugin::default());
    let mut engine = engine();
    let before = saved(&engine);
    host.load_data(&mut engine).unwrap();
    let data = &engine.document().extensions;
    assert_eq!(
        data.get(OWNER, "state").unwrap().bytes_payload().unwrap(),
        [12]
    );
    assert_eq!(data.get(OWNER, "unknown").unwrap().raw_json(), RAW);
    assert_eq!(data.get("org.absent", "state").unwrap().raw_json(), RAW);
    assert!(engine.is_dirty());
    assert_eq!(engine.history().count(), 1);
    assert!(engine.history().next().unwrap().changes.metadata_changed);
    let after = saved(&engine);
    assert_eq!(from_bytes(&after).unwrap().to_bytes().unwrap(), after);
    let revision = engine.revision();
    host.load_data(&mut engine).unwrap();
    assert_eq!(engine.revision(), revision);
    let head = engine.head().unwrap();
    let receipt = engine.undo(engine.revision(), head).unwrap();
    assert!(receipt.changes.metadata_changed);
    assert_eq!(saved(&engine), before);
    engine.redo(engine.revision(), head).unwrap();
    assert_eq!(saved(&engine), after);
    engine
        .revert(
            engine.revision(),
            head,
            Uuid::new_v4(),
            Actor::new(Origin::Human),
        )
        .unwrap();
    assert_eq!(saved(&engine), before);
}

#[test]
fn disabled_and_absent_plugins_never_bind_or_rewrite_data() {
    let mut host = host(DataPlugin {
        panic: true,
        ..Default::default()
    });
    host.disable(OWNER).unwrap();
    let mut engine = engine();
    let before = saved(&engine);
    host.load_data(&mut engine).unwrap();
    assert_eq!(saved(&engine), before);
    assert!(!engine.is_dirty());
    assert!(
        host.with_data(OWNER, &mut engine, |cx| cx.write("state", None))
            .is_err()
    );
    Host::default().load_data(&mut engine).unwrap();
    assert_eq!(saved(&engine), before);
}

#[test]
fn scoped_access_and_schema_checks_cannot_modify_other_sections() {
    let mut host = host(DataPlugin::default());
    host.register(vec![Box::new(DataPlugin {
        id: "org.second",
        ..Default::default()
    })]);
    assert_eq!(host.registry().data_sections().count(), 2);
    let mut engine = engine();
    host.load_data(&mut engine).unwrap();
    let before = saved(&engine);
    for name in ["unknown", "org.absent/state", "extensions", "palette"] {
        assert!(
            host.with_data(OWNER, &mut engine, |cx| {
                cx.read(name)?;
                Ok(())
            })
            .is_err()
        );
        assert!(
            host.with_data(OWNER, &mut engine, |cx| cx.write(name, None))
                .is_err()
        );
    }
    assert!(
        host.with_data(OWNER, &mut engine, |cx| cx
            .write("state", Some(DataSection::json(1, &Value::Null)?)))
            .is_err()
    );
    assert_eq!(saved(&engine), before);
    host.with_data("org.second", &mut engine, |cx| {
        cx.write("state", Some(DataSection::json(3, &Value::Bool(true))?))
    })
    .unwrap();
    assert_eq!(
        engine
            .document()
            .extensions
            .get(OWNER, "state")
            .unwrap()
            .bytes_payload()
            .unwrap(),
        [12]
    );
    host.with_data(OWNER, &mut engine, |cx| cx.write("state", None))
        .unwrap();
    assert!(engine.document().extensions.get(OWNER, "state").is_none());
    assert!(
        engine
            .document()
            .extensions
            .get("org.second", "state")
            .is_some()
    );
}

#[test]
fn missing_ambiguous_backward_wrong_version_and_failed_migrations_roll_back() {
    let default = DataPlugin::default().migrations;
    let cases = [
        vec![],
        vec![default[0]],
        vec![default[0], default[0]],
        vec![DataMigration {
            from_version: 1,
            to_version: 1,
            migrate: one_to_two,
        }],
        vec![DataMigration {
            from_version: 1,
            to_version: 3,
            migrate: one_to_two,
        }],
        vec![
            default[0],
            DataMigration {
                from_version: 2,
                to_version: 3,
                migrate: |_| Err(DataError("corrupt payload".into())),
            },
        ],
    ];
    for migrations in cases {
        let mut host = host(DataPlugin {
            migrations,
            ..Default::default()
        });
        let mut engine = engine();
        let before = saved(&engine);
        let error = host.load_data(&mut engine).unwrap_err().to_string();
        assert!(error.contains("dev.example.data/state"), "{error}");
        assert_eq!(saved(&engine), before);
        assert_eq!(engine.revision(), 0);
    }
}

#[test]
fn corrupt_owner_and_panics_leave_rest_of_document_and_history_intact() {
    for plugin in [
        DataPlugin {
            reject: true,
            ..Default::default()
        },
        DataPlugin {
            panic: true,
            ..Default::default()
        },
        DataPlugin {
            migrations: vec![DataMigration {
                from_version: 1,
                to_version: 3,
                migrate: |_| panic!("migration panic"),
            }],
            ..Default::default()
        },
    ] {
        let mut host = host(plugin);
        let mut engine = engine();
        let before = saved(&engine);
        assert!(host.load_data(&mut engine).is_err());
        assert_eq!(saved(&engine), before);
        assert_eq!(engine.revision(), 0);
        assert_eq!(engine.history().count(), 0);
    }
    let mut host = host(DataPlugin::default());
    let mut engine = engine();
    let before = saved(&engine);
    let result: Result<(), _> = host.with_data(OWNER, &mut engine, |cx| {
        cx.write("state", None)?;
        panic!("after write")
    });
    assert!(matches!(
        result,
        Err(HostError::Faulted {
            phase: Phase::Data,
            ..
        })
    ));
    assert_eq!(saved(&engine), before);
    assert_eq!(host.registry().data_sections().count(), 0);
}

#[test]
fn later_owner_failure_rolls_back_earlier_migrations_and_newer_schemas_reject() {
    let mut host = host(DataPlugin::default());
    host.register(vec![Box::new(DataPlugin {
        id: "org.second",
        reject: true,
        ..Default::default()
    })]);
    let mut document = engine().document().clone();
    document
        .extensions
        .insert("org.second", "state", DataSection::from_raw(RAW).unwrap())
        .unwrap();
    let mut engine = Engine::new(document).unwrap();
    let before = saved(&engine);
    assert!(
        host.load_data(&mut engine)
            .unwrap_err()
            .to_string()
            .contains("org.second")
    );
    assert_eq!(saved(&engine), before);
    host.disable("org.second").unwrap();
    host.load_data(&mut engine).unwrap();
    assert_eq!(
        engine
            .document()
            .extensions
            .get("org.second", "state")
            .unwrap()
            .raw_json(),
        RAW
    );

    let mut document = engine.document().clone();
    document
        .extensions
        .insert(OWNER, "state", DataSection::bytes(4, &[42]).unwrap())
        .unwrap();
    let mut engine = Engine::new(document).unwrap();
    let before = saved(&engine);
    assert!(
        host.load_data(&mut engine)
            .unwrap_err()
            .to_string()
            .contains("newer than supported")
    );
    assert_eq!(saved(&engine), before);
    host.disable(OWNER).unwrap();
    host.load_data(&mut engine).unwrap();
    assert_eq!(saved(&engine), before);
}

#[test]
fn failed_scoped_write_discards_prior_writes_and_history_admission_is_atomic() {
    let mut host = host(DataPlugin::default());
    let mut engine = engine();
    let before = saved(&engine);
    assert!(
        host.with_data(OWNER, &mut engine, |cx| {
            cx.write("state", None)?;
            cx.write("undeclared", None)
        })
        .is_err()
    );
    assert_eq!(saved(&engine), before);
    let mut engine = Engine::with_limits(
        engine.document().clone(),
        voxelite_core::Limits {
            history_bytes: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(host.load_data(&mut engine).is_err());
    assert_eq!(saved(&engine), before);
    assert!(!engine.is_dirty());
}
