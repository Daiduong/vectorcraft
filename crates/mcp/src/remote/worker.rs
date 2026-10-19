use super::config::Settings;
use crate::{Backend, Remote, Server};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{
    Arc,
    mpsc::{SyncSender, sync_channel},
};
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

struct Job {
    message: Value,
    version: String,
    deadline: Instant,
    reply: tokio::sync::oneshot::Sender<Result<Value, String>>,
}
pub struct Worker {
    sender: SyncSender<Job>,
    pub pending: Arc<AtomicUsize>,
    pub uncertain: Arc<AtomicBool>,
}
struct Guarded {
    remote: Remote,
    uncertain: Arc<AtomicBool>,
    roots: Arc<vectorcraft_engine::file_access::AutomationRoots>,
}
impl Backend for Guarded {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        // Reject explicit file targets before scheduling native background work, even if a clean
        // document would make Save a no-op. The desktop also confines engine commands and input.
        use vectorcraft_engine::file_access::Access;
        let access = match method {
            "app.open" => Some(Access::Read),
            "app.save" | "app.export" | "ui.screenshot" | "ui.render" => Some(Access::Write),
            _ => None,
        };
        if let (Some(access), Some(path)) = (access, params.get("path").and_then(Value::as_str)) {
            self.roots.check(path, access)?;
        }
        let result = self.remote.call(method, params);
        if result.as_ref().err().is_some_and(|e| e.contains("may have completed") || e.contains("may yet complete") || e.contains("still running")) {
            self.uncertain.store(true, Ordering::SeqCst);
        }
        result
    }
    fn has_ui(&self) -> bool {
        true
    }
    fn describe(&self) -> String {
        self.remote.describe()
    }
}
fn opener() -> Value {
    json!({"name":"open_vectorcraft","title":"Open VectorCraft","description":"Open the configured VectorCraft desktop workspace, or reuse it if already running. Does not create or modify a document.",
    "inputSchema":{"type":"object","properties":{},"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false}})
}
fn shared_folders_tool() -> Value {
    json!({"name":"list_shared_folders","title":"List shared artwork folders","description":"List the owner-approved artwork folders and their availability, without opening the editor. Use their paths with file.list and file.open through command_run. Private connection settings remain excluded.",
    "inputSchema":{"type":"object","properties":{},"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false}})
}
fn read_only(message: &Value) -> bool {
    if message.get("method").and_then(Value::as_str) != Some("tools/call") {
        return true;
    }
    let name = message.pointer("/params/name").and_then(Value::as_str).unwrap_or("");
    if name == "list_shared_folders" {
        return true;
    }
    crate::tool_definitions()
        .iter()
        .any(|t| t.get("name").and_then(Value::as_str) == Some(name) && t.pointer("/annotations/readOnlyHint").and_then(Value::as_bool) == Some(true))
}
fn open(settings: &Settings) -> Result<Value, String> {
    let address = format!("127.0.0.1:{}", settings.control_port);
    if let Ok(mut remote) = Remote::connect(&address) {
        remote.call("ui.inspect", json!({}))?;
        return Ok(json!({"status":"already_running","ready":true}));
    }
    let mut command = std::process::Command::new(&settings.app_path);
    command
        .args(["--profile-dir"])
        .arg(&settings.profile_dir)
        .args(["--control", &settings.control_port.to_string()])
        .env("VECTORCRAFT_CONFIG_DIR", &settings.profile_dir)
        .env_remove("VECTORCRAFT_AUTOMATION_READ_ROOT")
        .env_remove("VECTORCRAFT_AUTOMATION_WRITE_ROOT")
        .env_remove("VECTORCRAFT_NO_PREFS")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(0x08000000);
    }
    let child = command.spawn().map_err(|e| format!("Cannot open the configured editor: {e}"))?;
    // Reap only the process we launched; a graphics-fallback child can take over its port.
    std::thread::spawn(move || {
        let mut child = child;
        if let Err(e) = child.wait() {
            log::debug!("Editor wait failed: {e}");
        }
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Ok(mut remote) = Remote::connect(&address) {
            remote.call("ui.inspect", json!({}))?;
            return Ok(json!({"status":"started","ready":true}));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err("The editor was launched but is not ready. Check its startup status before trying again".into())
}
fn dispatch(server: &mut Server, settings: &Settings, job: &Job) -> Value {
    let method = job.message.get("method").and_then(Value::as_str).unwrap_or("");
    if method != "initialize" {
        let _ = server.handle(json!({"jsonrpc":"2.0","id":"http-version","method":"initialize","params":{"protocolVersion":job.version}}));
    }
    let tool_name = job.message.pointer("/params/name").and_then(Value::as_str).unwrap_or("");
    let mut response = if method == "tools/call" && matches!(tool_name, "open_vectorcraft" | "list_shared_folders") {
        let args = job.message.pointer("/params/arguments");
        let result = if args.is_some_and(|a| !a.is_object() || a.as_object().is_some_and(|o| !o.is_empty())) {
            Err(format!("{tool_name} accepts no arguments"))
        } else if tool_name == "list_shared_folders" {
            Ok(
                json!({"folders":settings.artwork_roots().into_iter().map(|p| json!({"available":p.is_dir(),"path":p})).collect::<Vec<_>>(),"privateConnectionSettingsExcluded":true}),
            )
        } else {
            open(settings)
        };
        let tool = match result {
            Ok(result) => json!({"content":[{"type":"text","text":result.to_string()}],"structuredContent":result,"isError":false}),
            Err(error) => crate::ToolResult::error(error).to_value(),
        };
        json!({"jsonrpc":"2.0","id":job.message.get("id"),"result":tool})
    } else {
        server.handle(job.message.clone()).unwrap_or(Value::Null)
    };
    if method == "initialize" {
        if let Some(capabilities) = response.pointer_mut("/result/capabilities").and_then(Value::as_object_mut) {
            capabilities.remove("logging");
        }
        if let Some(instructions) = response.pointer_mut("/result/instructions") {
            *instructions = json!(format!(
                "{} This connection controls the shared live desktop. Use list_shared_folders to discover approved folders and open_vectorcraft if the editor is closed. Inspect before repeating a call whose reply was lost. Connection settings are local-only.",
                instructions.as_str().unwrap_or("")
            ));
        }
    }
    if method == "tools/list"
        && let Some(tools) = response.pointer_mut("/result/tools").and_then(Value::as_array_mut)
    {
        tools.insert(0, opener());
        tools.insert(1, shared_folders_tool());
        for tool in tools {
            tool["securitySchemes"] = json!([{"type":"oauth2","scopes":[super::SCOPE]}]);
            tool["_meta"]["securitySchemes"] = tool["securitySchemes"].clone();
        }
    }
    response
}
impl Worker {
    pub fn start(settings: Settings) -> Result<Arc<Self>, String> {
        let roots = settings.automation_roots()?.ok_or("Artwork access was not configured")?;
        let (sender, receiver) = sync_channel::<Job>(8);
        let pending = Arc::new(AtomicUsize::new(0));
        let count = pending.clone();
        let uncertain = Arc::new(AtomicBool::new(false));
        let guard = uncertain.clone();
        std::thread::Builder::new()
            .name("native-mcp-worker".into())
            .spawn(move || {
                let backend = Guarded { remote: Remote::deferred(&format!("127.0.0.1:{}", settings.control_port)), uncertain: guard.clone(), roots };
                let mut server = Server::new(Box::new(backend));
                for job in receiver {
                    let mutable = !read_only(&job.message);
                    let result = if job.deadline <= Instant::now() {
                        Err("Request expired before it started; nothing was sent".into())
                    } else if guard.load(Ordering::SeqCst) && mutable {
                        Err("A previous edit has an unknown outcome. Inspect the artwork, then resume edits from the local connection settings"
                            .into())
                    } else {
                        vectorcraft_engine::guard::catch_panic(|| dispatch(&mut server, &settings, &job)).map_err(|e| {
                            if mutable {
                                guard.store(true, Ordering::SeqCst);
                            }
                            format!("Native MCP handler failed: {e}")
                        })
                    };
                    count.fetch_sub(1, Ordering::SeqCst);
                    // A lost HTTP reply must never trigger a replay. Require local review before
                    // another mutation when the waiter vanished after dispatch began.
                    if job.reply.send(result).is_err() && mutable {
                        guard.store(true, Ordering::SeqCst);
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Arc::new(Self { sender, pending, uncertain }))
    }
    pub async fn call(&self, message: Value, version: String) -> Result<Value, String> {
        let mutable = !read_only(&message);
        let (reply, response) = tokio::sync::oneshot::channel();
        self.pending.fetch_add(1, Ordering::SeqCst);
        let job = Job { message, version, deadline: Instant::now() + Duration::from_secs(120), reply };
        if self.sender.try_send(job).is_err() {
            self.pending.fetch_sub(1, Ordering::SeqCst);
            return Err("Native MCP queue is full or stopped; nothing was sent".into());
        }
        match tokio::time::timeout(Duration::from_secs(125), response).await {
            Ok(Ok(value)) => value,
            _ => {
                if mutable {
                    self.uncertain.store(true, Ordering::SeqCst);
                }
                Err("Native reply was not received; inspect the artwork before repeating the operation".into())
            }
        }
    }
}
