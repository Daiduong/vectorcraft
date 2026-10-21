//! RoboDhan-backed vector creation. Hiding the panel preserves prompt and pending work.
use crate::VectorcraftApp;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
};
pub type Service = Arc<dyn Fn(Value, Arc<AtomicBool>) -> Result<Value, String> + Send + Sync>;
#[derive(Default)]
pub struct State {
    pub prompt: String,
    pub edit_active: bool,
    pub status: String,
    model: String,
    pending: Option<Receiver<Result<Value, String>>>,
    cancel: Arc<AtomicBool>,
}
impl Drop for State {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
pub fn tick(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let result = match app.local_ai.pending.as_ref().map(|r| r.try_recv()) {
        Some(Ok(v)) => v,
        Some(Err(mpsc::TryRecvError::Disconnected)) => Err("Proses AI terputus".into()),
        _ => {
            if app.local_ai.pending.is_some() {
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
            }
            return;
        }
    };
    app.local_ai.pending = None;
    if app.local_ai.cancel.load(Ordering::Relaxed) {
        app.local_ai.status = "Hasil diabaikan".into();
        return;
    }
    match result {
        Ok(v) => {
            if let Some(models) = v["models"].as_array() {
                app.local_ai.model = models.first().and_then(Value::as_str).unwrap_or("").into();
                app.local_ai.status = if models.is_empty() {
                    "RoboDhan belum memiliki model chat".into()
                } else {
                    "Terhubung. Model mengikuti pilihan aktif di RoboDhan.".into()
                };
            } else if let Some(svg) = v["svg"].as_str() {
                match app.run("ai.openSvg", json!({"svg":svg})) {
                    Ok(_) => {
                        app.sync_views();
                        app.local_ai.status = "Ilustrasi dibuka di tab baru. Simpan melalui Save As.".into();
                    }
                    Err(e) => app.local_ai.status = e,
                }
            } else {
                app.local_ai.status = "Respons AI tidak dikenali".into();
            }
        }
        Err(e) => app.local_ai.status = e,
    }
}
fn start(app: &mut VectorcraftApp, ctx: &egui::Context, models: bool) {
    if app.local_ai.pending.is_some() {
        return;
    }
    let Some(service) = app.services.local_ai.clone() else { return };
    let source = if !models && app.local_ai.edit_active {
        let Some(st) = app.session.active() else {
            app.local_ai.status = "Buka ilustrasi yang ingin diedit terlebih dahulu".into();
            return;
        };
        let svg =
            vectorcraft_svg::export(&st.doc, &vectorcraft_svg::ExportOptions { preserve_editing: false, metadata: false, ..Default::default() });
        if let Err(e) = vectorcraft_engine::cmd::local_ai::validate(&svg) {
            app.local_ai.status = format!("Dokumen ini belum bisa dikirim ke AI: {e}");
            return;
        }
        svg
    } else {
        String::new()
    };
    let req = json!({"operation":if models{"models"}else{"svg"},"prompt":app.local_ai.prompt,"model":app.local_ai.model,"svg":source});
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    app.local_ai.cancel = cancel.clone();
    let ctx = ctx.clone();
    match std::thread::Builder::new().name("vectorcraft-robodhan".into()).spawn(move || {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| service(req, cancel))).unwrap_or_else(|_| Err("Proses AI gagal".into()));
        let _ = tx.send(result);
        ctx.request_repaint();
    }) {
        Ok(_) => {
            app.local_ai.pending = Some(rx);
            app.local_ai.status = "Menunggu RoboDhan…".into();
        }
        Err(e) => app.local_ai.status = e.to_string(),
    }
}
pub fn panel(app: &mut VectorcraftApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let mut action = None;
    let busy = app.local_ai.pending.is_some();
    egui::Panel::bottom("ai-studio").resizable(true).default_size(155.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong("AI Studio · RoboDhan");
            if ui.add_enabled(!busy, egui::Button::new("Hubungkan")).clicked() {
                action = Some(true);
            }
        });
        ui.add_enabled(
            !busy,
            egui::TextEdit::multiline(&mut app.local_ai.prompt)
                .hint_text("Deskripsikan ilustrasi atau perubahan yang Anda inginkan…")
                .desired_rows(2)
                .desired_width(f32::INFINITY)
                .char_limit(4000),
        );
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled(!busy && app.session.active().is_some(), egui::Checkbox::new(&mut app.local_ai.edit_active, "Edit ilustrasi aktif"));
            if ui.add_enabled(!busy && !app.local_ai.prompt.trim().is_empty(), egui::Button::new("Buat SVG")).clicked() {
                action = Some(false);
            }
            if busy {
                ui.spinner();
                if ui.add_enabled(!app.local_ai.cancel.load(Ordering::Relaxed), egui::Button::new("Abaikan hasil")).clicked() {
                    app.local_ai.cancel.store(true, Ordering::Relaxed);
                }
            }
            ui.label("Hasil menjadi tab baru; mode edit mengirim SVG aktif ke RoboDhan lokal.");
        });
        ui.label(&app.local_ai.status);
    });
    if let Some(models) = action {
        start(app, &ctx, models);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_panel_still_receives_result() {
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Default::default());
        app.ui.ai_studio = false;
        app.local_ai.prompt = "logo".into();
        let (tx, rx) = mpsc::channel();
        app.local_ai.pending = Some(rx);
        tx.send(Ok(json!({"models":["local"]}))).unwrap();
        tick(&mut app, &egui::Context::default());
        assert_eq!(app.local_ai.prompt, "logo");
        assert!(!app.ui.ai_studio);
        assert!(app.local_ai.pending.is_none());
    }
}
