//! Reviewable canonical project bytes and byte-identical round trips.

#[test]
fn canonical_project_bytes() {
    for input in [
        include_bytes!("../../../spec/voxl/1.0/fixtures/populated.input.voxl").as_slice(),
        include_bytes!("../../../spec/voxl/1.0/fixtures/populated.canonical.voxl").as_slice(),
    ] {
        let encoded = voxelite_format::from_bytes(input)
            .unwrap()
            .to_bytes()
            .unwrap();
        assert_eq!(
            voxelite_format::from_bytes(&encoded)
                .unwrap()
                .to_bytes()
                .unwrap(),
            encoded
        );
        // An explicit end marker preserves the canonical trailing newline in the snapshot.
        insta::assert_snapshot!(
            "populated_canonical_bytes",
            format!("{}<EOF>", String::from_utf8(encoded).unwrap())
        );
    }
}
