//! Archive-contained round-trip contract without repository fixtures.
use quantum_core::{Document, Uuid};
use quantum_format::{Project, from_bytes, read};

#[test]
fn canonical_empty_document_roundtrip() {
    let document = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    let project = Project::new(document).unwrap();
    let bytes = project.to_bytes().unwrap();
    for reopened in [from_bytes(&bytes).unwrap(), read(bytes.as_slice()).unwrap()] {
        assert_eq!(reopened.document, project.document);
        assert_eq!(reopened.to_bytes().unwrap(), bytes);
        assert_eq!(
            reopened.semantic_hash().unwrap(),
            project.semantic_hash().unwrap()
        );
    }
}
