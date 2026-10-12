//! VectorCraft in the browser.
//!
//! Runs the same [`vectorcraft_ui_egui::VectorcraftApp`] as the desktop app through eframe's web
//! runner (wgpu: WebGPU where available, WebGL2 otherwise). Build with `trunk build --release`
//! from this directory; see `docs/development.md` ("Web build").
//!
//! Differences from the desktop app:
//! - no TCP control server (browsers can't listen on sockets);
//! - File → Open uses the browser file picker; bytes arrive asynchronously through
//!   `Services::inbox`;
//! - Save / Export trigger a browser download;
//! - dropped files are read asynchronously by `web::WebShell` and delivered through the inbox.
//!
//! URL query flags: `?webgl` forces the WebGL2 backend instead of WebGPU; `?lang=es` (any
//! language code) stands in for the browser's languages when the interface language is Automatic.

#[cfg(target_arch = "wasm32")]
mod fonts;
#[cfg(target_arch = "wasm32")]
mod locks;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("vectorcraft-web only runs in the browser: build it with `trunk build --release` in apps/vectorcraft-web");
}

#[cfg(test)]
mod recovery_architecture_tests {
    #[test]
    fn recovery_has_no_handwritten_script_or_dom_stylesheet() {
        let page = include_str!("../index.html");
        assert!(!page.contains("session.js") && !page.contains("session.css"));
        assert!(!include_str!("craft_bridge.rs").contains("inline_js"));
        assert!(!std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("package.json").exists());
    }
}
