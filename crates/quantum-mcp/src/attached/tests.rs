use super::*;
use quantum_core::{Document, Limits};
use quantum_plugins_builtin::mcp_host;
use serde_json::json;

#[test]
fn socket_is_private_at_publication_boundaries() {
    use std::{fs, os::unix::net::UnixStream, process::Command};

    // Set umask only in a child, never in the multithreaded test runner.
    const CHILD: &str = "QUANTUM_SOCKET_PERMISSIONS_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let status = Command::new("sh")
            .args(["-c", "umask 000; exec \"$@\"", "socket-permissions"])
            .arg(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "attached::tests::socket_is_private_at_publication_boundaries",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o1777)).unwrap();
    let path = dir.path().join("session.sock");
    let mut stages = Vec::new();
    let (listener, metadata) = bind_private_socket_with(&path, |stage, staged| {
        // The binder cannot advance until these observations finish.
        assert_eq!(
            fs::symlink_metadata(&path).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(
            fs::metadata(staged.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        match stage {
            SocketStage::Created => assert!(!staged.exists()),
            SocketStage::Bound | SocketStage::Secured => assert_eq!(
                fs::metadata(staged).unwrap().permissions().mode() & 0o777,
                if stage == SocketStage::Bound {
                    0o777
                } else {
                    0o600
                }
            ),
        }
        stages.push(stage);
    })
    .unwrap();
    assert_eq!(
        stages,
        [
            SocketStage::Created,
            SocketStage::Bound,
            SocketStage::Secured
        ]
    );
    let published = fs::symlink_metadata(&path).unwrap();
    assert_eq!(published.permissions().mode() & 0o777, 0o600);
    assert_eq!(
        (published.dev(), published.ino()),
        (metadata.dev(), metadata.ino())
    );
    UnixStream::connect(&path).unwrap();
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    drop(listener);
    fs::remove_file(path).unwrap();
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn shared_socket_parents_require_sticky_protection_and_a_trusted_owner() {
    let uid = 1000;
    // In particular, ordinary users must be able to bind in root-owned /tmp.
    for owner in [0, uid] {
        for mode in [0o700, 0o755, 0o1777, 0o1770] {
            check_socket_parent(mode, owner, uid).unwrap();
        }
        for mode in [0o777, 0o770, 0o757] {
            assert_eq!(
                check_socket_parent(mode, owner, uid).unwrap_err().kind(),
                io::ErrorKind::PermissionDenied
            );
        }
    }
    // Even with the sticky bit, a foreign owner could substitute the staging dir.
    for mode in [0o1777, 0o1770, 0o1757] {
        assert_eq!(
            check_socket_parent(mode, uid + 1, uid).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}

fn attachment() -> (AttachedSession, DocumentSession, mpsc::SyncSender<Request>) {
    let project = Project::new(Document::new(Uuid::new_v4(), Uuid::new_v4())).unwrap();
    let document = DocumentSession::new(project.clone(), None, Limits::default()).unwrap();
    let mut session = Session::new(PathBuf::new(), mcp_host);
    session.attach(project).unwrap();
    let (send, requests) = mpsc::sync_channel(16);
    (
        AttachedSession {
            session,
            configured_root: PathBuf::new(),
            palette_enabled: true,
            comments_enabled: true,
            tools: Arc::new(RwLock::new(crate::exports::tools(&mcp_host()))),
            export_formats: mcp_host()
                .registry()
                .formats()
                .map(|(_, f)| f.clone())
                .collect(),
            requests,
            stop: None,
            worker: None,
            path: PathBuf::new(),
            clients: Arc::new(AtomicUsize::new(0)),
            render: None,
            paused: false,
        },
        document,
        send,
    )
}
fn queue(
    send: &mpsc::SyncSender<Request>,
    name: &str,
    arguments: Value,
) -> oneshot::Receiver<Result<CallToolResult, ErrorData>> {
    let (reply, receive) = oneshot::channel();
    send.send(Request {
        name: name.into(),
        arguments,
        reply,
    })
    .unwrap();
    receive
}
fn edit(
    attachment: &AttachedSession,
    document: &DocumentSession,
    revision: u64,
    slot: i32,
) -> Value {
    json!({"model_id":document.engine.document().id,"expected_epoch":attachment.session.epoch,
        "expected_revision":revision.to_string(),"operation_id":Uuid::new_v4(),
        "layer":"Base","voxels":[{"at":[slot,0,0],"slot":1}]})
}
fn result(receive: &mut oneshot::Receiver<Result<CallToolResult, ErrorData>>) -> Value {
    receive
        .try_recv()
        .unwrap()
        .unwrap()
        .structured_content
        .unwrap()
}

#[test]
fn attachment_roots_are_absolute_directories_and_failed_changes_are_atomic() {
    let (mut attachment, _, _) = attachment();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("Documents/Quantum");
    attachment.set_root(&root).unwrap();
    assert!(root.is_dir());
    let canonical = root.canonicalize().unwrap();
    assert_eq!(attachment.session.root, canonical);
    let file = dir.path().join("file.voxl");
    std::fs::write(&file, b"existing").unwrap();
    for invalid in [Path::new("relative-root"), file.as_path()] {
        assert!(attachment.set_root(invalid).is_err());
        assert_eq!(attachment.configured_root, root);
        assert_eq!(attachment.session.root, canonical);
    }
}

#[test]
fn root_and_ancestor_replacement_cannot_redirect_queued_exports() {
    use std::{fs, os::unix::fs::symlink};

    for replace_ancestor in [false, true] {
        let (mut attachment, mut document, send) = attachment();
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let ancestor = dir.path().join("ancestor");
        let root = ancestor.join("root");
        let outside_root = if replace_ancestor {
            outside.path().join("root")
        } else {
            outside.path().to_path_buf()
        };
        fs::create_dir_all(&outside_root).unwrap();
        attachment.set_root(&root).unwrap();
        let established = attachment.session.root.clone();
        let mut export = queue(
            &send,
            "quantum_export_glb",
            json!({"model_id":document.engine.document().id,"path":"escape.glb"}),
        );
        let replaced = if replace_ancestor { &ancestor } else { &root };
        fs::rename(replaced, dir.path().join("original")).unwrap();
        symlink(outside.path(), replaced).unwrap();

        // The editor repeats set_root on every poll, even without a lifecycle change.
        attachment.set_root(&root).unwrap();
        assert_eq!(attachment.session.root, established);
        attachment.poll(&mut document);
        assert_eq!(result(&mut export)["error"]["code"], "path_outside_root");
        assert_eq!(fs::read_dir(outside_root).unwrap().count(), 0);
    }
}

#[test]
fn palette_requests_and_tool_catalog_follow_the_live_editor_host() {
    let (mut attachment, mut document, send) = attachment();
    let mut host = mcp_host();
    let model = document.engine.document().id;
    let before = document.project().to_bytes().unwrap();
    host.disable("dev.quantum.palette").unwrap();
    let mut read = queue(&send, "quantum_palette_list", json!({"model_id":model}));
    let args = json!({"model_id":model,"expected_epoch":attachment.session.epoch,
        "expected_revision":"0","operation_id":Uuid::new_v4(),"format":"hex","text":"112233\n112233"});
    let mut import = queue(&send, "quantum_palette_import", args);
    attachment.poll_with_host(&mut document, &mut host);
    assert_eq!(result(&mut read)["error"]["code"], "palette_unavailable");
    assert_eq!(result(&mut import)["error"]["code"], "palette_unavailable");
    assert!(
        attachment
            .tools
            .read()
            .unwrap()
            .iter()
            .all(|t| !t.name.starts_with("quantum_palette_") || t.name == "quantum_palette_export")
    );
    assert_eq!(document.project().to_bytes().unwrap(), before);
    host.enable("dev.quantum.palette").unwrap();
    let args = json!({"model_id":model,"expected_epoch":attachment.session.epoch,
        "expected_revision":"0","operation_id":Uuid::new_v4(),"format":"hex","text":"112233\n112233"});
    let mut import = queue(&send, "quantum_palette_import", args);
    attachment.poll_with_host(&mut document, &mut host);
    let data = result(&mut import);
    assert_eq!(data["ok"], true);
    assert_eq!(
        data["data"]["slot_mapping"],
        json!([{"source":1,"slot":2},{"source":2,"slot":3}])
    );
    assert_eq!(
        document.engine.document().content.palette.to_hex(),
        "b48c64\n112233\n112233\n"
    );
    assert!(
        attachment
            .tools
            .read()
            .unwrap()
            .iter()
            .any(|t| t.name == "quantum_palette_import")
    );
}

#[test]
fn document_poll_preserves_fifo_budget_cancellation_and_pause() {
    let (mut attachment, mut document, send) = attachment();
    let info = json!({"model_id":document.engine.document().id});
    let mut first = queue(
        &send,
        "quantum_voxels_set",
        edit(&attachment, &document, 0, 0),
    );
    let mut read = queue(&send, "quantum_document_info", info.clone());
    let mut stale = queue(
        &send,
        "quantum_voxels_set",
        edit(&attachment, &document, 0, 1),
    );
    drop(queue(
        &send,
        "quantum_voxels_set",
        edit(&attachment, &document, 1, 2),
    ));
    let mut next = queue(
        &send,
        "quantum_voxels_set",
        edit(&attachment, &document, 1, 3),
    );
    let mut last = queue(&send, "quantum_document_info", info.clone());
    let events = attachment.poll(&mut document);
    assert_eq!(events.len(), 3, "four dequeues include the cancelled call");
    assert_eq!(
        events
            .iter()
            .map(|e| (e.success, e.revision))
            .collect::<Vec<_>>(),
        [(true, 1), (true, 1), (false, 1)]
    );
    assert_eq!(result(&mut first)["ok"], true);
    assert_eq!(result(&mut read)["data"]["revision"], "1");
    assert_eq!(result(&mut stale)["error"]["code"], "revision_conflict");
    assert!(matches!(
        next.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    ));
    assert_eq!(document.engine.revision(), 1);
    let events = attachment.poll(&mut document);
    assert_eq!(events.len(), 2);
    assert_eq!(result(&mut next)["ok"], true);
    assert_eq!(result(&mut last)["data"]["revision"], "2");
    let voxels = &document.engine.document().content.layers[0].voxels;
    assert_eq!(voxels.len(), 2);
    assert_eq!(voxels.get([2, 0, 0]), 0, "cancelled request never commits");
    assert!(document.is_dirty());

    let mut paused = queue(
        &send,
        "quantum_voxels_set",
        edit(&attachment, &document, 2, 4),
    );
    let mut read = queue(&send, "quantum_document_info", info);
    attachment.paused = true; // Pause takes effect before already queued work executes.
    let events = attachment.poll(&mut document);
    assert_eq!(events.len(), 2);
    assert_eq!(result(&mut paused)["error"]["code"], "agent_paused");
    assert_eq!(result(&mut read)["ok"], true);
    assert_eq!(document.engine.revision(), 2);
}

#[test]
fn completed_renders_precede_queued_calls_and_old_epochs_are_discarded() {
    let (mut attachment, mut document, send) = attachment();
    let event = SessionEvent {
        tool: "quantum_render".into(),
        label: "Render".into(),
        success: true,
        revision: 0,
        entry: None,
        changes: Changes::default(),
        error: None,
        inspection: Some(Arc::new(document.engine.document().content.clone())),
        views: vec!["front".into()],
    };
    let old_epoch = attachment.session.epoch;
    let (finished, receive) = mpsc::channel();
    attachment.render = Some(receive);
    finished.send((old_epoch, event.clone())).unwrap();
    let mut edit = queue(
        &send,
        "quantum_voxels_set",
        edit(&attachment, &document, 0, 0),
    );
    let events = attachment.poll(&mut document);
    assert_eq!(
        events.iter().map(|e| e.tool.as_str()).collect::<Vec<_>>(),
        ["quantum_render", "quantum_voxels_set"]
    );
    assert_eq!(events[0].revision, 0);
    assert_eq!(events[1].revision, 1);
    assert_eq!(result(&mut edit)["ok"], true);

    attachment.replace(document.project()).unwrap();
    let (finished, receive) = mpsc::channel();
    attachment.render = Some(receive);
    finished.send((old_epoch, event)).unwrap();
    assert!(attachment.poll(&mut document).is_empty());
    assert_eq!(document.engine.revision(), 1);
}

#[test]
fn attached_reads_share_the_owners_save_state_without_adopting_the_placeholder() {
    use quantum_format::session::SavePolicy;

    let (mut attachment, mut document, send) = attachment();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("project.voxl");
    let info = json!({"model_id":document.engine.document().id});
    let mut unsaved = queue(&send, "quantum_document_info", info.clone());
    attachment.poll(&mut document);
    assert_eq!(result(&mut unsaved)["data"]["dirty"], true);

    let receipt = document
        .prepare_save(&path, SavePolicy::ReplaceSelected)
        .write()
        .unwrap();
    document.finish_save(receipt);
    let mut saved = queue(&send, "quantum_document_info", info.clone());
    let mut capabilities = queue(&send, "quantum_capabilities_get", json!({}));
    attachment.poll(&mut document);
    assert_eq!(result(&mut saved)["data"]["dirty"], false);
    assert_eq!(
        result(&mut capabilities)["data"]["documents"][0]["dirty"],
        false
    );
    assert_eq!(document.path, Some(path.canonicalize().unwrap()));
    assert!(!document.needs_save());

    let mut changed = queue(
        &send,
        "quantum_voxels_set",
        edit(&attachment, &document, 0, 0),
    );
    let mut dirty = queue(&send, "quantum_document_info", info);
    attachment.poll(&mut document);
    assert_eq!(result(&mut changed)["ok"], true);
    assert_eq!(result(&mut dirty)["data"]["dirty"], true);
    assert!(document.is_dirty());
    let receipt = document
        .prepare_save(&path, SavePolicy::ReplaceSelected)
        .write()
        .unwrap();
    document.finish_save(receipt);
    assert!(!document.needs_save());
}

#[test]
fn exports_use_live_registry_and_disablement_removes_tools_without_mutating_document() {
    let (mut attachment, mut document, send) = attachment();
    let root = tempfile::tempdir().unwrap();
    attachment.session = Session::new(root.path().canonicalize().unwrap(), mcp_host);
    attachment.session.attach(document.project()).unwrap();
    let mut host = mcp_host();
    let model = document.engine.document().id;
    let before = document.project().to_bytes().unwrap();
    let revision = document.engine.revision();
    for (tool, ext) in [("quantum_export_glb", "glb"), ("quantum_export_png", "png")] {
        let path = format!("enabled.{ext}");
        let args = if ext == "png" {
            json!({"model_id":model,"path":path,"views":["iso"],"width":32,"height":32})
        } else {
            json!({"model_id":model,"path":path})
        };
        let mut receive = queue(&send, tool, args.clone());
        attachment.poll_with_host(&mut document, &mut host);
        assert_eq!(result(&mut receive)["ok"], true);
        assert!(root.path().join(&path).exists());
        host.disable("dev.quantum.exporters").unwrap();
        let mut args = args;
        args["path"] = json!(format!("disabled.{ext}"));
        let mut receive = queue(&send, tool, args);
        attachment.poll_with_host(&mut document, &mut host);
        assert_eq!(result(&mut receive)["error"]["code"], "export_unavailable");
        assert!(!root.path().join(format!("disabled.{ext}")).exists());
        assert!(
            attachment
                .tools
                .read()
                .unwrap()
                .iter()
                .all(|t| !t.name.starts_with("quantum_export_"))
        );
        assert!(
            attachment
                .tools
                .read()
                .unwrap()
                .iter()
                .any(|t| t.name == "quantum_palette_export")
        );
        let mut receive = queue(
            &send,
            "quantum_palette_export",
            json!({"model_id":model,"format":"pal"}),
        );
        attachment.poll_with_host(&mut document, &mut host);
        assert_eq!(result(&mut receive)["ok"], true);
        host.enable("dev.quantum.exporters").unwrap();
    }
    attachment.poll_with_host(&mut document, &mut host);
    assert!(
        attachment
            .tools
            .read()
            .unwrap()
            .iter()
            .any(|t| t.name == "quantum_export_png")
    );
    assert_eq!(document.project().to_bytes().unwrap(), before);
    assert_eq!(document.engine.revision(), revision);
}

#[test]
fn comment_requests_and_catalog_follow_live_host_without_losing_core_data() {
    let (mut attachment, mut document, send) = attachment();
    let mut host = mcp_host();
    let model = document.engine.document().id;
    let thread = Uuid::new_v4();
    let message = Uuid::new_v4();
    let args = json!({"model_id":model,"expected_epoch":attachment.session.epoch,
        "expected_revision":"0","operation_id":Uuid::new_v4(),
        "thread_id":thread,"message_id":message,"anchor":{"kind":"voxel","at":[0,0,0]},"text":"Keep this thread"});
    let mut add = queue(&send, "quantum_comments_add", args.clone());
    attachment.poll_with_host(&mut document, &mut host);
    let added = result(&mut add);
    assert_eq!(added["ok"], true, "{added}");
    assert_eq!(added["data"]["thread_id"], thread.to_string());
    let before = document.project().to_bytes().unwrap();
    let history = document.engine.history().count();
    host.disable(quantum_plugin_comments::PLUGIN_ID).unwrap();
    let mut read = queue(&send, "quantum_comments_list", json!({"model_id":model}));
    let mut reply_args = args;
    reply_args["expected_revision"] = document.engine.revision().to_string().into();
    reply_args["operation_id"] = Uuid::new_v4().to_string().into();
    reply_args.as_object_mut().unwrap().remove("anchor");
    reply_args["message_id"] = Uuid::new_v4().to_string().into();
    let mut reply = queue(&send, "quantum_comments_reply", reply_args);
    attachment.poll_with_host(&mut document, &mut host);
    assert_eq!(result(&mut read)["error"]["code"], "comments_unavailable");
    assert_eq!(result(&mut reply)["error"]["code"], "comments_unavailable");
    assert!(
        attachment
            .tools
            .read()
            .unwrap()
            .iter()
            .all(|t| !t.name.starts_with("quantum_comments_"))
    );
    assert_eq!(document.project().to_bytes().unwrap(), before);
    assert_eq!(document.engine.history().count(), history);
    host.enable(quantum_plugin_comments::PLUGIN_ID).unwrap();
    let mut read = queue(&send, "quantum_comments_list", json!({"model_id":model}));
    attachment.poll_with_host(&mut document, &mut host);
    let data = result(&mut read);
    assert_eq!(
        data["data"]["comments"][0]["thread"]["messages"][0]["text"],
        "Keep this thread"
    );
    assert_eq!(
        data["data"]["comments"][0]["thread"]["messages"][0]["origin"],
        "agent"
    );
    assert!(
        attachment
            .tools
            .read()
            .unwrap()
            .iter()
            .any(|t| t.name == "quantum_comments_add")
    );
    assert_eq!(document.project().to_bytes().unwrap(), before);
}
