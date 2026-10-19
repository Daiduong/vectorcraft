//! Owner-only connection wizard. Secrets never enter UiState, document data or control replies.
use crate::VectorcraftApp;
use serde_json::{Value, json};

pub type RemoteAccessFn = Box<dyn FnMut(Value) -> std::sync::mpsc::Receiver<Result<Value, String>>>;
pub type RemoteControlFn =
    Box<dyn FnMut(u16, std::sync::Arc<vectorcraft_engine::file_access::AutomationRoots>) -> std::sync::mpsc::Receiver<crate::ControlRequest>>;
#[derive(Default)]
pub struct RemoteAccessUi {
    pub open: bool,
    step: usize,
    pub_origin: String,
    lan_access: bool,
    trusted_proxy: String,
    artworks: Vec<String>,
    phrase: String,
    phrase_confirmation: String,
    show_phrase: bool,
    privacy_epoch: u64,
    callback: String,
    client_name: String,
    client_id: String,
    client_secret: String,
    status: Value,
    draft_loaded: bool,
    error: String,
    public_ready: bool,
    pending: Option<(String, std::sync::mpsc::Receiver<Result<Value, String>>)>,
}
impl RemoteAccessUi {
    pub fn pauses_control(&self) -> bool {
        self.open || self.pending.as_ref().is_some_and(|(operation, _)| matches!(operation.as_str(), "configure" | "resync"))
    }
    pub fn privacy_epoch(&self) -> u64 {
        self.privacy_epoch
    }
    pub fn allows_screenshot(&self, epoch: u64) -> bool {
        self.privacy_epoch == epoch && !(self.open && self.show_phrase)
    }
    fn validate_phrase(&self) -> Result<(), String> {
        let keeping_existing =
            self.status.get("configured").and_then(Value::as_bool) == Some(true) && self.phrase.is_empty() && self.phrase_confirmation.is_empty();
        if keeping_existing {
            return Ok(());
        }
        if self.phrase.len() > 1024 || !(12..=256).contains(&self.phrase.chars().count()) {
            return Err("Choose a passphrase of 12 to 256 characters and enter it twice".into());
        }
        if self.phrase != self.phrase_confirmation {
            return Err("Passphrases do not match. Check both fields before saving".into());
        }
        Ok(())
    }
    fn has_unsaved_changes(&self) -> bool {
        let saved_origin = self.status.get("publicOrigin").and_then(Value::as_str).unwrap_or("");
        let saved_proxy = self.status.get("trustedProxy").and_then(Value::as_str).unwrap_or("");
        let saved_lan = !saved_proxy.is_empty();
        let saved_folders: Vec<_> =
            self.status.get("artworkRoots").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
        self.pub_origin.trim().trim_end_matches('/') != saved_origin.trim_end_matches('/')
            || self.artworks.iter().map(|p| p.trim()).collect::<Vec<_>>() != saved_folders
            || !self.phrase.is_empty()
            || !self.phrase_confirmation.is_empty()
            || self.lan_access != saved_lan
            || (self.lan_access && self.trusted_proxy.trim() != saved_proxy)
    }
    fn folder_active(&self, folder: &str) -> bool {
        self.status.get("running").and_then(Value::as_bool) == Some(true)
            && self
                .status
                .get("activeArtworkRoots")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .any(|active| same_folder(active, folder))
    }
    fn needs_folder_sync(&self) -> bool {
        let active: Vec<_> =
            self.status.get("activeArtworkRoots").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
        self.status.get("running").and_then(Value::as_bool) != Some(true)
            || active.len() != self.artworks.len()
            || self.artworks.iter().any(|folder| !active.iter().any(|saved| same_folder(saved, folder)))
    }
    fn can_resync(&self) -> bool {
        self.status.get("configured").and_then(Value::as_bool) == Some(true) && !self.artworks.is_empty() && self.needs_folder_sync()
    }
    fn close(&mut self) {
        self.open = false;
        self.phrase.clear();
        self.phrase_confirmation.clear();
        self.show_phrase = false;
        self.client_secret.clear();
        self.privacy_epoch = self.privacy_epoch.wrapping_add(1);
    }
}
fn same_folder(first: &str, second: &str) -> bool {
    if first.trim().is_empty() || second.trim().is_empty() {
        return false;
    }
    let normalize = |value: &str| {
        let value = if cfg!(windows) { value.trim().replace('\\', "/") } else { value.trim().to_owned() };
        value.trim_end_matches('/').to_owned()
    };
    let first = normalize(first);
    let second = normalize(second);
    if cfg!(windows) { first.eq_ignore_ascii_case(&second) } else { first == second }
}
fn resync_button(ui: &mut egui::Ui, enabled: bool) -> egui::Response {
    let response = crate::widgets::primary_button_enabled(ui, "     Resync folders", enabled);
    let color = if enabled { egui::Color32::WHITE } else { crate::theme::Tokens::get(ui.ctx()).text_disabled };
    let center = egui::pos2(response.rect.left() + 17.0, response.rect.center().y);
    // Original, code-drawn paired arrows: no additional asset or icon dependency.
    for offset in [0.0, std::f32::consts::PI] {
        let points: Vec<_> = (0..16)
            .map(|index| {
                let angle = offset + index as f32 / 15.0 * 2.3;
                center + egui::vec2(angle.cos(), angle.sin()) * 6.0
            })
            .collect();
        if let Some(tip) = points.last().copied() {
            let angle = offset + 2.3;
            let tangent = egui::vec2(-angle.sin(), angle.cos());
            let normal = egui::vec2(angle.cos(), angle.sin());
            ui.painter().line_segment([tip - tangent * 3.0 + normal * 2.5, tip], egui::Stroke::new(1.5, color));
            ui.painter().line_segment([tip - tangent * 3.0 - normal * 2.5, tip], egui::Stroke::new(1.5, color));
        }
        ui.painter().add(egui::Shape::line(points, egui::Stroke::new(1.5, color)));
    }
    response.on_hover_text("Apply folder changes to the connection. Existing linked accounts and the passphrase are kept.")
}
pub fn open(app: &mut VectorcraftApp) -> Result<Value, String> {
    app.refuse_automation("ui.remoteAccess")?;
    if app.services.remote_access.is_none() {
        return Err("Remote connections are unavailable in this build".into());
    }
    app.remote_access.open = true;
    app.remote_access.draft_loaded = false;
    app.remote_access.show_phrase = false;
    app.remote_access.privacy_epoch = app.remote_access.privacy_epoch.wrapping_add(1);
    app.ui.dialog = None;
    action(app, &json!({"action":"status"}))?;
    Ok(Value::Null)
}
pub fn action(app: &mut VectorcraftApp, parameters: &Value) -> Result<Value, String> {
    app.refuse_automation("ui.remoteAccess.action")?;
    let operation = parameters.get("action").and_then(Value::as_str).unwrap_or("");
    if app.remote_access.pending.is_some() {
        return Err("Wait for the current connection operation to finish".into());
    }
    match operation {
        "toggle_phrase_visibility" => {
            app.remote_access.show_phrase = !app.remote_access.show_phrase;
            app.remote_access.privacy_epoch = app.remote_access.privacy_epoch.wrapping_add(1);
            return Ok(Value::Null);
        }
        "done" => {
            if app.remote_access.status.get("configured").and_then(Value::as_bool) != Some(true) || app.remote_access.has_unsaved_changes() {
                return Err("Save your setup on the Authorize step before finishing".into());
            }
            app.remote_access.close();
            return Ok(Value::Null);
        }
        "choose_folder" => {
            if app.remote_access.artworks.len() >= 32 {
                return Err("You can share up to 32 artwork folders".into());
            }
            if let Some(folder) = crate::picks::folder(app)
                && !app
                    .remote_access
                    .artworks
                    .iter()
                    .any(|existing| if cfg!(windows) { existing.eq_ignore_ascii_case(&folder) } else { existing == &folder })
            {
                app.remote_access.artworks.push(folder);
            }
            return Ok(Value::Null);
        }
        "add_path" => {
            if app.remote_access.artworks.len() >= 32 {
                return Err("You can share up to 32 artwork folders".into());
            }
            app.remote_access.artworks.push(String::new());
            return Ok(Value::Null);
        }
        "remove_folder" => {
            let index = parameters.get("index").and_then(Value::as_u64).and_then(|n| usize::try_from(n).ok()).ok_or("Choose a folder to remove")?;
            if app.remote_access.artworks.get(index).is_none() {
                return Err("The folder is no longer in the list".into());
            }
            app.remote_access.artworks.remove(index);
            return Ok(Value::Null);
        }
        "copy_url" => {
            app.clipboard_out = app.remote_access.status.get("endpoint").and_then(Value::as_str).map(str::to_owned);
            return Ok(Value::Null);
        }
        "copy_id" => {
            app.clipboard_out = Some(app.remote_access.client_id.clone());
            return Ok(Value::Null);
        }
        "copy_secret" => {
            app.clipboard_out = Some(app.remote_access.client_secret.clone());
            return Ok(Value::Null);
        }
        _ => {}
    }
    if operation == "start" {
        let status = app.remote_access.status.clone();
        install_authority(app, &status)?;
    }
    let state = &mut app.remote_access;
    let payload = match operation {
        "configure" => {
            state.validate_phrase()?;
            state.phrase_confirmation.clear();
            state.show_phrase = false;
            state.privacy_epoch = state.privacy_epoch.wrapping_add(1);
            json!({"action":operation,"publicOrigin":state.pub_origin.trim(),"lanAccess":state.lan_access,"trustedProxy":state.trusted_proxy.trim(),"artworkRoots":state.artworks.iter().map(|p| p.trim()).collect::<Vec<_>>(),"passphrase":std::mem::take(&mut state.phrase)})
        }
        "resync" => {
            if !state.can_resync() {
                return Err("Choose folders in an existing setup before resyncing".into());
            }
            json!({"action":"resync","artworkRoots":state.artworks.iter().map(|p| p.trim()).collect::<Vec<_>>()})
        }
        "register" => json!({"action":operation,"name":state.client_name.trim(),"callback":state.callback.trim()}),
        "revoke_client" => json!({"action":operation,"id":parameters.get("id").and_then(Value::as_str).unwrap_or("")}),
        "status" | "start" | "stop" | "check_public" | "revoke_all" | "resume" => json!({"action":operation}),
        _ => return Err("Unknown local connection action".into()),
    };
    let service = app.services.remote_access.as_mut().ok_or("Remote connections are unavailable in this build")?;
    state.error.clear();
    state.pending = Some((operation.into(), service(payload)));
    Ok(Value::Null)
}
fn install_authority(app: &mut VectorcraftApp, value: &Value) -> Result<(), String> {
    if let Some(error) = value.get("configurationError").and_then(Value::as_str) {
        return Err(error.into());
    }
    let folders: Vec<std::path::PathBuf> =
        serde_json::from_value(value.get("artworkRoots").cloned().unwrap_or(Value::Null)).map_err(|_| "The saved folder grants are unavailable")?;
    let private = value.get("privateFolder").and_then(Value::as_str).ok_or("The private settings folder is unavailable")?;
    let roots = vectorcraft_engine::file_access::AutomationRoots::new_many(&folders, &folders, &[private.into()])?
        .ok_or("Choose artwork folders before starting the connection")?;
    app.install_automation_roots(Some(roots.clone()));
    if app.control_rx.is_none() {
        let port = value
            .get("controlPort")
            .and_then(Value::as_u64)
            .and_then(|n| u16::try_from(n).ok())
            .filter(|p| *p != 0)
            .ok_or("The saved control port is unavailable")?;
        let start = app.services.start_remote_control.as_mut().ok_or("The local control channel cannot start in this build")?;
        app.control_rx = Some(start(port, roots));
    }
    Ok(())
}
fn poll(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let mut start_after_configure = false;
    let result = app.remote_access.pending.as_ref().and_then(|(name, receiver)| match receiver.try_recv() {
        Ok(result) => Some((name.clone(), result)),
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Some((name.clone(), Err("The local connection task ended unexpectedly".into()))),
        Err(_) => None,
    });
    if let Some((operation, result)) = result {
        app.remote_access.pending = None;
        match result {
            Ok(mut value) => {
                if operation == "register" {
                    app.remote_access.client_id = value.get("client_id").and_then(Value::as_str).unwrap_or("").into();
                    app.remote_access.client_secret = value.get("client_secret").and_then(Value::as_str).unwrap_or("").into();
                    if let Some(object) = value.as_object_mut() {
                        object.remove("client_secret");
                    }
                    app.remote_access.error = "Client created. Copy its ID and secret into your AI app, then complete sign-in.".into();
                } else {
                    if let Some(error) = value.get("configurationError").and_then(Value::as_str) {
                        app.remote_access.error = error.into();
                    }
                    if matches!(operation.as_str(), "configure" | "resync") {
                        match install_authority(app, &value) {
                            Ok(()) => start_after_configure = true,
                            Err(e) => app.remote_access.error = e,
                        }
                        app.remote_access.public_ready = false;
                        if operation == "configure" {
                            app.remote_access.step = 3;
                        }
                    }
                    if !app.remote_access.draft_loaded || operation == "configure" {
                        if let Some(origin) = value.get("publicOrigin").and_then(Value::as_str) {
                            app.remote_access.pub_origin = origin.into();
                        }
                        if let Some(proxy) = value.get("trustedProxy").and_then(Value::as_str) {
                            app.remote_access.trusted_proxy = proxy.into();
                            app.remote_access.lan_access = !proxy.is_empty();
                        }
                        if let Some(folders) = value.get("artworkRoots").and_then(Value::as_array) {
                            app.remote_access.artworks = folders.iter().filter_map(Value::as_str).map(str::to_owned).collect();
                        }
                        app.remote_access.draft_loaded = true;
                    }
                    if value.get("publicReady").and_then(Value::as_bool) == Some(true) {
                        app.remote_access.public_ready = true;
                    }
                    app.remote_access.status = value;
                }
            }
            Err(error) => app.remote_access.error = error,
        }
    }
    if start_after_configure && let Some(service) = app.services.remote_access.as_mut() {
        // Continue the owner's Save-and-start command only after its new authority is live.
        app.remote_access.pending = Some(("start".into(), service(json!({"action":"start"}))));
    }
    if app.remote_access.pending.is_some() {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
pub fn show(app: &mut VectorcraftApp, ctx: &egui::Context) {
    poll(app, ctx);
    if !app.remote_access.open {
        return;
    }
    let owner_input = !app.synthetic_frame && !app.ui.answering_control;
    let mut opened = true;
    let mut request: Option<Value> = None;
    let state = &mut app.remote_access;
    let running = state.status.get("running").and_then(Value::as_bool) == Some(true);
    let configured = state.status.get("configured").and_then(Value::as_bool) == Some(true);
    let busy = state.pending.is_some();
    egui::Window::new("Remote AI connections")
        .id(egui::Id::new("remote-ai-connections"))
        .open(&mut opened)
        .collapsible(false)
        .resizable(false)
        .default_width(620.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.add_enabled_ui(owner_input && !busy, |ui| {
                ui.label("Connect your AI Apps to Vectorcraft");
                ui.small("Connection settings can only be changed in this menu");
                ui.horizontal_wrapped(|ui| {
                    ui.small("Adding/removing folders after setup requires resync");
                    if resync_button(ui, state.can_resync()).clicked() { request = Some(json!({"action":"resync"})); }
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    for (index, label) in ["1  Artwork", "2  Gateway", "3  Authorize", "4  Connections"].iter().enumerate() {
                        if ui.selectable_label(state.step == index, *label).clicked() {
                            state.step = index;
                        }
                    }
                });
                ui.separator();
                ui.add_space(8.0);
                match state.step {
                    0 => {
                        ui.heading("Choose artwork folders");
                        ui.label("AI apps can open and save files in these folders and their subfolders. Private connection settings are always excluded.");
                        ui.add_space(8.0);
                        let active: Vec<_> = state.artworks.iter().map(|folder| state.folder_active(folder)).collect();
                        egui::ScrollArea::vertical().id_salt("remote-artwork-folders").max_height(200.0).show(ui, |ui| {
                            for (index, folder) in state.artworks.iter_mut().enumerate() {
                                let label = std::path::Path::new(folder).file_name().and_then(|s| s.to_str()).filter(|s| !s.is_empty()).unwrap_or(folder);
                                ui.label(egui::RichText::new(label).strong());
                                ui.horizontal(|ui| {
                                    let synced = active.get(index).copied().unwrap_or(false);
                                    let t = crate::theme::Tokens::get(ui.ctx());
                                    ui.colored_label(if synced { t.axis_y } else { t.error }, if synced { "✓" } else { "×" })
                                        .on_hover_text(if synced { "Active in the running connection" } else { "Not active in the running connection; resync to apply" });
                                    let full_path = folder.clone();
                                    ui.add(egui::TextEdit::singleline(folder).id_salt(("artwork", index)).hint_text("Folder path").desired_width(480.0)).on_hover_text(full_path);
                                    if ui.button("Remove").clicked() {
                                        request = Some(json!({"action":"remove_folder","index":index}));
                                    }
                                });
                            }
                        });
                        ui.small("✓ Active in this connection     × Pending resync or connection stopped");
                        if configured && state.needs_folder_sync() { ui.colored_label(crate::theme::Tokens::get(ui.ctx()).warning, "Folder changes are pending. Resync folders to apply additions and removals."); }
                        ui.horizontal(|ui| {
                            if ui.add_enabled(state.artworks.len() < 32, egui::Button::new("Add folder…")).clicked() {
                                request = Some(json!({"action":"choose_folder"}));
                            }
                            if ui.add_enabled(state.artworks.len() < 32, egui::Button::new("Add path")).clicked() {
                                request = Some(json!({"action":"add_path"}));
                            }
                            ui.small(format!("{} / 32 folders", state.artworks.len()));
                        });
                        ui.add_space(12.0);
                        if ui.add_enabled(!state.artworks.is_empty(), egui::Button::new("Next: gateway address")).clicked() {
                            state.step = 1;
                        }
                    }
                    1 => {
                        ui.heading("Enter your gateway address");
                        ui.label("Use the public HTTPS address provided by your tunnel or reverse proxy.");
                        ui.add(egui::TextEdit::singleline(&mut state.pub_origin).hint_text("https://your-gateway.example").desired_width(590.0));
                        ui.small("Enter only the address, without a path. This connection uses /vectorcraft-native/mcp.");
                        ui.add_space(8.0);
                        ui.label("Your gateway must forward /vectorcraft-native/ and its OAuth discovery paths to this computer on port 3002.");
                        ui.small("Your existing editor connections can use their own routes alongside this one.");
                        ui.add_space(8.0);
                        ui.checkbox(&mut state.lan_access, "My gateway runs on another computer (allow LAN access)");
                        if state.lan_access {
                            ui.add(egui::TextEdit::singleline(&mut state.trusted_proxy).hint_text("Trusted gateway IP, e.g. 192.168.1.10").desired_width(400.0));
                            ui.colored_label(crate::theme::Tokens::get(ui.ctx()).warning, "The gateway-to-desktop HTTP hop is unencrypted. Use only a trusted private network or encrypted tunnel.");
                            ui.small("Only this gateway IP and this computer are accepted. Restrict your firewall to the same gateway.");
                        } else {
                            ui.small("Default: this computer only (127.0.0.1:3002). Run the HTTPS gateway locally or use a tunnel to localhost.");
                        }
                        if ui.button("Next: owner authorization").clicked() {
                            state.step = 2;
                        }
                    }
                    2 => {
                        ui.heading("Choose an owner passphrase");
                        ui.label("Enter this passphrase on the sign-in page when linking an AI app. It is stored as a password hash.");
                        ui.label("Passphrase");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.phrase).password(!state.show_phrase)
                                .hint_text(if configured { "Leave both fields blank to keep the existing passphrase" } else { "At least 12 characters" })
                                .desired_width(550.0));
                            let tip = if state.show_phrase { "Hide passphrase" } else { "Show passphrase" };
                            let eye = crate::widgets::icon_button(ui, "eye", tip, state.show_phrase, 24.0);
                            if !state.show_phrase {
                                let rect = eye.rect.shrink(5.0);
                                ui.painter().line_segment([rect.left_bottom(), rect.right_top()], egui::Stroke::new(1.5, crate::theme::Tokens::get(ui.ctx()).text));
                            }
                            if eye.clicked() { request = Some(json!({"action":"toggle_phrase_visibility"})); }
                        });
                        ui.label("Confirm passphrase");
                        ui.add(egui::TextEdit::singleline(&mut state.phrase_confirmation).password(!state.show_phrase)
                            .hint_text("Enter the same passphrase again").desired_width(550.0));
                        if !state.phrase_confirmation.is_empty() {
                            ui.small(if state.phrase == state.phrase_confirmation { "Passphrases match." } else { "Passphrases do not match yet." });
                        }
                        ui.add_space(8.0);
                        ui.label("All authorized accounts share the same open document and selection.");
                        if running {
                            ui.small("Stop the current connection on the connections tab before changing this passphrase");
                        }
                        if ui.add_enabled(!running && state.validate_phrase().is_ok(), egui::Button::new("Save setup and start connection")).clicked() {
                            request = Some(json!({"action":"configure"}));
                        }
                    }
                    _ => {
                        ui.heading(if running { "Connection is running" } else { "Connection is stopped" });
                        if let Some(endpoint) = state.status.get("endpoint").and_then(Value::as_str) {
                            ui.label(egui::RichText::new(endpoint).monospace());
                        }
                        ui.horizontal(|ui| {
                            if ui.add_enabled(configured, egui::Button::new("Copy connection URL")).clicked() {
                                request = Some(json!({"action":"copy_url"}));
                            }
                            if ui.button("Refresh status").clicked() {
                                request = Some(json!({"action":"status"}));
                            }
                            if ui.add_enabled(configured, egui::Button::new(if running { "Stop connection" } else { "Start connection" })).clicked() {
                                request = Some(json!({"action":if running{"stop"}else{"start"}}));
                            }
                        });
                        if state.public_ready {
                            ui.label("✓ Public gateway reached this native service.");
                        } else if ui.add_enabled(running, egui::Button::new("Check public gateway")).clicked() {
                            request = Some(json!({"action":"check_public"}));
                        }
                    ui.small("Add the URL to your AI app with OAuth, then authorize it using your owner passphrase. Close this window before asking the app to edit artwork.");
                        ui.separator();
                        ui.label(egui::RichText::new("Linked apps").strong());
                        let clients = state.status.get("clients").and_then(Value::as_array).cloned().unwrap_or_default();
                        if clients.is_empty() {
                            ui.small("No apps have registered yet.");
                        }
                        egui::ScrollArea::vertical().id_salt("remote-clients").max_height(110.0).show(ui, |ui| {
                            for client in clients {
                                ui.horizontal(|ui| {
                                    ui.label(client.get("name").and_then(Value::as_str).unwrap_or("AI app"));
                                    ui.small(if client.get("connections").and_then(Value::as_u64).unwrap_or(0) > 0 {
                                        "Linked"
                                    } else {
                                        "Waiting for sign-in"
                                    });
                                    if ui.add_enabled(running, egui::Button::new("Revoke")).clicked() {
                                        request = Some(json!({"action":"revoke_client","id":client.get("id")}));
                                    }
                                });
                            }
                        });
                        if ui.add_enabled(running, egui::Button::new("Revoke all app connections")).clicked() {
                            request = Some(json!({"action":"revoke_all"}));
                        }
                        if state.status.get("uncertain").and_then(Value::as_bool) == Some(true) {
                            ui.colored_label(egui::Color32::YELLOW, "An edit's outcome is unknown. Inspect the document before resuming.");
                            if ui.button("I reviewed the artwork — resume edits").clicked() {
                                request = Some(json!({"action":"resume"}));
                            }
                        }
                        ui.collapsing("App asks for a client ID and secret?", |ui| {
                            ui.small("Paste the exact redirect URI from the AI app. Create a separate client for each account.");
                            ui.add(egui::TextEdit::singleline(&mut state.client_name).hint_text("Connection name").desired_width(560.0));
                            ui.add(egui::TextEdit::singleline(&mut state.callback).hint_text("https://… redirect URI").desired_width(560.0));
                            if ui.add_enabled(running, egui::Button::new("Create client credentials")).clicked() {
                                request = Some(json!({"action":"register"}));
                            }
                            if !state.client_id.is_empty() {
                                ui.label(format!("Client ID: {}", state.client_id));
                                ui.label("Client secret: ••••••••••••••••••••");
                                ui.horizontal(|ui| {
                                    if ui.button("Copy client ID").clicked() {
                                        request = Some(json!({"action":"copy_id"}));
                                    }
                                    if ui.button("Copy client secret").clicked() {
                                        request = Some(json!({"action":"copy_secret"}));
                                    }
                                });
                            }
                        });
                    }
                }
                if !state.error.is_empty() {
                    ui.separator();
                    ui.label(&state.error);
                }
            });
            if busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Updating connection…");
                });
            }
            if state.step == 3 {
                ui.add_space(12.0);
                ui.separator();
                let unsaved = state.has_unsaved_changes();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add_enabled(owner_input && !busy && configured && !unsaved, egui::Button::new("Done")).clicked() {
                        request = Some(json!({"action":"done"}));
                    }
                    ui.small(if unsaved { "Resync folder changes, or save other setup changes on the Authorize step." }
                        else if running { "Setup saved. The connection keeps running when you finish." }
                        else if configured { "Setup saved. Done closes this window." } else { "Complete setup before finishing." });
                });
            }
        });
    if !opened {
        app.remote_access.close();
    }
    if let Some(request) = request
        && let Err(error) = app.run("ui.remoteAccess.action", request)
    {
        app.remote_access.error = error;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn folder_checks_describe_the_running_grant_not_just_saved_settings() {
        let mut state = RemoteAccessUi::default();
        state.artworks = vec!["C:/artwork".into(), "C:/new-artwork".into()];
        state.status = json!({"configured":true,"running":true,"artworkRoots":state.artworks,"activeArtworkRoots":["C:/artwork"]});
        assert!(state.folder_active("C:/artwork"));
        assert!(!state.folder_active("C:/new-artwork"));
        assert!(state.can_resync());
        state.status["activeArtworkRoots"] = json!(state.artworks);
        assert!(!state.needs_folder_sync());
        state.artworks.pop();
        assert!(state.needs_folder_sync(), "removals need resync too");
        state.status["running"] = json!(false);
        assert!(!state.folder_active("C:/artwork"));
    }
    #[test]
    fn status_refresh_preserves_unsaved_folder_and_passphrase_drafts() {
        let mut app = app();
        open(&mut app).unwrap();
        poll(&mut app, &egui::Context::default());
        app.remote_access.artworks.push("C:/new-artwork".into());
        app.remote_access.phrase = "a local draft passphrase".into();
        action(&mut app, &json!({"action":"status"})).unwrap();
        poll(&mut app, &egui::Context::default());
        assert_eq!(app.remote_access.artworks.len(), 2);
        assert_eq!(app.remote_access.phrase, "a local draft passphrase");
    }
    #[test]
    fn gateway_starts_local_only_and_lan_access_is_an_explicit_unsaved_owner_choice() {
        let mut app = app();
        open(&mut app).unwrap();
        poll(&mut app, &egui::Context::default());
        app.remote_access.step = 1;
        assert!(!app.remote_access.lan_access);
        let text = crate::tests_labels::painted_text(&mut app, |app, ui| show(app, ui.ctx()));
        assert!(text.contains("127.0.0.1:3002"));
        assert!(!text.contains("hop is unencrypted"));
        app.remote_access.lan_access = true;
        app.remote_access.trusted_proxy = "192.0.2.10".into();
        assert!(app.remote_access.has_unsaved_changes());
        let text = crate::tests_labels::painted_text(&mut app, |app, ui| show(app, ui.ctx()));
        assert!(text.contains("hop is unencrypted"));
        app.services.remote_access = Some(Box::new(|request| {
            assert_eq!(request["lanAccess"], true);
            assert_eq!(request["trustedProxy"], "192.0.2.10");
            std::sync::mpsc::channel().1
        }));
        action(&mut app, &json!({"action":"configure"})).unwrap();
    }
    #[test]
    fn starting_saved_setup_installs_authority_and_opens_control_before_http() {
        let root = std::env::temp_dir().join(format!("vc-remote-lazy-control-{}", std::process::id()));
        let artwork = root.join("artwork");
        let private = root.join("Remote Access");
        std::fs::create_dir_all(&artwork).unwrap();
        std::fs::create_dir_all(&private).unwrap();
        let mut app = app();
        app.services.read = Some(Box::new(|_| Ok(vec![])));
        app.services.write_shared = Some(std::sync::Arc::new(|_, _| Ok(())));
        app.remote_access.status = json!({"configured":true,"artworkRoots":[artwork],"privateFolder":private,"controlPort":33982});
        app.services.start_remote_control = Some(Box::new(move |port, roots| {
            assert_eq!(port, 33982);
            assert!(roots.check(&artwork.join("new.svg").to_string_lossy(), vectorcraft_engine::file_access::Access::Write).is_ok());
            assert!(roots.check(&private.join("settings.json").to_string_lossy(), vectorcraft_engine::file_access::Access::Read).is_err());
            std::sync::mpsc::channel().1
        }));
        assert!(app.control_rx.is_none());
        action(&mut app, &json!({"action":"start"})).unwrap();
        assert!(app.control_rx.is_some());
        assert!(app.automation_roots.is_some());
        let grants = app.automation_roots.clone().unwrap();
        let outside = root.join("outside.svg").to_string_lossy().into_owned();
        assert!(vectorcraft_engine::file_access::confine(Some(&grants), || app.services.read.as_ref().unwrap()(&outside)).is_err());
        assert!(vectorcraft_engine::file_access::confine(Some(&grants), || app.services.write_shared.as_ref().unwrap()(&outside, &[])).is_err());
        app.remote_access.pending = None;
        app.remote_access.status["configurationError"] = json!("private store needs repair");
        assert!(action(&mut app, &json!({"action":"start"})).is_err());
        assert!(app.remote_access.pending.is_none(), "a failed repair never starts HTTP");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn resync_sends_only_folders_and_waits_for_installed_authority_before_start() {
        let mut app = app();
        open(&mut app).unwrap();
        poll(&mut app, &egui::Context::default());
        let root = std::env::temp_dir()
            .join(format!("vectorcraft-resync-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let artwork = root.join("artwork");
        let private = root.join("Remote Access");
        std::fs::create_dir_all(&artwork).unwrap();
        std::fs::create_dir_all(&private).unwrap();
        app.remote_access.artworks = vec![artwork.to_string_lossy().into_owned()];
        app.remote_access.phrase = "unsaved passphrase stays local".into();
        app.remote_access.pub_origin = "https://unsaved-origin.example".into();
        let expected_artwork = artwork.clone();
        let private_response = private.clone();
        app.services.remote_access = Some(Box::new(move |request| {
            assert_eq!(request, json!({"action":"resync","artworkRoots":[expected_artwork]}));
            let (send, receive) = std::sync::mpsc::channel();
            send.send(Ok(json!({"configured":true,"running":false,"publicOrigin":"https://native.example.test","artworkRoots":[expected_artwork],"activeArtworkRoots":[],"privateFolder":private_response,"controlPort":33982}))).unwrap();
            receive
        }));
        action(&mut app, &json!({"action":"resync"})).unwrap();
        assert!(app.remote_access.pauses_control());
        let starts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = starts.clone();
        let expected_artwork = artwork.clone();
        app.services.remote_access = Some(Box::new(move |request| {
            assert_eq!(request, json!({"action":"start"}));
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let (send, receive) = std::sync::mpsc::channel();
            send.send(Ok(json!({"configured":true,"running":true,"activeArtworkRoots":[expected_artwork]}))).unwrap();
            receive
        }));
        poll(&mut app, &egui::Context::default());
        assert_eq!(starts.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(!app.remote_access.folder_active(&artwork.to_string_lossy()));
        let roots = app.automation_roots.as_ref().unwrap();
        assert!(roots.check(&artwork.join("new.svg").to_string_lossy(), vectorcraft_engine::file_access::Access::Write).is_ok());
        assert!(roots.check(&private.join("settings.json").to_string_lossy(), vectorcraft_engine::file_access::Access::Read).is_err());
        poll(&mut app, &egui::Context::default());
        assert!(app.remote_access.folder_active(&artwork.to_string_lossy()));
        assert_eq!(app.remote_access.phrase, "unsaved passphrase stays local");
        assert_eq!(app.remote_access.pub_origin, "https://unsaved-origin.example");
        std::fs::remove_dir_all(root).unwrap();
    }
    fn app() -> VectorcraftApp {
        let services = crate::Services {
            start_remote_control: Some(Box::new(|_, _| std::sync::mpsc::channel().1)),
            remote_access: Some(Box::new(|_| {
                let (sender, receiver) = std::sync::mpsc::channel();
                sender.send(Ok(json!({"configured":true,"running":false,"publicOrigin":"https://native.example.test","artworkRoots":["C:/artwork"],"activeArtworkRoots":["C:/artwork"],"endpoint":"https://native.example.test/vectorcraft-native/mcp","clients":[]}))).unwrap();
                receiver
            })),
            ..Default::default()
        };
        VectorcraftApp::new(vectorcraft_engine::Session::new(), services)
    }
    #[test]
    fn a_new_passphrase_requires_matching_confirmation_before_any_host_call() {
        let mut app = app();
        app.remote_access.phrase = "a sufficiently long passphrase".into();
        assert!(action(&mut app, &json!({"action":"configure"})).is_err());
        assert!(app.remote_access.pending.is_none());
        app.remote_access.phrase_confirmation = "another sufficiently long phrase".into();
        assert!(action(&mut app, &json!({"action":"configure"})).is_err());
        assert!(!app.remote_access.phrase.is_empty());
        app.remote_access.phrase_confirmation = app.remote_access.phrase.clone();
        action(&mut app, &json!({"action":"configure"})).unwrap();
        assert!(app.remote_access.pending.is_some());
        assert!(app.remote_access.phrase.is_empty());
        assert!(app.remote_access.phrase_confirmation.is_empty());
        assert!(!app.remote_access.show_phrase);
    }
    #[test]
    fn both_blank_fields_keep_an_existing_passphrase_but_not_a_new_one() {
        let mut state = RemoteAccessUi::default();
        assert!(state.validate_phrase().is_err());
        state.status = json!({"configured":true});
        assert!(state.validate_phrase().is_ok());
        state.phrase_confirmation = "confirmation alone is not valid".into();
        assert!(state.validate_phrase().is_err());
        state.phrase = "short".into();
        state.phrase_confirmation = state.phrase.clone();
        assert!(state.validate_phrase().is_err());
    }
    #[test]
    fn revealed_passphrases_are_local_only_and_in_flight_screenshots_are_discarded() {
        let mut app = app();
        open(&mut app).unwrap();
        let ctx = egui::Context::default();
        poll(&mut app, &ctx);
        app.remote_access.phrase = "private local reveal only".into();
        app.remote_access.phrase_confirmation = app.remote_access.phrase.clone();
        let (send, receive) = std::sync::mpsc::channel();
        app = app.with_control(receive);
        let (request, reply) = crate::control::ControlRequest::new("ui.screenshot", json!({"data":true}));
        send.send(request).unwrap();
        app.drain_control(&ctx);
        let token = app.pending_screenshots.first().unwrap().token;
        action(&mut app, &json!({"action":"toggle_phrase_visibility"})).unwrap();
        assert!(app.remote_access.show_phrase);
        let inspect = crate::control::inspect(&app, &ctx).to_string();
        assert!(!inspect.contains("private local reveal only"));
        let (request, _) = crate::control::ControlRequest::new("ui.screenshot", json!({"data":true}));
        match crate::control::handle(&mut app, &ctx, &request) {
            crate::control::Outcome::Done(value) => assert_eq!(value["ok"], false),
            _ => panic!("A visible passphrase must not be captured"),
        }
        action(&mut app, &json!({"action":"toggle_phrase_visibility"})).unwrap();
        let input = egui::RawInput {
            events: vec![egui::Event::Screenshot {
                viewport_id: egui::ViewportId::ROOT,
                user_data: egui::UserData::new(token),
                image: std::sync::Arc::new(egui::ColorImage::filled([1, 1], egui::Color32::WHITE)),
            }],
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| app.collect_screenshots(ui.ctx()));
        output.textures_delta.clear();
        let response = reply.try_recv().unwrap();
        assert_eq!(response["ok"], false);
        assert!(response["error"].as_str().unwrap().contains("visibility changed"));
        app.ui.answering_control = true;
        assert!(action(&mut app, &json!({"action":"toggle_phrase_visibility"})).is_err());
        assert!(!app.remote_access.show_phrase);
    }
    #[test]
    fn done_preserves_saved_setup_and_running_service_and_refuses_unsaved_edits() {
        let mut app = app();
        open(&mut app).unwrap();
        poll(&mut app, &egui::Context::default());
        app.remote_access.status["running"] = json!(true);
        app.remote_access.artworks.push("C:/not-saved".into());
        assert!(action(&mut app, &json!({"action":"done"})).is_err());
        app.remote_access.artworks.pop();
        app.remote_access.client_secret = "temporary-client-secret".into();
        app.remote_access.show_phrase = true;
        let saved_status = app.remote_access.status.clone();
        action(&mut app, &json!({"action":"done"})).unwrap();
        assert!(!app.remote_access.open);
        assert_eq!(app.remote_access.status, saved_status);
        assert!(app.remote_access.pending.is_none());
        assert!(app.remote_access.client_secret.is_empty());
        assert!(!app.remote_access.show_phrase);
    }
    #[test]
    fn connection_wizard_is_available_in_preferences_without_an_edit_menu_duplicate() {
        let app = app();
        assert!(!crate::menus::menu_entries(&app).iter().any(|entry| entry.command.as_deref() == Some("ui.remoteAccess")));
    }
    #[test]
    fn authorize_and_completion_steps_render_the_owner_controls() {
        let mut app = app();
        open(&mut app).unwrap();
        poll(&mut app, &egui::Context::default());
        app.remote_access.step = 2;
        app.remote_access.phrase = "example visual test phrase".into();
        app.remote_access.phrase_confirmation = app.remote_access.phrase.clone();
        let mut styled = false;
        let mut harness = egui_kittest::Harness::builder().with_size([900.0, 720.0]).build_ui_state(
            move |ui, app: &mut VectorcraftApp| {
                if !styled {
                    crate::theme::install_fonts(ui.ctx());
                    crate::theme::apply(ui.ctx(), app.ui.brightness);
                    egui_extras::install_image_loaders(ui.ctx());
                    ui.ctx().all_styles_mut(|style| style.animation_time = 0.0);
                    styled = true;
                    ui.ctx().request_repaint();
                    return;
                }
                show(app, ui.ctx());
            },
            app,
        );
        harness.run();
        harness.step();
        harness.step();
        let output = std::env::var_os("VECTORCRAFT_TEST_UI_OUTPUT").map(std::path::PathBuf::from);
        if let Some(folder) = &output {
            std::fs::create_dir_all(folder).unwrap();
            harness.render().unwrap().save(folder.join("authorize.png")).unwrap();
        }
        {
            let app = harness.state_mut();
            app.remote_access.step = 1;
            app.remote_access.lan_access = true;
            app.remote_access.trusted_proxy = "192.0.2.10".into();
        }
        harness.run();
        harness.step();
        harness.step();
        if let Some(folder) = &output {
            harness.render().unwrap().save(folder.join("gateway.png")).unwrap();
        }
        {
            let app = harness.state_mut();
            app.remote_access.lan_access = false;
            app.remote_access.trusted_proxy.clear();
            app.remote_access.step = 0;
            app.remote_access.status["running"] = json!(true);
            app.remote_access.artworks.push("C:/new-artwork".into());
        }
        harness.run();
        harness.step();
        harness.step();
        if let Some(folder) = &output {
            harness.render().unwrap().save(folder.join("folders.png")).unwrap();
        }
        let app = harness.state_mut();
        app.remote_access.artworks.pop();
        assert!(app.remote_access.validate_phrase().is_ok());
        app.remote_access.phrase.clear();
        app.remote_access.phrase_confirmation.clear();
        app.remote_access.step = 3;
        app.remote_access.status["running"] = json!(true);
        harness.run();
        harness.step();
        harness.step();
        if let Some(folder) = &output {
            harness.render().unwrap().save(folder.join("connections.png")).unwrap();
        }
        let app = harness.state_mut();
        assert!(!app.remote_access.has_unsaved_changes());
        action(app, &json!({"action":"done"})).unwrap();
        assert!(!app.remote_access.open);
    }
    #[test]
    fn remote_commands_and_synthetic_frames_cannot_open_or_manage_the_wizard() {
        let mut app = app();
        app.ui.answering_control = true;
        assert!(app.run("ui.remoteAccess", json!({})).is_err());
        assert!(app.run("ui.remoteAccess.action", json!({"action":"configure"})).is_err());
        assert!(!app.remote_access.open);
        app.ui.answering_control = false;
        app.synthetic_frame = true;
        assert!(app.run("ui.remoteAccess", json!({})).is_err());
        assert!(app.run("ui.remoteAccess.action", json!({"action":"copy_secret"})).is_err());
    }
    #[test]
    fn wizard_secrets_are_not_in_control_inspection_or_painted_text() {
        let mut app = app();
        app.run("ui.remoteAccess", json!({})).unwrap();
        app.remote_access.phrase = "owner-secret-not-serialized".into();
        app.remote_access.client_id = "safe-client-id".into();
        app.remote_access.client_secret = "client-secret-not-painted".into();
        let inspect = crate::control::inspect(&app, &egui::Context::default()).to_string();
        assert!(!inspect.contains("owner-secret-not-serialized"));
        assert!(!inspect.contains("client-secret-not-painted"));
        app.remote_access.step = 2;
        let text = crate::tests_labels::painted_text(&mut app, |app, ui| show(app, ui.ctx()));
        assert!(text.contains("Choose an owner passphrase"));
        assert!(!text.contains("owner-secret-not-serialized"));
        app.remote_access.step = 3;
        let text = crate::tests_labels::painted_text(&mut app, |app, ui| show(app, ui.ctx()));
        assert!(text.contains("Connection is stopped"));
        assert!(!text.contains("client-secret-not-painted"));
    }
    #[test]
    fn open_wizard_pauses_remote_edits_before_queuing_input() {
        let mut app = app();
        app.run("ui.remoteAccess", json!({})).unwrap();
        for method in ["engine.execute", "ui.pointer", "ui.key", "app.open", "app.quit"] {
            let (request, _reply) = crate::control::ControlRequest::new(method, json!({}));
            let response = crate::control::handle(&mut app, &egui::Context::default(), &request);
            match response {
                crate::control::Outcome::Done(value) => assert_eq!(value["ok"], false, "{method}: {value}"),
                _ => panic!("Control should have been refused"),
            }
        }
        assert!(app.synthetic.is_empty());
    }
    #[test]
    fn ordinary_and_web_builds_show_disabled_remote_actions_without_a_host_service() {
        let app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        assert!(!crate::menus::enabled(&app, "ui.remoteAccess"));
        assert!(!crate::menus::enabled(&app, "ui.remoteAccess.action"));
    }

    #[test]
    fn folder_picker_adds_and_removes_independent_locations_without_duplicates() {
        let mut app = app();
        app.services.pick_folder = Some(Box::new(|| Some("C:/chosen".into())));
        action(&mut app, &json!({"action":"choose_folder"})).unwrap();
        action(&mut app, &json!({"action":"choose_folder"})).unwrap();
        assert_eq!(app.remote_access.artworks, vec!["C:/chosen"]);
        action(&mut app, &json!({"action":"add_path"})).unwrap();
        assert_eq!(app.remote_access.artworks.len(), 2);
        action(&mut app, &json!({"action":"remove_folder","index":0})).unwrap();
        assert_eq!(app.remote_access.artworks, vec![""]);
        assert!(action(&mut app, &json!({"action":"remove_folder","index":10})).is_err());
        app.remote_access.artworks = vec!["C:/chosen".into(); 32];
        assert!(action(&mut app, &json!({"action":"add_path"})).is_err());
    }

    #[test]
    fn non_ascii_passphrases_follow_the_same_character_limits_as_the_host() {
        let mut app = app();
        for length in [4, 11, 12, 256, 257] {
            app.remote_access.phrase = "界".repeat(length);
            app.remote_access.phrase_confirmation = app.remote_access.phrase.clone();
            assert_eq!(app.remote_access.validate_phrase().is_ok(), (12..=256).contains(&length));
        }
    }

    #[test]
    fn folder_picker_accepts_an_async_answer_after_the_active_document_closes() {
        let mut app = app();
        app.run("file.new", json!({})).unwrap();
        app.services.pick_folder = Some(Box::new(|| panic!("the folder picker must run asynchronously")));
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut receiver = Some(receiver);
        app.services.start_pick = Some(Box::new(move |request| {
            assert!(matches!(request, crate::picks::PickRequest::Folder));
            receiver.take()
        }));
        app.run("ui.remoteAccess.action", json!({"action":"choose_folder"})).unwrap();
        assert!(app.file_dialog_open());
        assert!(app.remote_access.artworks.is_empty());
        app.session.execute("file.close", &json!({})).unwrap();
        sender.send(vec!["C:/chosen".into()]).unwrap();
        crate::picks::poll(&mut app, &egui::Context::default());
        assert!(!app.file_dialog_open());
        assert_eq!(app.remote_access.artworks, vec!["C:/chosen"]);
    }

    #[test]
    fn remote_ai_preferences_category_is_visible_only_with_native_host_support() {
        let mut app = app();
        crate::prefs_dialog::open(&mut app, Some("Remote AI"));
        let painted = crate::tests_labels::painted_text(&mut app, |app, ui| crate::prefs_dialog::show(app, ui.ctx()));
        assert!(painted.contains("Remote AI"));
        assert!(painted.contains("Open Remote AI connections"));
        let mut ordinary = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        crate::prefs_dialog::open(&mut ordinary, Some("Remote AI"));
        assert_eq!(ordinary.ui.dialog.as_ref().unwrap().str("__category"), "General");
    }

    #[test]
    fn closing_a_configuring_wizard_keeps_control_paused_until_new_authority_is_installed() {
        let root = std::env::temp_dir().join(format!("vc-remote-folder-transition-{}", std::process::id()));
        let first = root.join("first");
        let second = root.join("second");
        let private = root.join("Remote Access");
        for folder in [&first, &second, &private] {
            std::fs::create_dir_all(folder).unwrap();
        }
        let mut app = app();
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = calls.clone();
        app.services.remote_access = Some(Box::new(move |request| {
            assert_eq!(request["action"], "start");
            seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let (send, recv) = std::sync::mpsc::channel();
            send.send(Ok(json!({"running":true}))).unwrap();
            recv
        }));
        let (send, recv) = std::sync::mpsc::channel();
        app.remote_access.open = false;
        app.remote_access.pending = Some(("configure".into(), recv));
        assert!(app.remote_access.pauses_control());
        let (request, _) = crate::control::ControlRequest::new("app.open", json!({"path":"unapproved.svg"}));
        match crate::control::handle(&mut app, &egui::Context::default(), &request) {
            crate::control::Outcome::Done(value) => assert_eq!(value["ok"], false),
            _ => panic!("Configuration must pause control"),
        }
        assert!(app.control_rx.is_none(), "no listener before the owner saves setup");
        send.send(Ok(json!({"configured":true,"running":false,"artworkRoots":[first,second],"privateFolder":private,"controlPort":33982}))).unwrap();
        poll(&mut app, &egui::Context::default());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(app.control_rx.is_some(), "saving setup activates the control channel after folder checks");
        let roots = app.automation_roots.as_ref().unwrap();
        assert!(roots.check(&root.join("second/new.svg").to_string_lossy(), vectorcraft_engine::file_access::Access::Write).is_ok());
        assert!(roots.check(&root.join("Remote Access/settings.json").to_string_lossy(), vectorcraft_engine::file_access::Access::Read).is_err());
        assert!(!app.remote_access.pauses_control());
        let _ = std::fs::remove_dir_all(root);
    }
}
