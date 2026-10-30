use super::*;

#[test]
fn inventory_requires_current_and_boolean_stability() {
    for entry in [
        r#"{ stable = true }"#,
        r#"{ current = "x" }"#,
        r#"{ current = "x", stable = "true" }"#,
    ] {
        assert!(toml::from_str::<Inventory>(&format!("[identifiers]\nname = {entry}")).is_err());
    }
    let inventory: Inventory =
        toml::from_str("[identifiers]\nname = { current = [], stable = true }").unwrap();
    assert_eq!(inventory["identifiers"]["name"].current, json!([]));
}

#[test]
fn only_unstable_entries_may_plan_renames() {
    let workspace = Workspace::discover().unwrap();
    let mut audit = Audit {
        workspace: &workspace,
        errors: vec![],
        inventory: toml::from_str(
            r#"
        [product]
        allowed = { current = "old", next = "new", stable = false }
        [identifiers]
        unchanged = { current = "same", next = "same", stable = true }
        implicit = { current = ["one", "two"], stable = true }
        forbidden = { current = "old", next = "new", stable = true }
    "#,
        )
        .unwrap(),
    };
    audit.stability().unwrap();
    assert_eq!(
        audit.errors,
        ["brand.toml: stable identifiers.forbidden must keep its current value"]
    );
    assert!(
        audit
            .identifier("absent")
            .unwrap_err()
            .to_string()
            .contains("missing identifiers.absent.current")
    );
    assert!(
        audit
            .identifier("implicit")
            .unwrap_err()
            .to_string()
            .contains("must be a string")
    );
}

fn bundle_xml(extra: &str) -> String {
    format!(
        r#"<?xml version="1.0"?><plist version="1.0"><dict>
        <key>CFBundleIdentifier</key><string>example.app</string>
        <key>CFBundleName</key><string>A &amp; B</string>
        <key>CFBundleDisplayName</key><string>Display</string>
        <key>CFBundleExecutable</key><string>Executable</string>
        <key>CFBundleIconFile</key><string>Icon</string>
        {extra}</dict></plist>"#
    )
}

#[test]
fn plist_decodes_entities_optional_arrays_and_ordered_document_types() {
    let empty: Bundle = plist::from_bytes(bundle_xml("").as_bytes()).unwrap();
    assert_eq!(empty.name, "A & B");
    assert!(empty.exported.is_empty() && empty.imported.is_empty() && empty.documents.is_empty());
    let xml = bundle_xml(
        r#"
        <key>UTExportedTypeDeclarations</key><array>
            <dict><key>UTTypeIdentifier</key><string>export.one</string></dict>
            <dict><key>UTTypeIdentifier</key><string>export.two</string></dict>
        </array>
        <key>UTImportedTypeDeclarations</key><array>
            <dict><key>UTTypeIdentifier</key><string>import.one</string></dict>
        </array>
        <key>CFBundleDocumentTypes</key><array>
            <dict><key>LSItemContentTypes</key><array><string>one</string><string>two</string></array></dict>
            <dict/>
            <dict><key>LSItemContentTypes</key><array><string>three</string></array></dict>
        </array>
    "#,
    );
    let info: Bundle = plist::from_bytes(xml.as_bytes()).unwrap();
    assert_eq!(
        info.exported
            .iter()
            .map(|item| item.identifier.as_str())
            .collect::<Vec<_>>(),
        ["export.one", "export.two"]
    );
    assert_eq!(info.imported[0].identifier, "import.one");
    assert_eq!(
        info.documents
            .into_iter()
            .flat_map(|item| item.content_types)
            .collect::<Vec<_>>(),
        ["one", "two", "three"]
    );
    assert!(
        plist::from_bytes::<Bundle>(
            bundle_xml("<key>UTExportedTypeDeclarations</key><array><dict/></array>").as_bytes()
        )
        .is_err()
    );
    assert!(plist::from_bytes::<Bundle>(b"invalid plist").is_err());
}

#[test]
fn member_patterns_deduplicate_and_exclude_container_manifest() {
    let temp = tempfile::tempdir().unwrap();
    // The root itself can contain glob metacharacters.
    let root = temp.path().join("repo [copy]");
    for path in [
        "xtask/Cargo.toml",
        "crates/one/Cargo.toml",
        "crates/plugins/Cargo.toml",
        "crates/plugins/two/Cargo.toml",
    ] {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
    std::fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
    let workspace = Workspace::at(root).unwrap();
    let members = Members {
        members: vec![
            "crates/*".into(),
            "crates/plugins/*".into(),
            "crates/one".into(),
            "xtask".into(),
        ],
        exclude: vec!["crates/plugins/Cargo.toml".into()],
    };
    assert_eq!(
        member_manifests(&workspace, &members).unwrap(),
        [
            "crates/one/Cargo.toml",
            "crates/plugins/two/Cargo.toml",
            "xtask/Cargo.toml"
        ]
        .map(|path| workspace.path(path))
        .into()
    );
}

#[test]
fn source_captures_require_exactly_one_match_and_skill_names_match_whole_lines() {
    let pattern = r#"format!\(\s*"([^"]+)\{name\}"\s*\)"#;
    assert_eq!(
        super::super::captures("format!( \"voxelite_{name}\" )", pattern).unwrap(),
        ["voxelite_"]
    );
    let names = super::super::captures(
        "---\nname: voxelite-one\nother-name: ignored\nname: voxelite-two\n",
        r"(?m)^name:\s*(\S+)\s*$",
    )
    .unwrap();
    assert_eq!(names, ["voxelite-one", "voxelite-two"]);
    let workspace = Workspace::discover().unwrap();
    let mut audit = Audit {
        workspace: &workspace,
        inventory: BTreeMap::new(),
        errors: vec![],
    };
    audit.equal("duplicate constants", vec!["same", "same"], vec!["same"]);
    audit.equal("missing constant", Vec::<String>::new(), vec!["same"]);
    assert_eq!(audit.errors.len(), 2);
}

#[test]
fn repository_brand_consistency() {
    let errors = check(&Workspace::discover().unwrap()).unwrap();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn storage_checks_production_paths_and_env_even_with_unchanged_test_literals() {
    let workspace = Workspace::discover().unwrap();
    let source = workspace
        .read("crates/voxelite-app/src/local_settings.rs")
        .unwrap();
    for (old, new) in [
        (
            "Library/Application Support/voxelite",
            "Library/Application Support/quantum",
        ),
        (".join(\"voxelite\")", ".join(\"quantum\")"),
        (".local/share", ".local/data"),
        ("VOXELITE_DATA_DIR", "QUANTUM_DATA_DIR"),
        ("XDG_DATA_HOME", "XDG_CONFIG_HOME"),
    ] {
        let mut audit = Audit {
            workspace: &workspace,
            inventory: read_toml(&workspace, "brand.toml").unwrap(),
            errors: vec![],
        };
        audit.storage_source(&source).unwrap();
        assert!(audit.errors.is_empty(), "{:?}", audit.errors);
        // Replace only the production occurrence, preserving matching test literals.
        let changed = source.replacen(old, new, 1);
        assert_ne!(changed, source);
        audit.storage_source(&changed).unwrap();
        assert!(!audit.errors.is_empty(), "missed rename of {old}");
    }
}
