use super::*;
use quantum_core::Uuid;
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

// Isolate untitled exports without changing the process environment or user files.
pub(super) fn home_dir() -> Option<PathBuf> {
    thread_local! {
        static HOME_DIR: tempfile::TempDir = tempfile::tempdir().unwrap();
    }
    HOME_DIR.with(|home| Some(home.path().to_path_buf()))
}

struct Client {
    stream: BufReader<UnixStream>,
    id: u64,
}
impl Client {
    fn connect(attachment: &mut AttachedSession, editor: &mut Editor) -> Self {
        let stream = UnixStream::connect(attachment.path()).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_millis(10)))
            .unwrap();
        let mut client = Self {
            stream: BufReader::new(stream),
            id: 0,
        };
        client.send(
            "initialize",
            json!({"protocolVersion":"2025-11-25","capabilities":{},
            "clientInfo":{"name":"editor-lifecycle-test","version":"1"}}),
        );
        assert_eq!(
            client.reply(attachment, editor)["protocolVersion"],
            "2025-11-25"
        );
        writeln!(
            client.stream.get_mut(),
            "{}",
            json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .unwrap();
        client
    }

    fn send(&mut self, method: &str, params: Value) {
        self.id += 1;
        writeln!(
            self.stream.get_mut(),
            "{}",
            json!({"jsonrpc":"2.0","id":self.id,"method":method,"params":params})
        )
        .unwrap();
    }

    fn queue(&mut self, name: &str, arguments: Value) {
        self.send("tools/call", json!({"name":name,"arguments":arguments}));
    }

    fn reply(&mut self, attachment: &mut AttachedSession, editor: &mut Editor) -> Value {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut line = String::new();
        loop {
            editor.poll_attachment(attachment).unwrap();
            match self.stream.read_line(&mut line) {
                Ok(0) => panic!("attachment closed before replying"),
                Ok(_) => {
                    let reply: Value = serde_json::from_str(&line).unwrap();
                    assert_eq!(reply["id"], self.id);
                    assert!(reply.get("error").is_none(), "{reply}");
                    return reply["result"].clone();
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => panic!("reading attachment reply: {error}"),
            }
            assert!(Instant::now() < deadline, "attachment reply timed out");
        }
    }

    fn result(&mut self, attachment: &mut AttachedSession, editor: &mut Editor) -> Value {
        let result = self.reply(attachment, editor)["structuredContent"].clone();
        assert_eq!(result["ok"], true, "{result}");
        result
    }

    fn export(&mut self, editor: &Editor, name: &str) {
        self.queue(
            "quantum_export_glb",
            json!({"model_id":editor.session.engine.document().id,"path":name}),
        );
    }
}

#[test]
fn untitled_root_is_dedicated_and_resolved_without_creating_it() {
    let home = tempfile::tempdir().unwrap();
    let root = file_root(None, Some(home.path())).unwrap();
    assert_eq!(root, home.path().join("Documents/Quantum"));
    assert!(!root.exists());
    assert!(file_root(None, None).is_err());
    assert!(file_root(None, Some(Path::new("relative-home"))).is_err());
    assert!(file_root(Some(Path::new("relative.voxl")), Some(home.path())).is_err());
    assert_eq!(
        file_root(Some(&home.path().join("project.voxl")), None).unwrap(),
        home.path()
    );
}

#[test]
fn queued_exports_follow_save_as_failed_save_open_and_new() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let mut editor = Editor::blank().unwrap();
    let untitled = editor.attachment_root().unwrap();
    let mut attachment = AttachedSession::bind(
        &untitled,
        editor.project(),
        &first.path().join("app.sock"),
        quantum_plugins_builtin::mcp_host,
    )
    .unwrap();
    assert!(untitled.is_dir());
    let mut client = Client::connect(&mut attachment, &mut editor);
    client.queue("quantum_capabilities_get", json!({}));
    let epoch = client.result(&mut attachment, &mut editor)["epoch"].clone();
    let edit = json!({"model_id":editor.session.engine.document().id,"expected_epoch":epoch,
        "expected_revision":"0","operation_id":Uuid::new_v4(),"layer":"Base","voxels":[{"at":[0,0,0],"slot":1}]});
    client.queue("quantum_voxels_set", edit.clone());
    let receipt = client.result(&mut attachment, &mut editor);
    assert_eq!(editor.session.engine.revision(), 1);
    let first_path = first.path().join("first.voxl");
    client.export(&editor, "first.glb");
    editor.save(&first_path).unwrap();
    let exported = client.result(&mut attachment, &mut editor);
    assert_eq!(exported["epoch"], epoch);
    assert!(first.path().join("first.glb").is_file());
    assert!(!untitled.join("first.glb").exists());
    // A saved project does not depend on a home directory being available.
    assert_eq!(
        file_root(editor.path.as_deref(), None).unwrap(),
        editor.attachment_root().unwrap()
    );

    client.export(&editor, "second.glb");
    editor.save(&second.path().join("second.voxl")).unwrap();
    assert_eq!(client.result(&mut attachment, &mut editor)["epoch"], epoch);
    assert!(second.path().join("second.glb").is_file());
    assert!(!first.path().join("second.glb").exists());

    client.queue("quantum_voxels_set", edit.clone());
    assert_eq!(client.result(&mut attachment, &mut editor), receipt);
    assert_eq!(editor.session.engine.revision(), 1);

    // An unavailable new root must stop polling instead of reusing the old root.
    let saved_path = editor.session.path.take();
    editor.session.path = Some(PathBuf::from("relative.voxl"));
    assert!(editor.poll_attachment(&mut attachment).is_err());
    editor.session.path = Some(second.path().join("second.voxl/invalid.voxl"));
    assert!(editor.poll_attachment(&mut attachment).is_err());
    editor.session.path = saved_path;

    client.export(&editor, "failed-save.glb");
    assert!(
        editor
            .save(&first.path().join("missing/failure.voxl"))
            .is_err()
    );
    assert_eq!(client.result(&mut attachment, &mut editor)["epoch"], epoch);
    assert!(second.path().join("failed-save.glb").is_file());
    assert!(!first.path().join("missing").exists());
    client.queue("quantum_voxels_set", edit);
    assert_eq!(client.result(&mut attachment, &mut editor), receipt);

    client.export(&editor, "opened.glb");
    editor = Editor::open(&first_path).unwrap();
    // The native shell replaces the attachment on New/Open before polling.
    attachment.replace(editor.project()).unwrap();
    let opened_epoch = client.result(&mut attachment, &mut editor)["epoch"].clone();
    assert_ne!(opened_epoch, epoch);
    assert!(first.path().join("opened.glb").is_file());
    assert!(!second.path().join("opened.glb").exists());

    let blank = Editor::blank().unwrap();
    client.export(&blank, "new.glb");
    editor = blank;
    attachment.replace(editor.project()).unwrap();
    let new_epoch = client.result(&mut attachment, &mut editor)["epoch"].clone();
    assert_ne!(new_epoch, opened_epoch);
    assert_ne!(new_epoch, epoch);
    assert!(untitled.join("new.glb").is_file());
    assert!(!first.path().join("new.glb").exists());
    assert!(!second.path().join("new.glb").exists());
}
