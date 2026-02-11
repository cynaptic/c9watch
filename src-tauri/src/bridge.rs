//! SDK Bridge — manages a long-lived Node.js process that wraps the
//! Claude Agent SDK V2, providing persistent multi-turn sessions with
//! no cold-start after the first message.
//!
//! Protocol (stdin → bridge):
//!   { "cmd":"resume", "id":"<uuid>", "sessionId":"<id>", "cwd":"<path>" }
//!   { "cmd":"send",   "id":"<uuid>", "sessionId":"<id>", "message":"<text>" }
//!   { "cmd":"close",  "id":"<uuid>", "sessionId":"<id>" }
//!
//! Protocol (bridge → stdout):
//!   { "type":"ack",         "id":"<uuid>", "success":true/false, "error":"..." }
//!   { "type":"streamEvent", "sessionId":"<id>", "data":{...} }
//!   { "type":"streamEnd",   "sessionId":"<id>", "success":true/false, "error":"..." }

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, oneshot, Mutex};

/// Handle to the SDK bridge, shareable across threads.
pub type SdkBridgeHandle = Arc<SdkBridge>;

/// Manages the Node.js bridge child process.
pub struct SdkBridge {
    inner: Mutex<BridgeInner>,
    /// Broadcast channel for stream events (serialized JSON strings).
    /// Subscribers (Tauri event forwarder, WS connections) receive these.
    event_tx: broadcast::Sender<String>,
}

struct BridgeInner {
    child: Option<Child>,
    stdin: Option<tokio::process::ChildStdin>,
    /// Pending ack waiters, keyed by request ID.
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<AckResponse>>>>,
    /// Whether the stdout reader task is running.
    reader_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AckResponse {
    success: bool,
    error: Option<String>,
}

// ── Stdout message types ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum BridgeMessage {
    #[serde(rename = "ack")]
    Ack {
        id: String,
        success: bool,
        error: Option<String>,
    },
    #[serde(rename = "streamEvent")]
    StreamEvent {
        #[serde(rename = "sessionId")]
        session_id: String,
        data: serde_json::Value,
    },
    #[serde(rename = "streamEnd")]
    StreamEnd {
        #[serde(rename = "sessionId")]
        session_id: String,
        success: bool,
        error: Option<String>,
    },
}

impl SdkBridge {
    /// Create a new bridge (does NOT spawn the child yet — lazy on first use).
    pub fn new() -> SdkBridgeHandle {
        let (event_tx, _) = broadcast::channel(256);
        Arc::new(Self {
            inner: Mutex::new(BridgeInner {
                child: None,
                stdin: None,
                pending: Arc::new(Mutex::new(HashMap::new())),
                reader_running: false,
            }),
            event_tx,
        })
    }

    /// Get a receiver for stream events (JSON strings).
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.event_tx.subscribe()
    }

    /// Ensure the bridge process is running. Spawns it if needed.
    async fn ensure_running(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;

        // Check if child is still alive
        if let Some(ref mut child) = inner.child {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Child exited — clean up
                    eprintln!("[bridge] Child process exited, will respawn");
                    inner.child = None;
                    inner.stdin = None;
                    inner.reader_running = false;
                }
                Ok(None) => return Ok(()), // Still running
                Err(e) => {
                    eprintln!("[bridge] Error checking child: {}", e);
                    inner.child = None;
                    inner.stdin = None;
                    inner.reader_running = false;
                }
            }
        }

        // Spawn the bridge process
        let bridge_script = find_bridge_script()?;
        let node_path = find_node()?;
        eprintln!("[bridge] Spawning: {} {}", node_path, bridge_script);

        let mut child = Command::new(&node_path)
            .arg(&bridge_script)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit()) // Bridge stderr → our stderr
            .spawn()
            .map_err(|e| format!("Failed to spawn bridge: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture bridge stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture bridge stdout".to_string())?;

        inner.child = Some(child);
        inner.stdin = Some(stdin);

        // Start stdout reader if not already running
        if !inner.reader_running {
            inner.reader_running = true;
            let pending = inner.pending.clone();
            let event_tx = self.event_tx.clone();

            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let msg: BridgeMessage = match serde_json::from_str(&line) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!(
                                "[bridge] Failed to parse: {} (line: {})",
                                e,
                                &line[..line.len().min(200)]
                            );
                            continue;
                        }
                    };

                    match msg {
                        BridgeMessage::Ack { id, success, error } => {
                            let mut map = pending.lock().await;
                            if let Some(tx) = map.remove(&id) {
                                let _ = tx.send(AckResponse { success, error });
                            }
                        }
                        BridgeMessage::StreamEvent {
                            session_id,
                            data,
                        } => {
                            let json = serde_json::json!({
                                "type": "streamEvent",
                                "sessionId": session_id,
                                "data": data,
                            });
                            let _ =
                                event_tx.send(serde_json::to_string(&json).unwrap_or_default());
                        }
                        BridgeMessage::StreamEnd {
                            session_id,
                            success,
                            error,
                        } => {
                            let json = serde_json::json!({
                                "type": "streamEnd",
                                "sessionId": session_id,
                                "success": success,
                                "error": error,
                            });
                            let _ =
                                event_tx.send(serde_json::to_string(&json).unwrap_or_default());
                        }
                    }
                }

                eprintln!("[bridge] Stdout reader exited");
            });
        }

        Ok(())
    }

    /// Send a command to the bridge and wait for the ack.
    async fn send_command(&self, cmd: serde_json::Value) -> Result<(), String> {
        self.ensure_running().await?;

        let id = cmd["id"]
            .as_str()
            .ok_or("Missing id in command")?
            .to_string();

        let (tx, rx) = oneshot::channel();

        {
            let inner = self.inner.lock().await;
            let mut pending = inner.pending.lock().await;
            pending.insert(id.clone(), tx);
        }

        // Write command to stdin
        {
            let mut inner = self.inner.lock().await;
            let stdin = inner
                .stdin
                .as_mut()
                .ok_or("Bridge stdin not available")?;
            let line = serde_json::to_string(&cmd).unwrap() + "\n";
            stdin
                .write_all(line.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to bridge: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("Failed to flush bridge stdin: {}", e))?;
        }

        // Wait for ack with timeout
        let ack = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| "Bridge command timed out (30s)".to_string())?
            .map_err(|_| "Bridge ack channel dropped".to_string())?;

        if ack.success {
            Ok(())
        } else {
            Err(ack.error.unwrap_or_else(|| "Unknown bridge error".to_string()))
        }
    }

    /// Resume (or start tracking) a Claude Code session in the bridge.
    pub async fn resume_session(&self, session_id: &str, cwd: &str) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.send_command(serde_json::json!({
            "cmd": "resume",
            "id": id,
            "sessionId": session_id,
            "cwd": cwd,
        }))
        .await
    }

    /// Send a message to a session and begin streaming the response.
    /// Stream events are broadcast via `self.subscribe()`.
    /// Pass `cwd` so the bridge can auto-register the session on first send.
    pub async fn send_message(&self, session_id: &str, message: &str, cwd: &str) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.send_command(serde_json::json!({
            "cmd": "send",
            "id": id,
            "sessionId": session_id,
            "message": message,
            "cwd": cwd,
        }))
        .await
    }

    /// Close a session in the bridge.
    pub async fn close_session(&self, session_id: &str) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.send_command(serde_json::json!({
            "cmd": "close",
            "id": id,
            "sessionId": session_id,
        }))
        .await
    }
}

/// Find the node binary — resolves via `which` for full path.
fn find_node() -> Result<String, String> {
    if let Ok(output) = std::process::Command::new("which").arg("node").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
    }
    // Fallback: common locations
    for path in &[
        "/usr/bin/node",
        "/usr/local/bin/node",
        "/opt/homebrew/bin/node",
    ] {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    // Last resort
    Ok("node".to_string())
}

/// Find the bridge script — checks dev path first, then bundled resource.
fn find_bridge_script() -> Result<String, String> {
    // Dev mode: source file in bridge/ directory
    let dev_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("bridge/bridge.mjs");
    if dev_path.exists() {
        return Ok(dev_path.to_string_lossy().to_string());
    }

    // Production: bundled resource next to the binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Linux/Windows: resources are next to binary
            let resource = dir.join("resources/bridge.cjs");
            if resource.exists() {
                return Ok(resource.to_string_lossy().to_string());
            }
            // macOS: resources are in ../Resources/
            let mac_resource = dir.join("../Resources/bridge.cjs");
            if mac_resource.exists() {
                return Ok(mac_resource.to_string_lossy().to_string());
            }
        }
    }

    Err("Could not find bridge script (bridge.mjs or bridge.cjs)".to_string())
}
