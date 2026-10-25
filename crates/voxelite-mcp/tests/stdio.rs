//! Real process tests: no SDK client can accidentally mask wire regressions.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    time::Duration,
};
use voxelite_core::Uuid;

struct Client {
    child: Child,
    input: Option<ChildStdin>,
    output: mpsc::Receiver<Value>,
    next: u64,
    modern: bool,
}
impl Client {
    fn start(root: &std::path::Path, modern: bool) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_voxelite-mcp"))
            .arg("--root")
            .arg(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, output) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let line = line.unwrap();
                let value = serde_json::from_str(&line).expect("stdout must contain only JSON-RPC");
                if tx.send(value).is_err() {
                    break;
                }
            }
        });
        let mut client = Self {
            child,
            input,
            output,
            next: 1,
            modern,
        };
        if !modern {
            let init=client.rpc("initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"wire-test","version":"1.0"}}));
            assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
            assert!(init["result"]["capabilities"]["tools"].is_object());
            client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        }
        client
    }
    fn send(&mut self, v: Value) {
        self.raw(&format!("{v}\n"));
    }
    fn raw(&mut self, s: &str) {
        let input = self.input.as_mut().unwrap();
        input.write_all(s.as_bytes()).unwrap();
        input.flush().unwrap();
    }
    fn receive(&self) -> Value {
        self.output
            .recv_timeout(Duration::from_secs(30))
            .expect("server reply within 30 seconds")
    }
    fn rpc(&mut self, method: &str, mut params: Value) -> Value {
        let id = self.next;
        self.next += 1;
        if self.modern {
            params["_meta"] = json!({"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"wire-test","version":"1.0"},"io.modelcontextprotocol/clientCapabilities":{}});
        }
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let result = self.receive();
        assert_eq!(result["id"], id, "{result}");
        assert_eq!(result["jsonrpc"], "2.0");
        result
    }
    fn call(&mut self, name: &str, args: Value) -> Value {
        let wire = self.rpc(
            "tools/call",
            json!({"name":format!("voxelite_{name}"),"arguments":args}),
        );
        assert!(wire.get("error").is_none(), "{wire}");
        let result = wire["result"].clone();
        assert_eq!(result["isError"], false, "{result}");
        let text: Value =
            serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(text, result["structuredContent"]);
        assert_eq!(text["api_version"], "voxelite.v1");
        if self.modern {
            assert_eq!(result["resultType"], "complete");
        } else {
            assert!(result.get("resultType").is_none());
        }
        result
    }
    fn exit(mut self) {
        self.input.take();
        for _ in 0..100 {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("server did not exit after stdin EOF");
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn data(result: &Value) -> Value {
    result["structuredContent"]["data"].clone()
}
fn edit(c: &mut Client, name: &str, model: &Value, epoch: &Value, mut args: Value) -> Value {
    let info = data(&c.call("document_info", json!({"model_id":model})));
    args["model_id"] = model.clone();
    args["expected_epoch"] = epoch.clone();
    args["operation_id"] = json!(Uuid::new_v4());
    args["expected_revision"] = info["revision"].clone();
    data(&c.call(name, args))
}
fn workflow(modern: bool) {
    let root = tempfile::tempdir().unwrap();
    let mut c = Client::start(root.path(), modern);
    let list = c.rpc("tools/list", json!({}));
    assert_eq!(list["result"]["tools"].as_array().unwrap().len(), 38);
    let caps = c.call("capabilities_get", json!({}));
    let epoch = caps["structuredContent"]["epoch"].clone();
    let create = c.call(
        "document_create",
        json!({"name":"Wire art","expected_epoch":epoch,"operation_id":Uuid::new_v4()}),
    );
    let model = data(&create)["model_id"].clone();
    let batch = edit(
        &mut c,
        "voxels_batch",
        &model,
        &epoch,
        json!({"operations":[{"kind":"box","layer":"Base","region":{"min":[1,1,1],"max":[5,4,5]},"slot":1},{"kind":"set","layer":"Base","voxels":[{"at":[2,4,2],"slot":1}]}]}),
    );
    assert_eq!(batch["added"], 49);
    let render=c.call("render",json!({"model_id":model,"views":["iso","front","right","top"],"width":96,"height":80,"image_mode":"both"}));
    let images: Vec<_> = render["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["type"] == "image")
        .collect();
    assert_eq!(images.len(), 5);
    for (i, image) in images.iter().enumerate() {
        assert_eq!(image["mimeType"], "image/png");
        let bytes = STANDARD.decode(image["data"].as_str().unwrap()).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
            .read_info()
            .unwrap();
        let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
        let out = reader.next_frame(&mut buffer).unwrap();
        assert_eq!(
            (out.width, out.height),
            if i == 0 { (192, 160) } else { (96, 80) }
        );
        assert!(buffer.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    }
    edit(
        &mut c,
        "history_undo",
        &model,
        &epoch,
        json!({"entry_id":batch["undo_entry_id"]}),
    );
    assert_eq!(
        data(&c.call("document_info", json!({"model_id":model})))["voxel_count"],
        0
    );
    edit(
        &mut c,
        "history_redo",
        &model,
        &epoch,
        json!({"entry_id":batch["undo_entry_id"]}),
    );
    let comment = edit(
        &mut c,
        "comments_add",
        &model,
        &epoch,
        json!({"anchor":{"kind":"region","min":[1,1,1],"max":[5,4,5]},"text":"Inspect the rear"}),
    );
    edit(
        &mut c,
        "comments_reply",
        &model,
        &epoch,
        json!({"thread_id":comment["thread_id"],"text":"Rear silhouette checked"}),
    );
    edit(
        &mut c,
        "comments_resolve",
        &model,
        &epoch,
        json!({"thread_id":comment["thread_id"]}),
    );
    edit(
        &mut c,
        "versions_save",
        &model,
        &epoch,
        json!({"name":"Reviewed"}),
    );
    let saved = edit(
        &mut c,
        "document_save",
        &model,
        &epoch,
        json!({"path":"wire.voxl"}),
    );
    assert_eq!(saved["dirty"], false);
    c.call(
        "export_png",
        json!({"model_id":model,"path":"wire.png","views":["iso"],"width":64,"height":64}),
    );
    c.call("export_glb", json!({"model_id":model,"path":"wire.glb"}));
    assert_eq!(
        &std::fs::read(root.path().join("wire.glb")).unwrap()[..4],
        b"glTF"
    );
    let bytes = std::fs::read(root.path().join("wire.voxl")).unwrap();
    c.exit();
    let mut c = Client::start(root.path(), modern);
    let epoch = c.call("capabilities_get", json!({}))["structuredContent"]["epoch"].clone();
    let reopened = data(&c.call(
        "document_open",
        json!({"path":"wire.voxl","expected_epoch":epoch,"operation_id":Uuid::new_v4()}),
    ));
    assert_eq!(reopened["model_id"], model);
    assert_eq!(reopened["voxel_count"], 49);
    assert_eq!(reopened["versions"], 1);
    let comments = data(&c.call("comments_list", json!({"model_id":model})));
    assert_eq!(comments["comments"][0]["thread"]["resolved"], true);
    assert_eq!(
        comments["comments"][0]["thread"]["messages"][1]["origin"],
        "agent"
    );
    edit(
        &mut c,
        "document_save",
        &model,
        &epoch,
        json!({"path":"wire.voxl"}),
    );
    assert_eq!(std::fs::read(root.path().join("wire.voxl")).unwrap(), bytes);
    c.exit();
}
#[test]
fn legacy_initialize_and_complete_editing_workflow() {
    workflow(false);
}
#[test]
fn modern_independent_requests_and_complete_editing_workflow() {
    workflow(true);
}
#[test]
fn protocol_errors_and_domain_errors_are_distinct() {
    let root = tempfile::tempdir().unwrap();
    let mut c = Client::start(root.path(), false);
    let unknown = c.rpc("not/a/method", json!({}));
    assert_eq!(unknown["error"]["code"], -32601);
    let unknown = c.rpc("tools/call", json!({"name":"absent","arguments":{}}));
    assert_eq!(unknown["error"]["code"], -32602);
    let invalid = c.rpc(
        "tools/call",
        json!({"name":"voxelite_document_create","arguments":{"unexpected":true}}),
    );
    assert_eq!(invalid["result"]["isError"], true);
    assert_eq!(
        invalid["result"]["structuredContent"]["error"]["code"],
        "invalid_argument"
    );
    c.raw("{bad json}\n");
    assert_eq!(c.receive()["error"]["code"], -32700);
    c.send(json!({"jsonrpc":"1.0","id":200,"method":"ping"}));
    assert_eq!(c.receive()["error"]["code"], -32600);
    c.raw("[]\n");
    assert_eq!(c.receive()["error"]["code"], -32600);
    c.raw(&(" ".repeat(16 * 1024 * 1024) + "\n"));
    assert_eq!(c.receive()["error"]["code"], -32600);
    c.send(json!({"jsonrpc":"2.0","method":"notifications/unknown"}));
    let ping = c.rpc("ping", json!({}));
    assert!(ping["result"].is_object());
    c.exit();
}

#[test]
fn newer_profile_rejects_missing_and_unsupported_metadata_then_recovers() {
    let root = tempfile::tempdir().unwrap();
    let mut c = Client::start(root.path(), true);
    c.send(json!({"jsonrpc":"2.0","id":99,"method":"tools/list","params":{}}));
    let missing = c.receive();
    assert_eq!(missing["id"], 99);
    assert!(missing["error"].is_object());
    c.send(json!({"jsonrpc":"2.0","id":100,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2099-01-01","io.modelcontextprotocol/clientInfo":{"name":"wire-test","version":"1.0"},"io.modelcontextprotocol/clientCapabilities":{}}}}));
    let unsupported = c.receive();
    assert_eq!(unsupported["id"], 100);
    assert!(unsupported["error"].is_object());
    let tools = c.rpc("tools/list", json!({}));
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 38);
    c.exit();
}
