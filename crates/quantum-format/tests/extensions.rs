//! Raw extension persistence, compatibility and atomic failure contracts.
use quantum_core::*;
use quantum_format::{
    Project, from_bytes,
    session::{DocumentSession, SavePolicy},
};

const GOLDEN: &[u8] = include_bytes!("../../../spec/voxl/1.0/fixtures/populated.canonical.voxl");
const RAW: &str = r#"{ "payload" : {"z":1e-03,"a":"\u0061","nested":{"voxels":[0,255]}},
  "encoding":"json", "schema_version": 7, "future": true }"#;

#[expect(
    clippy::unwrap_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn with_sections(sections: &str) -> Vec<u8> {
    format!(
        "{{\"extensions\":{sections},{}",
        &std::str::from_utf8(GOLDEN).unwrap()[1..]
    )
    .into_bytes()
}

#[test]
fn old_golden_and_empty_namespaces_keep_exact_output() {
    for bytes in [
        GOLDEN.to_vec(),
        with_sections("{}"),
        with_sections(r#"{"org.example":{}}"#),
    ] {
        let project = from_bytes(&bytes).unwrap();
        assert!(project.document.extensions.is_empty());
        assert_eq!(project.version.major, 1);
        assert_eq!(project.version.minor, 0);
        assert_eq!(project.to_bytes().unwrap(), GOLDEN);
        assert_eq!(
            Project::new(project.document).unwrap().to_bytes().unwrap(),
            GOLDEN
        );
    }
}

#[test]
fn unknown_sections_retain_exact_envelopes_across_reader_engine_and_save() {
    let bytes = with_sections(&format!(
        r#"{{"org.example":{{"state":{RAW},"binary":{{"schema_version":1,"encoding":"bytes","payload":[0, 255, 128, 10]}}}}}}"#
    ));
    let mut project = quantum_format::read(std::io::Cursor::new(&bytes)).unwrap();
    let before = project.semantic_hash().unwrap();
    let mut engine = project.open_engine(Limits::default()).unwrap();
    engine
        .execute(Transaction {
            expected_revision: 0,
            id: Uuid::new_v4(),
            actor: Actor::new(Origin::Human),
            label: "Rename".into(),
            command: Command::EditDocument {
                name: Some("Renamed".into()),
                bounds: None,
            },
        })
        .unwrap();
    project.document = engine.document().clone();
    let saved = project.to_bytes().unwrap();
    assert!(std::str::from_utf8(&saved).unwrap().contains(RAW));
    let loaded = from_bytes(&saved).unwrap();
    assert_eq!(
        loaded
            .document
            .extensions
            .get("org.example", "state")
            .unwrap()
            .raw_json(),
        RAW
    );
    assert_eq!(
        loaded
            .document
            .extensions
            .get("org.example", "binary")
            .unwrap()
            .bytes_payload()
            .unwrap(),
        [0, 255, 128, 10]
    );
    assert_eq!(loaded.to_bytes().unwrap(), saved);
    assert_ne!(before, loaded.semantic_hash().unwrap());
    let mut no_extensions = loaded.clone();
    no_extensions.document.extensions = Extensions::default();
    assert_ne!(
        loaded.semantic_hash().unwrap(),
        no_extensions.semantic_hash().unwrap()
    );
    assert!(
        !std::str::from_utf8(&no_extensions.to_bytes().unwrap())
            .unwrap()
            .contains("extensions")
    );
}

#[test]
fn malformed_sections_reject_with_owner_and_name() {
    for section in [
        r#"{"schema_version":0,"encoding":"json","payload":null}"#,
        r#"{"schema_version":1,"encoding":"json"}"#,
        r#"{"schema_version":1,"encoding":"bytes","payload":[256]}"#,
        r#"{"schema_version":1,"encoding":"bytes","payload":[1.5]}"#,
        r#"{"schema_version":1,"encoding":"unknown","payload":null}"#,
        r#"{"schema_version":1,"encoding":"json","payload":{"a":1,"a":2}}"#,
        r#"{"schema_version":1,"schema_version":2,"encoding":"json","payload":null}"#,
    ] {
        let bytes = with_sections(&format!(
            r#"{{"org.example":{{"good":{RAW},"bad":{section}}}}}"#
        ));
        let error = from_bytes(&bytes).unwrap_err().to_string();
        assert!(error.contains("extension org.example/bad"), "{error}");
    }
    for sections in [
        r#"{"org.example":{},"org.example":{}}"#,
        r#"{"org.example":{"state":{},"state":{}}}"#,
        r#"{"not_a_plugin":{}}"#,
        "null",
        "[]",
    ] {
        assert!(from_bytes(&with_sections(sections)).is_err());
    }
}

#[test]
fn extension_depth_and_raw_whitespace_are_bounded() {
    let deep = format!("{}0{}", "[".repeat(30), "]".repeat(30));
    assert!(
        DataSection::from_raw(&format!(
            r#"{{"schema_version":1,"encoding":"json","payload":{deep}}}"#
        ))
        .is_err()
    );
    let padded = format!(
        r#"{{{}"schema_version":1,"encoding":"json","payload":null}}"#,
        " ".repeat(MAX_EXTENSION_BYTES)
    );
    assert!(DataSection::from_raw(&padded).is_err());
}

#[test]
fn durable_save_retains_data_and_failed_open_leaves_file_and_session_intact() {
    let root = std::env::temp_dir().join(format!("quantum-extensions-{}", Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("project.voxl");
    let bytes = with_sections(&format!(r#"{{"org.example":{{"state":{RAW}}}}}"#));
    std::fs::write(&path, &bytes).unwrap();
    let mut session = DocumentSession::open(&path, Limits::default()).unwrap();
    let receipt = session
        .prepare_save(&path, SavePolicy::ReplaceSelected)
        .write()
        .unwrap();
    session.finish_save(receipt);
    let good = std::fs::read(&path).unwrap();
    assert!(std::str::from_utf8(&good).unwrap().contains(RAW));
    let bad = root.join("bad.voxl");
    let broken = with_sections(r#"{"org.example":{"broken":{"schema_version":"bad"}}}"#);
    std::fs::write(&bad, &broken).unwrap();
    assert!(DocumentSession::open(&bad, Limits::default()).is_err());
    assert_eq!(std::fs::read(&bad).unwrap(), broken);
    assert_eq!(std::fs::read(&path).unwrap(), good);
    assert_eq!(session.project().to_bytes().unwrap(), good);
    std::fs::remove_dir_all(root).unwrap();
}
