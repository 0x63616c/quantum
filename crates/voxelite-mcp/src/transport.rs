//! Bounded newline framing around rmcp's protocol/lifecycle implementation.
//!
//! rmcp's default transport deliberately ignores malformed JSON. This adapter
//! instead returns JSON-RPC parse/invalid-request errors, keeps partial frames
//! across cancelled receive futures, and finishes each output frame even when
//! the awaiting caller is cancelled.
use rmcp::{
    RoleServer,
    model::{
        CallToolRequestParams, ClientJsonRpcMessage, ErrorCode, ErrorData, InitializeRequestParams,
        PaginatedRequestParams, RequestId, ServerJsonRpcMessage,
    },
    transport::Transport,
};
use serde_json::Value;
use std::{io, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::Mutex,
};

const MAX_LINE: usize = 16 * 1024 * 1024;

pub(crate) struct JsonLines<R, W> {
    read: BufReader<R>,
    write: Arc<Mutex<W>>,
    partial: Vec<u8>,
    oversized: bool,
}
impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin + Send + 'static> JsonLines<R, W> {
    pub fn new(read: R, write: W) -> Self {
        Self {
            read: BufReader::new(read),
            write: Arc::new(Mutex::new(write)),
            partial: Vec::new(),
            oversized: false,
        }
    }
    async fn frame(&mut self) -> io::Result<Option<Vec<u8>>> {
        loop {
            let available = self.read.fill_buf().await?;
            if available.is_empty() {
                return Ok(None);
            }
            let newline = available.iter().position(|b| *b == b'\n');
            let take = newline.map_or(available.len(), |n| n + 1);
            if !self.oversized {
                if self.partial.len() + take > MAX_LINE {
                    self.partial.clear();
                    self.oversized = true;
                } else {
                    self.partial.extend_from_slice(&available[..take]);
                }
            }
            self.read.consume(take);
            if newline.is_some() {
                if self.oversized {
                    self.oversized = false;
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "request line exceeds 16 MiB",
                    ));
                }
                return Ok(Some(std::mem::take(&mut self.partial)));
            }
        }
    }
    fn output(
        &self,
        item: ServerJsonRpcMessage,
    ) -> impl Future<Output = io::Result<()>> + Send + 'static {
        let write = self.write.clone();
        // A detached write must finish its frame; dropping a write_all future
        // halfway through would corrupt the next newline-delimited response.
        let task = tokio::spawn(async move {
            let mut value = serde_json::to_value(&item).map_err(io::Error::other)?;
            // rmcp omits an absent error ID; JSON-RPC requires an explicit null
            // when a malformed message cannot be correlated with a request.
            if matches!(item, ServerJsonRpcMessage::Error(_)) && value.get("id").is_none() {
                value["id"] = Value::Null;
            }
            let mut bytes = serde_json::to_vec(&value).map_err(io::Error::other)?;
            bytes.push(b'\n');
            let mut write = write.lock().await;
            write.write_all(&bytes).await?;
            write.flush().await
        });
        async move { task.await.map_err(io::Error::other)? }
    }
}
impl<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin + Send + 'static> Transport<RoleServer>
    for JsonLines<R, W>
{
    type Error = io::Error;
    fn send(
        &mut self,
        item: ServerJsonRpcMessage,
    ) -> impl Future<Output = io::Result<()>> + Send + 'static {
        self.output(item)
    }
    async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
        loop {
            let frame = match self.frame().await {
                Ok(Some(frame)) => frame,
                Ok(None) => return None,
                Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                    self.output(ServerJsonRpcMessage::error(
                        ErrorData::invalid_request("request line exceeds 16 MiB", None),
                        None,
                    ))
                    .await
                    .ok()?;
                    continue;
                }
                Err(_) => return None,
            };
            if frame.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let value: Value = match serde_json::from_slice(&frame) {
                Ok(v) => v,
                Err(_) => {
                    self.output(ServerJsonRpcMessage::error(
                        ErrorData::new(ErrorCode::PARSE_ERROR, "Parse error", None),
                        None,
                    ))
                    .await
                    .ok()?;
                    continue;
                }
            };
            let id: Option<RequestId> = value
                .get("id")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            let valid = value.is_object()
                && value["jsonrpc"] == "2.0"
                && (value.get("id").is_none() || id.is_some());
            let method = value.get("method").and_then(Value::as_str);
            if valid && let Some(method) = method {
                if value.get("id").is_none() {
                    // Invalid notification parameters must not turn a one-way
                    // message into an uncorrelated response. Valid notifications
                    // still reach rmcp for lifecycle/cancellation handling.
                    if !valid_params(method, value.get("params")) {
                        continue;
                    }
                } else if !valid_params(method, value.get("params")) {
                    self.output(ServerJsonRpcMessage::error(
                        ErrorData::invalid_params("Invalid MCP method parameters", None),
                        id,
                    ))
                    .await
                    .ok()?;
                    continue;
                }
            }
            let notification = valid && method.is_some() && value.get("id").is_none();
            let parsed = if valid {
                serde_json::from_value(value).ok()
            } else {
                None
            };
            if let Some(message) = parsed {
                return Some(message);
            }
            if notification {
                continue;
            }
            self.output(ServerJsonRpcMessage::error(
                ErrorData::invalid_request("Invalid JSON-RPC request", None),
                id,
            ))
            .await
            .ok()?;
        }
    }
    async fn close(&mut self) -> io::Result<()> {
        self.write.lock().await.shutdown().await
    }
}

/// Validate wire parameters before rmcp's custom-method fallback can obscure a
/// malformed known request or discard optional pagination fields. Unknown
/// methods remain the SDK's responsibility, including version negotiation.
fn valid_params(method: &str, params: Option<&Value>) -> bool {
    let empty = serde_json::json!({});
    let params = params.unwrap_or(&empty);
    if !params.is_object() || params.get("_meta").is_some_and(|m| !m.is_object()) {
        return false;
    }
    match method {
        "initialize" => serde_json::from_value::<InitializeRequestParams>(params.clone()).is_ok(),
        "tools/list" => {
            params.get("cursor").is_none_or(Value::is_string)
                && serde_json::from_value::<PaginatedRequestParams>(params.clone()).is_ok()
        }
        "tools/call" => {
            params.get("arguments").is_none_or(Value::is_object)
                // No task-augmented execution is advertised by this server.
                && params.get("task").is_none()
                && serde_json::from_value::<CallToolRequestParams>(params.clone()).is_ok()
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn partial_frames_survive_cancellation_and_crlf() {
        let (mut input, read) = tokio::io::duplex(1024);
        let (write, _output) = tokio::io::duplex(1024);
        let mut transport = JsonLines::new(read, write);
        input.write_all(b"{\"jsonrpc\":\"2.0\",").await.unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(5), transport.receive())
                .await
                .is_err()
        );
        input
            .write_all(b"\"id\":1,\"method\":\"ping\"}\r\n")
            .await
            .unwrap();
        assert!(matches!(
            transport.receive().await,
            Some(ClientJsonRpcMessage::Request(_))
        ));
    }
}
