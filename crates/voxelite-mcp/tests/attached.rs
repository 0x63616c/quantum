//! A real stdio bridge process edits the same engine used by the native app.
#![cfg(all(unix, feature = "standalone"))]
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
use voxelite_core::*;
use voxelite_format::{Project, session::DocumentSession};
use voxelite_mcp::attached::AttachedSession;
use voxelite_plugins_builtin::mcp_host;
#[test]
fn live_socket_shares_history_conflicts_pause_and_receipts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("app.sock");
    let doc = Document::new(Uuid::new_v4(), Uuid::new_v4());
    let model = doc.id;
    let layer = doc.content.layers[0].id;
    let mut owner =
        DocumentSession::new(Project::new(doc.clone()).unwrap(), None, Limits::default()).unwrap();
    let mut session =
        AttachedSession::bind(dir.path(), Project::new(doc).unwrap(), &path, mcp_host).unwrap();
    assert!(
        AttachedSession::bind(
            dir.path(),
            Project::new(owner.engine.document().clone()).unwrap(),
            &path,
            mcp_host,
        )
        .is_err()
    );
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_voxelite-mcp"))
        .args(["--connect", path.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if tx
                .send(serde_json::from_str::<Value>(&line.unwrap()).unwrap())
                .is_err()
            {
                break;
            }
        }
    });
    let mut id = 0;
    let mut rpc = |method: &str,
                   params: Value,
                   session: &mut AttachedSession,
                   document: &mut DocumentSession| {
        id += 1;
        writeln!(
            stdin,
            "{}",
            json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
        )
        .unwrap();
        stdin.flush().unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut events = vec![];
        loop {
            events.extend(session.poll(document));
            if let Ok(reply) = rx.try_recv() {
                assert_eq!(reply["id"], id);
                return (reply, events);
            }
            assert!(Instant::now() < deadline, "reply timeout");
            std::thread::sleep(Duration::from_millis(2));
        }
    };
    let (init, _) = rpc(
        "initialize",
        json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test-agent","version":"1"}}),
        &mut session,
        &mut owner,
    );
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
    // rmcp accepts tool requests after its initialize response.
    let (cap, _) = rpc(
        "tools/call",
        json!({"name":"voxelite_capabilities_get","arguments":{}}),
        &mut session,
        &mut owner,
    );
    let envelope = &cap["result"]["structuredContent"];
    assert_eq!(
        envelope["data"]["documents"][0]["model_id"],
        model.to_string()
    );
    let epoch = envelope["epoch"].clone();
    let op = Uuid::new_v4();
    let args = json!({"model_id":model,"expected_epoch":epoch,"expected_revision":"0","operation_id":op,"layer":layer,"voxels":[{"at":[1,2,3],"slot":1}],"label":"Agent placed a voxel"});
    let (mut response, events) = rpc(
        "tools/call",
        json!({"name":"voxelite_voxels_set","arguments":args}),
        &mut session,
        &mut owner,
    );
    assert_eq!(response["result"]["isError"], false, "{response}");
    assert_eq!(
        owner.engine.document().content.layers[0]
            .voxels
            .get([1, 2, 3]),
        1
    );
    assert_eq!(
        owner.engine.history().next().unwrap().actor.origin,
        Origin::Agent
    );
    assert_eq!(events[0].changes.added, 1);
    let revision = owner.engine.revision();
    let (_, events) = rpc(
        "tools/call",
        json!({"name":"voxelite_voxels_set","arguments":args}),
        &mut session,
        &mut owner,
    );
    assert_eq!(owner.engine.revision(), revision);
    assert_eq!(events[0].changes.voxel_count(), 0);
    assert!(events[0].entry.is_none());
    owner
        .engine
        .execute(Transaction {
            expected_revision: revision,
            id: Uuid::new_v4(),
            actor: Actor::new(Origin::Human),
            label: "Human rename".into(),
            command: voxelite_core::Command::EditDocument {
                name: Some("Together".into()),
                bounds: None,
            },
        })
        .unwrap();
    let mut stale = args.clone();
    stale["operation_id"] = json!(Uuid::new_v4());
    stale["expected_revision"] = json!(revision.to_string());
    (response, _) = rpc(
        "tools/call",
        json!({"name":"voxelite_voxels_set","arguments":stale}),
        &mut session,
        &mut owner,
    );
    assert_eq!(
        response["result"]["structuredContent"]["error"]["code"],
        "revision_conflict"
    );
    let head = owner.engine.head().unwrap();
    let (undone, undo_events) = rpc(
        "tools/call",
        json!({"name":"voxelite_history_undo","arguments":{
            "model_id":model,"expected_epoch":epoch,"expected_revision":owner.engine.revision().to_string(),"operation_id":Uuid::new_v4(),"entry_id":head
        }}),
        &mut session,
        &mut owner,
    );
    assert_eq!(undone["result"]["isError"], false, "{undone}");
    assert!(
        undo_events[0].entry.is_none(),
        "Undo must not offer to revert the remaining head"
    );
    let (redone, redo_events) = rpc(
        "tools/call",
        json!({"name":"voxelite_history_redo","arguments":{
            "model_id":model,"expected_epoch":epoch,"expected_revision":owner.engine.revision().to_string(),"operation_id":Uuid::new_v4(),"entry_id":head
        }}),
        &mut session,
        &mut owner,
    );
    assert_eq!(redone["result"]["isError"], false, "{redone}");
    assert!(redo_events[0].entry.is_none());
    session.paused = true;
    let mut paused = args;
    paused["operation_id"] = json!(Uuid::new_v4());
    paused["expected_revision"] = json!(owner.engine.revision().to_string());
    (response, _) = rpc(
        "tools/call",
        json!({"name":"voxelite_voxels_set","arguments":paused}),
        &mut session,
        &mut owner,
    );
    assert_eq!(
        response["result"]["structuredContent"]["error"]["code"],
        "agent_paused"
    );
    assert_eq!(owner.engine.document().content.name, "Together");
    assert_eq!(owner.engine.history().count(), 2);
    session.paused = false;
    session
        .replace(Project::new(owner.engine.document().clone()).unwrap())
        .unwrap();
    let stale_epoch = json!({"model_id":model,"expected_epoch":epoch,"expected_revision":owner.engine.revision().to_string(),"operation_id":Uuid::new_v4(),"layer":layer,"voxels":[{"at":[2,2,3],"slot":1}]});
    let (response, _) = rpc(
        "tools/call",
        json!({"name":"voxelite_voxels_set","arguments":stale_epoch}),
        &mut session,
        &mut owner,
    );
    assert_eq!(
        response["result"]["structuredContent"]["error"]["code"],
        "epoch_conflict"
    );
    assert_eq!(
        owner.engine.document().content.layers[0]
            .voxels
            .get([2, 2, 3]),
        0
    );
    let rendered_revision = owner.engine.revision();
    let (rendered, mut render_events) = rpc(
        "tools/call",
        json!({"name":"voxelite_render","arguments":{
            "model_id":model,"revision":rendered_revision.to_string(),"views":["front","right","back"],"width":64,"height":64
        }}),
        &mut session,
        &mut owner,
    );
    render_events.extend(session.poll(&mut owner));
    assert_eq!(rendered["result"]["isError"], false, "{rendered}");
    assert_eq!(
        rendered["result"]["structuredContent"]["data"]["revision"],
        rendered_revision.to_string()
    );
    let capture = render_events
        .iter()
        .find(|e| e.inspection.is_some())
        .expect("actual render event");
    assert_eq!(capture.views, ["front", "right", "back"]);
    assert_eq!(
        capture.inspection.as_ref().unwrap().layers[0]
            .voxels
            .get([1, 2, 3]),
        1
    );
    assert_eq!(owner.engine.revision(), rendered_revision);
    drop(rpc);
    drop(stdin);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(s) = child.try_wait().unwrap() {
            assert!(s.success());
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(session);
    assert!(!path.exists(), "normal shutdown removes the owned socket");
}
