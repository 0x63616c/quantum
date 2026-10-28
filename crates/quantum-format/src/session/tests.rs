use super::*;
use quantum_core::{Actor, Command, Document, Origin, Transaction, Uuid};

fn new_session() -> DocumentSession {
    DocumentSession::new(
        Project::new(Document::new(Uuid::new_v4(), Uuid::new_v4())).unwrap(),
        None,
        Limits::default(),
    )
    .unwrap()
}
fn rename(session: &mut DocumentSession, name: &str) {
    session
        .engine
        .execute(Transaction {
            expected_revision: session.engine.revision(),
            id: Uuid::new_v4(),
            actor: Actor::new(Origin::Human),
            label: "Rename".into(),
            command: Command::EditDocument {
                name: Some(name.into()),
                bounds: None,
            },
        })
        .unwrap();
}
fn save(session: &mut DocumentSession, path: &Path, policy: SavePolicy) -> SessionResult<()> {
    let receipt = session.prepare_save(path, policy).write()?;
    assert_eq!(
        receipt.fingerprint(),
        fingerprint(&std::fs::read(path).unwrap())
    );
    assert_eq!(
        receipt.bytes(),
        std::fs::metadata(path).unwrap().len() as usize
    );
    session.finish_save(receipt);
    Ok(())
}
const POLICIES: [SavePolicy; 2] = [SavePolicy::ReplaceSelected, SavePolicy::CreateOrTracked];

#[test]
fn dirty_save_and_open_roundtrip_for_both_hosts() {
    for policy in POLICIES {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut session = new_session();
        assert!(!session.is_dirty());
        assert!(session.needs_save(), "agent documents need a first save");
        rename(&mut session, "Saved");
        assert!(save(&mut session, &dir.path().join("missing/file.voxl"), policy).is_err());
        assert!(session.is_dirty());
        assert!(session.path.is_none());
        save(&mut session, &path, policy).unwrap();
        assert!(!session.is_dirty());
        assert!(!session.needs_save());
        let opened = DocumentSession::open(&path, Limits::default()).unwrap();
        assert_eq!(opened.engine.document(), session.engine.document());
        assert_eq!(opened.path, Some(path.canonicalize().unwrap()));
        assert!(!opened.needs_save());
        let before = std::fs::read(&path).unwrap();
        save(&mut session, &path, policy).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}

#[test]
fn background_save_receipt_keeps_later_edits_dirty_and_ignores_other_documents() {
    for policy in POLICIES {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut session = new_session();
        rename(&mut session, "Captured");
        let task = session.prepare_save(&path, policy);
        rename(&mut session, "Later edit");
        let receipt = std::thread::spawn(move || task.write())
            .join()
            .unwrap()
            .unwrap();
        session.finish_save(receipt);
        assert!(session.is_dirty());
        assert_eq!(
            DocumentSession::open(&path, Limits::default())
                .unwrap()
                .engine
                .document()
                .content
                .name,
            "Captured"
        );
        save(&mut session, &path, policy).unwrap();
        assert!(!session.is_dirty());
        let receipt = session.prepare_save(&path, policy).write().unwrap();
        let mut other = new_session();
        other.finish_save(receipt);
        assert!(other.path.is_none());
        assert!(other.needs_save());
    }
}

#[test]
fn background_save_updates_the_dirty_baseline_even_after_undo() {
    for policy in POLICIES {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut session = new_session();
        save(&mut session, &path, policy).unwrap();
        rename(&mut session, "Captured");
        let task = session.prepare_save(&path, policy);
        let entry = session.engine.head().unwrap();
        session
            .engine
            .undo(session.engine.revision(), entry)
            .unwrap();
        assert!(!session.is_dirty(), "undo returns to the old disk state");

        session.finish_save(task.write().unwrap());
        assert!(session.is_dirty(), "disk now contains the captured edit");
        assert!(session.engine.is_dirty(), "UI dirty checks must agree");
        session
            .engine
            .redo(session.engine.revision(), entry)
            .unwrap();
        assert!(!session.is_dirty(), "redo now matches the saved snapshot");

        rename(&mut session, "Next save");
        let task = session.prepare_save(&path, policy);
        rename(&mut session, "Temporary edit");
        session
            .engine
            .undo(session.engine.revision(), session.engine.head().unwrap())
            .unwrap();
        session.finish_save(task.write().unwrap());
        assert!(
            !session.is_dirty(),
            "matching content at a later revision is clean"
        );
    }
}

#[test]
fn save_receipts_cannot_be_adopted_by_another_open_of_the_same_document() {
    for policy in POLICIES {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let copy = dir.path().join("copy.voxl");
        let mut session = new_session();
        save(&mut session, &path, policy).unwrap();
        let receipt = session.prepare_save(&copy, policy).write().unwrap();
        let mut reopened = DocumentSession::open(&path, Limits::default()).unwrap();
        let baselines = reopened.baselines.clone();
        reopened.finish_save(receipt);
        assert_eq!(reopened.path, Some(path.canonicalize().unwrap()));
        assert_eq!(reopened.baselines, baselines);
        assert!(!reopened.needs_save());
    }
}

#[test]
fn external_changes_leave_disk_session_and_temporary_directory_intact() {
    for policy in POLICIES {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut session = new_session();
        save(&mut session, &path, policy).unwrap();
        rename(&mut session, "Unsaved");
        let baseline = session.baselines.clone();
        let task = session.prepare_save(&path, policy);
        std::fs::write(&path, b"external change").unwrap();
        assert!(matches!(task.write(), Err(SessionError::ExternalChange)));
        assert_eq!(std::fs::read(&path).unwrap(), b"external change");
        assert_eq!(session.baselines, baseline);
        assert!(session.is_dirty());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        // The staging layer must also recheck a fingerprint just before rename.
        assert!(matches!(
            atomic_write(
                &path,
                b"replacement",
                baseline.values().next().map(String::as_str),
                policy
            ),
            Err(SessionError::ExternalChange)
        ));
        assert_eq!(std::fs::read(&path).unwrap(), b"external change");
        std::fs::remove_file(&path).unwrap();
        assert!(save(&mut session, &path, policy).is_err());
        assert!(session.is_dirty());
        assert!(!path.exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}

#[test]
fn save_as_policies_preserve_selected_replacement_and_agent_path_history() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first.voxl");
    let second = dir.path().join("second.voxl");
    let mut agent = new_session();
    save(&mut agent, &first, SavePolicy::CreateOrTracked).unwrap();
    save(&mut agent, &second, SavePolicy::CreateOrTracked).unwrap();
    rename(&mut agent, "Update earlier path");
    save(&mut agent, &first, SavePolicy::CreateOrTracked).unwrap();
    std::fs::write(&second, b"external").unwrap();
    assert!(matches!(
        save(&mut agent, &second, SavePolicy::CreateOrTracked),
        Err(SessionError::ExternalChange)
    ));
    let mut app = new_session();
    assert!(matches!(
        save(&mut app, &first, SavePolicy::CreateOrTracked),
        Err(SessionError::FileExists)
    ));
    save(&mut app, &first, SavePolicy::ReplaceSelected).unwrap();
    save(&mut app, &second, SavePolicy::ReplaceSelected).unwrap();
    assert_eq!(app.baselines.len(), 1, "app retains only its current path");
    std::fs::write(&first, b"selected replacement").unwrap();
    save(&mut app, &first, SavePolicy::ReplaceSelected).unwrap();
}

#[cfg(unix)]
#[test]
fn save_replaces_inode_atomically_and_preserves_app_permissions() {
    use std::{
        fs,
        io::Read,
        os::unix::fs::{MetadataExt, PermissionsExt},
    };
    for policy in POLICIES {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("project.voxl");
        let mut session = new_session();
        save(&mut session, &path, policy).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let before = fs::read(&path).unwrap();
        let mut reader = fs::File::open(&path).unwrap();
        let inode = reader.metadata().unwrap().ino();
        rename(&mut session, "New inode");
        save(&mut session, &path, policy).unwrap();
        assert_ne!(fs::metadata(&path).unwrap().ino(), inode);
        let mut retained = Vec::new();
        reader.read_to_end(&mut retained).unwrap();
        assert_eq!(
            retained, before,
            "existing readers see the whole previous file"
        );
        assert_ne!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        if policy == SavePolicy::ReplaceSelected {
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o640
            );
        }
    }
}

#[test]
fn create_only_artifacts_never_clobber_or_leave_temporary_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("export.png");
    write_new(&path, b"first").unwrap();
    assert!(matches!(
        write_new(&path, b"second"),
        Err(SessionError::FileExists)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), b"first");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn create_only_artifacts_accept_a_bare_relative_filename() {
    // Reserve a unique filename and retain a cleanup guard without changing cwd
    // (other tests may be accessing files concurrently).
    let cleanup = tempfile::NamedTempFile::new_in(".")
        .unwrap()
        .into_temp_path();
    std::fs::remove_file(&cleanup).unwrap();
    let path = Path::new(cleanup.file_name().unwrap());
    write_new(path, b"export").unwrap();
    assert_eq!(std::fs::read(&cleanup).unwrap(), b"export");
    assert!(matches!(
        write_new(path, b"replace"),
        Err(SessionError::FileExists)
    ));
}

#[cfg(unix)]
#[test]
fn replacement_rechecks_the_file_type_for_both_hosts() {
    for policy in POLICIES {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("external.voxl");
        let path = dir.path().join("project.voxl");
        let bytes = b"identical original bytes";
        std::fs::write(&target, bytes).unwrap();
        // Model a regular destination replaced with a symlink after preflight.
        // Matching target bytes must not allow the final rename to replace it.
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(matches!(
            atomic_write(&path, b"replacement", Some(&fingerprint(bytes)), policy),
            Err(SessionError::InvalidDestination)
        ));
        assert!(path.is_symlink());
        assert_eq!(std::fs::read(&target).unwrap(), bytes);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }
}
