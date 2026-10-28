//! Canonical bytes, hostile input, metadata and portable-version contracts.
use proptest::prelude::*;
use serde_json::{Value, json};
use voxelite_core::*;
use voxelite_format::*;
const INPUT: &[u8] = include_bytes!("../../../spec/voxl/1.0/fixtures/populated.input.voxl");
const GOLDEN: &[u8] = include_bytes!("../../../spec/voxl/1.0/fixtures/populated.canonical.voxl");
fn input() -> Value {
    serde_json::from_slice(INPUT).unwrap()
}
fn parse(v: Value) -> FormatResult<Project> {
    from_bytes(&serde_json::to_vec(&v).unwrap())
}
#[test]
fn golden_and_idempotence() {
    let p = from_bytes(INPUT).unwrap();
    assert_eq!(p.to_bytes().unwrap(), GOLDEN);
    assert_eq!(from_bytes(GOLDEN).unwrap().to_bytes().unwrap(), GOLDEN);
    assert_eq!(from_bytes(GOLDEN).unwrap().document, p.document);
}
#[test]
fn unknown_fields_every_level_survive_and_minor_is_retained() {
    let mut v = input();
    v["version"]["minor"] = json!(8);
    v["future_metadata"] = json!({"org.example":{"text":"Élan / forest","flags":[true,null,-7]}});
    v["document"]["bounds"]["future"] = json!(42);
    v["palette"]["slots"][0]["future"] = json!("slot");
    v["layers"][0]["future"] = json!([2, 1]);
    let p = parse(v).unwrap();
    let b = p.to_bytes().unwrap();
    let out: Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(out["document"]["bounds"]["future"], 42);
    assert_eq!(out["palette"]["slots"][0]["future"], "slot");
    assert_eq!(out["version"]["minor"], 8);
    assert_eq!(from_bytes(&b).unwrap().to_bytes().unwrap(), b);
}
#[test]
fn duplicate_keys_and_coordinates_rejected() {
    for text in [
        "{\"format\":\"voxelite\",\"format\":\"voxelite\"}",
        "{\"extensions\":{\"x\":1,\"x\":2}}",
    ] {
        assert!(from_bytes(text.as_bytes()).is_err());
    }
    let mut v = input();
    v["layers"][0]["voxels"] = json!([[0, 0, 0, 1], [0, 0, 0, 1]]);
    assert!(parse(v).is_err());
}
#[test]
fn strict_model_validation() {
    let cases = [
        ("/version/major", json!(2)),
        ("/required_features", json!(["unknown"])),
        ("/palette/order", json!([1, 1])),
        ("/palette/slots/0/rgba", json!([1, 2, 3, 0])),
        ("/layers/0/voxels", json!([[0, 0, 0, 0]])),
        ("/layers/0/voxels", json!([[0, 0, 0, 2]])),
        ("/layers/0/voxels", json!([[32, 0, 0, 1]])),
        ("/layers/0/voxels", json!([[0.0, 0, 0, 1]])),
        ("/document/bounds/max", json!([999, 1, 1])),
        ("/document/name", json!("")),
    ];
    for (path, value) in cases {
        let mut v = input();
        *v.pointer_mut(path).unwrap() = value;
        assert!(parse(v).is_err(), "{path}");
    }
}
#[test]
fn malformed_utf8_surrogates_numbers_and_depth() {
    for bytes in [
        b"\xff".as_slice(),
        b"{\"x\":\"\\ud800\"}",
        b"{\"x\":1e999}",
        b"{\"x\":-0}",
        b"{\"x\":1.0}",
        b"{\"x\":9223372036854775808}",
    ] {
        assert!(from_bytes(bytes).is_err());
    }
    let mut v = input();
    let mut deep = json!(0);
    for _ in 0..18 {
        deep = json!([deep]);
    }
    v["extensions"] = deep;
    assert!(parse(v).is_err());
    assert!(from_bytes(&vec![b' '; MAX_FILE_BYTES + 1]).is_err());
}
#[test]
fn one_cell_changes_one_tuple_line() {
    let p = from_bytes(INPUT).unwrap();
    let mut e = Engine::new(p.document).unwrap();
    let layer = e.document().content.layers[0].id;
    e.execute(Transaction {
        expected_revision: 0,
        id: Uuid::from_u128(100),
        actor: Actor::new(Origin::Agent),
        label: "One cell".into(),
        command: Command::SetVoxels {
            layer,
            voxels: vec![Voxel {
                at: [1, 0, 0],
                slot: 1,
            }],
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
    })
    .unwrap();
    let bytes = Project::new(e.document().clone())
        .unwrap()
        .to_bytes()
        .unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("        [0, 0, 0, 1],\n        [1, 0, 0, 1]\n"));
}
#[test]
fn portable_comments_versions_and_hashes() {
    let mut e = Engine::new(from_bytes(INPUT).unwrap().document).unwrap();
    let layer = e.document().content.layers[0].id;
    for (i, c) in [
        Command::CreateComment {
            id: Uuid::from_u128(10),
            anchor: Anchor::Voxel { at: [0; 3] },
            layer: Some(layer),
            message: Uuid::from_u128(11),
            text: "Line one\nLine two\t✓".into(),
        },
        Command::SaveVersion {
            id: Uuid::from_u128(20),
            name: "Approved".into(),
            parent: None,
        },
    ]
    .into_iter()
    .enumerate()
    {
        e.execute(Transaction {
            expected_revision: e.revision(),
            id: Uuid::from_u128(100 + i as u128),
            actor: Actor::new(Origin::Agent),
            label: "Portable content".into(),
            command: c,
        })
        .unwrap();
    }
    let p = Project::new(e.document().clone()).unwrap();
    let bytes = p.to_bytes().unwrap();
    assert_eq!(from_bytes(&bytes).unwrap().document, p.document);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("\\u000a"));
    assert!(!text.contains("revision"));
    assert!(!text.contains("client_id"));
    let mut corrupt: Value = serde_json::from_str(&text).unwrap();
    corrupt["versions"][0]["snapshot"]["name"] = json!("Tampered");
    assert!(parse(corrupt).is_err());
}
#[test]
fn view_is_excluded_from_semantic_hash() {
    let mut p = from_bytes(INPUT).unwrap();
    let hash = p.semantic_hash().unwrap();
    p.view = Some(json!({"eye":[1.5,2.0,3.0],"target":[0,0,0],"projection":"orthographic"}));
    assert_eq!(hash, p.semantic_hash().unwrap());
    assert!(p.to_bytes().unwrap() != GOLDEN);
}
proptest! {
    #[test]fn random_voxel_roundtrip(cells in prop::collection::btree_set((-20i32..20,0i32..20,-20i32..20),0..100)){
        let mut e=Engine::new(Document::new(Uuid::from_u128(1),Uuid::from_u128(2))).unwrap();
        e.execute(Transaction{expected_revision:0,id:Uuid::from_u128(3),actor:Actor::new(Origin::Human),label:"Generated".into(),command:Command::SetVoxels{layer:Uuid::from_u128(2),voxels:cells.into_iter().map(|(x,y,z)|Voxel{at:[x,y,z],slot:1}).collect(),mode:Mode::Place,mirror:Mirror::default()}}).unwrap();
        let p=Project::new(e.document().clone()).unwrap();let bytes=p.to_bytes().unwrap();let read=from_bytes(&bytes).unwrap();prop_assert_eq!(&read.document,&p.document);prop_assert_eq!(read.to_bytes().unwrap(),bytes);
    }
    #[test]fn arbitrary_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(),0..1024)){let _=from_bytes(&bytes);}
}

#[test]
fn legacy_file_roundtrips_after_runtime_history_undo_redo() {
    let project = from_bytes(INPUT).unwrap();
    let original = project.document.clone();
    let mut engine = Engine::new(original.clone()).unwrap();
    let id = Uuid::from_u128(900);
    engine
        .execute(Transaction {
            expected_revision: 0,
            id,
            actor: Actor::new(Origin::Human),
            label: "Rename".into(),
            command: Command::EditDocument {
                name: Some("Temporary".into()),
                bounds: None,
            },
        })
        .unwrap();
    engine.undo(engine.revision(), id).unwrap();
    assert_eq!(
        Project::new(engine.document().clone())
            .unwrap()
            .to_bytes()
            .unwrap(),
        GOLDEN
    );
    engine.redo(engine.revision(), id).unwrap();
    engine.undo(engine.revision(), id).unwrap();
    assert_eq!(
        from_bytes(
            &Project::new(engine.document().clone())
                .unwrap()
                .to_bytes()
                .unwrap()
        )
        .unwrap()
        .document,
        original
    );
}
