use super::*;

const FILES: &[(&str, &str)] = &[
    ("settings.json", "{\"grid\":false}"),
    ("palettes.json", "[1,2,3]"),
    (
        "recents.json",
        "[\"/Users/artist/Documents/original.voxl\"]",
    ),
    ("thumbnails/scene.png", "thumbnail bytes"),
];
fn seed(old: &Path) {
    fs::create_dir_all(old.join("thumbnails")).unwrap();
    for (name, bytes) in FILES {
        fs::write(old.join(name), bytes).unwrap();
    }
}
fn assert_data(new: &Path) {
    for (name, bytes) in FILES {
        assert_eq!(fs::read_to_string(new.join(name)).unwrap(), *bytes);
    }
}
#[test]
fn old_only_moves_complete_directory_once() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old");
    let new = dir.path().join("new");
    seed(&old);
    migrate(&old, &new).unwrap();
    assert_data(&new);
    assert!(!old.exists());
    migrate(&old, &new).unwrap();
    assert_data(&new);
}
#[test]
fn new_only_both_and_neither_never_overwrite_or_create_data() {
    for old_exists in [false, true] {
        for new_exists in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let old = dir.path().join("old");
            let new = dir.path().join("new");
            if old_exists {
                seed(&old);
            }
            if new_exists {
                fs::create_dir(&new).unwrap();
                fs::write(new.join("settings.json"), "authoritative").unwrap();
            }
            migrate(&old, &new).unwrap();
            if new_exists {
                assert_eq!(
                    fs::read_to_string(new.join("settings.json")).unwrap(),
                    "authoritative"
                );
                assert!(!new.join("palettes.json").exists());
                if old_exists {
                    assert_data(&old);
                }
            } else {
                assert_eq!(new.exists(), old_exists);
            }
        }
    }
}
#[test]
fn cross_device_copy_retains_source_and_all_durable_files() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old");
    let new = dir.path().join("new");
    seed(&old);
    #[cfg(unix)]
    let _socket = std::os::unix::net::UnixListener::bind(old.join("session-123.sock")).unwrap();
    migrate_with(
        &old,
        &new,
        |_, _| Err(io::ErrorKind::CrossesDevices.into()),
        copy_directory,
    )
    .unwrap();
    assert_data(&old);
    assert_data(&new);
    assert!(!new.join("session-123.sock").exists());
}
#[test]
fn unreadable_source_and_partial_copy_leave_no_destination() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old");
    let new = dir.path().join("new");
    seed(&old);
    let error = migrate_with(
        &old,
        &new,
        |_, _| Err(io::ErrorKind::PermissionDenied.into()),
        copy_directory,
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    let error = migrate_with(
        &old,
        &new,
        |_, _| Err(io::ErrorKind::CrossesDevices.into()),
        |_, staging| {
            fs::write(staging.join("settings.json"), "partial")?;
            Err(io::ErrorKind::PermissionDenied.into())
        },
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(!new.exists());
    assert_data(&old);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}
#[test]
fn destination_created_during_copy_wins_even_when_empty() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old");
    let new = dir.path().join("new");
    seed(&old);
    migrate_with(
        &old,
        &new,
        |_, _| Err(io::ErrorKind::CrossesDevices.into()),
        |old, staging| {
            copy_directory(old, staging)?;
            fs::create_dir(&new)
        },
    )
    .unwrap();
    assert_eq!(fs::read_dir(&new).unwrap().count(), 0);
    assert_data(&old);
}
#[test]
fn platform_paths_migrate_without_touching_documents() {
    for macos in [false, true] {
        for use_xdg in [false, true] {
            let home = tempfile::tempdir().unwrap();
            let xdg = home.path().join("xdg");
            let xdg = use_xdg.then_some(xdg.as_path());
            let new = super::super::settings_dir(home.path(), xdg, macos);
            let old = new.with_file_name("voxelite");
            seed(&old);
            let documents = home.path().join("Documents/old-project.voxl");
            fs::create_dir_all(documents.parent().unwrap()).unwrap();
            fs::write(&documents, "user project").unwrap();
            initialize(home.path(), xdg, macos).unwrap();
            assert_data(&new);
            assert_eq!(fs::read_to_string(documents).unwrap(), "user project");
        }
    }
}
#[test]
fn macos_also_migrates_earlier_xdg_layout() {
    let home = tempfile::tempdir().unwrap();
    let old = home.path().join(".local/share/voxelite");
    seed(&old);
    initialize(home.path(), None, true).unwrap();
    assert_data(&home.path().join("Library/Application Support/quantum"));
}
