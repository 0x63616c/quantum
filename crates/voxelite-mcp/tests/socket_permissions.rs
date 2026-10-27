//! Socket publication must never expose the permissions inherited from umask.
#![cfg(unix)]

use std::{
    fs, io,
    os::unix::{
        fs::{FileTypeExt, MetadataExt, PermissionsExt, symlink},
        net::{SocketAddr, UnixListener, UnixStream},
    },
    path::Path,
};
use voxelite_core::{Document, Uuid};
use voxelite_format::Project;
use voxelite_mcp::attached::AttachedSession;

fn bind(root: &Path, path: &Path) -> io::Result<AttachedSession> {
    AttachedSession::bind(
        root,
        Project::new(Document::new(Uuid::new_v4(), Uuid::new_v4())).unwrap(),
        path,
    )
}

#[test]
fn socket_is_private_and_connectable_in_an_owned_sticky_parent() {
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o1777)).unwrap();
    let path = dir.path().join("session.sock");
    let session = bind(dir.path(), &path).unwrap();
    assert_eq!(session.path(), path);
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert!(metadata.file_type().is_socket());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    UnixStream::connect(&path).unwrap();
    drop(session);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    assert_eq!(
        fs::metadata(dir.path()).unwrap().permissions().mode() & 0o7777,
        0o1777
    );
}

#[test]
fn unsafe_shared_parent_is_refused_before_binding() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.sock");
    for mode in [0o777, 0o770] {
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(mode)).unwrap();
        let error = bind(dir.path(), &path)
            .err()
            .expect("unsafe parent accepted");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        assert_eq!(
            fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
            mode
        );
    }
}

#[test]
fn existing_paths_are_preserved_and_staging_is_cleaned_up() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("existing");
    let link = dir.path().join("session.sock");
    fs::write(&file, b"keep me").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    symlink(&file, &link).unwrap();
    let dangling = dir.path().join("dangling");
    symlink(dir.path().join("absent"), &dangling).unwrap();
    let directory = dir.path().join("directory");
    fs::create_dir(&directory).unwrap();
    let socket = dir.path().join("live.sock");
    let _listener = UnixListener::bind(&socket).unwrap();
    for path in [&file, &link, &dangling, &directory, &socket] {
        let before = fs::symlink_metadata(path).unwrap();
        let error = bind(dir.path(), path)
            .err()
            .expect("existing path replaced");
        assert_eq!(error.kind(), io::ErrorKind::AddrInUse, "{error}");
        let after = fs::symlink_metadata(path).unwrap();
        assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
        assert_eq!(before.permissions().mode(), after.permissions().mode());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 5);
    }
    assert_eq!(fs::read(&file).unwrap(), b"keep me");
    assert_eq!(fs::read_link(&link).unwrap(), file);
    UnixStream::connect(&socket).unwrap();
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 5);
}

// sockaddr_un.sun_path includes the terminating NUL on both platforms.
#[cfg(target_os = "macos")]
const MAX_SOCKET_PATH: usize = 103;
#[cfg(not(target_os = "macos"))]
const MAX_SOCKET_PATH: usize = 107;

fn parent_with_length(root: &Path, length: usize) -> std::path::PathBuf {
    let parent = root.join("p".repeat(length - root.as_os_str().len() - 1));
    fs::create_dir(&parent).unwrap();
    assert_eq!(parent.as_os_str().len(), length);
    parent
}

#[test]
fn short_socket_name_in_an_eighty_byte_parent_is_supported() {
    let dir = tempfile::tempdir_in("/tmp").unwrap();
    let parent = parent_with_length(dir.path(), 80);
    let path = parent.join("s");
    let session = bind(dir.path(), &path).unwrap();
    UnixStream::connect(&path).unwrap();
    drop(session);
    assert_eq!(fs::read_dir(parent).unwrap().count(), 0);
}

#[test]
fn advertised_address_length_is_validated_before_publication() {
    let dir = tempfile::tempdir_in("/tmp").unwrap();
    for length in [MAX_SOCKET_PATH, MAX_SOCKET_PATH + 1] {
        let path = dir
            .path()
            .join("s".repeat(length - dir.path().as_os_str().len() - 1));
        assert_eq!(path.as_os_str().len(), length);
        if length == MAX_SOCKET_PATH {
            let session = bind(dir.path(), &path).unwrap();
            UnixStream::connect(&path).unwrap();
            drop(session);
        } else {
            assert_eq!(
                SocketAddr::from_pathname(&path).unwrap_err().kind(),
                io::ErrorKind::InvalidInput
            );
            let error = bind(dir.path(), &path)
                .err()
                .expect("oversized address accepted");
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(error.to_string().contains("session socket address"));
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}

#[test]
fn staging_address_length_is_validated_before_creating_files() {
    let dir = tempfile::tempdir_in("/tmp").unwrap();
    // The staging suffix is /.m<8 hex>/s (13 bytes).
    for length in [MAX_SOCKET_PATH, MAX_SOCKET_PATH + 1] {
        let parent = parent_with_length(dir.path(), length - 13);
        let path = parent.join("s");
        // The advertised address itself is valid in both cases.
        drop(UnixListener::bind(&path).unwrap());
        fs::remove_file(&path).unwrap();
        if length == MAX_SOCKET_PATH {
            let session = bind(dir.path(), &path).unwrap();
            UnixStream::connect(&path).unwrap();
            drop(session);
        } else {
            let error = bind(dir.path(), &path)
                .err()
                .expect("oversized staging address accepted");
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(error.to_string().contains("staging socket address"));
        }
        assert_eq!(fs::read_dir(parent).unwrap().count(), 0);
    }
}
