//! `vectorcraft-cli`: VectorCraft from the command line.
//!
//! ```text
//! vectorcraft-cli mcp [--connect 127.0.0.1:7979 | --headless] [--automation-read-root DIR] [--automation-write-root DIR]
//! vectorcraft-cli run [--in FILE] [--cmd id [--params '{json}']]... [--export out.svg]... [--scale 2]
//! vectorcraft-cli commands
//! vectorcraft-cli convert IN OUT [--scale 2] [--artboard 0 | --range 1-3,5] [--outline-text] [--decimals N] [--replace-lossy] [--editing-data] [--ai-version 7|pdf] [--hidden-objects layer|omit|visible] [--transparent-images flatten|omit]
//! vectorcraft-cli info FILE
//! vectorcraft-cli bench FILE [--size 2880x1800] [--iters 5]
//! vectorcraft-cli perf [--paths 50000] [--deep]
//! ```
// Denied, not forbidden: the font listers shared with the desktop app allow it for their
// DirectWrite (Windows) and CoreText (macOS) calls, and nothing else may.
#![deny(unsafe_code)]

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

/// `println!` / `print!` that end the program quietly when stdout is closed
/// (`vectorcraft-cli commands | head`) instead of panicking with "failed printing to stdout:
/// Broken pipe (os error 32)".
macro_rules! outln {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        if let Err(e) = writeln!(std::io::stdout(), $($arg)*) {
            $crate::stdout_failed(e);
        }
    }};
}
macro_rules! out {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        if let Err(e) = write!(std::io::stdout(), $($arg)*) {
            $crate::stdout_failed(e);
        }
    }};
}

/// The desktop app's CoreText font lister, shared: exports and MCP find the same fonts.
#[cfg(target_os = "macos")]
#[path = "../../vectorcraft/src/mac_fonts.rs"]
mod mac_fonts;
mod perf;
/// The desktop app's settings folder, shared: exports and MCP read the same Fonts folder.
#[path = "../../vectorcraft/src/prefs_dir.rs"]
mod prefs_dir;
/// The desktop app's DirectWrite font lister, shared: exports and MCP find the same fonts.
#[cfg(all(windows, not(target_vendor = "win7")))]
#[path = "../../vectorcraft/src/system_fonts.rs"]
mod system_fonts;

use serde_json::{Value, json};
use vectorcraft_engine::file_access::{self, AutomationRoots};
use vectorcraft_mcp::{Backend, DEFAULT_ADDR, Headless, Remote, Server};

/// stdout went away. A reader that stopped early (a closed pipe) ends the program quietly, as
/// ripgrep does; any other write error is reported.
fn stdout_failed(e: std::io::Error) -> ! {
    if e.kind() == std::io::ErrorKind::BrokenPipe {
        std::process::exit(0);
    }
    eprintln!("vectorcraft-cli: can't write to stdout: {e}");
    std::process::exit(1);
}

const USAGE: &str = "\
vectorcraft-cli — VectorCraft automation

USAGE:
  vectorcraft-cli mcp [--connect ADDR | --headless] [--automation-read-root DIR] [--automation-write-root DIR]
      Run the MCP server on stdio. Default: connect to a running app at 127.0.0.1:7979
      (vectorcraft --control 7979), falling back to a headless in-process session.
      --automation-read-root / --automation-write-root confine the files the agent's commands read
      (open, place, relink, library loads…) and write (save, export, package, library saves…) to
      DIR, links followed; a root left out grants none of its access. They imply --headless: a
      running app enforces its own (vectorcraft --control PORT --automation-read-root DIR …).

  vectorcraft-cli run [--in FILE] [--cmd ID [--params JSON]]... [--export FILE]... [--scale N]
      Headless batch: open FILE (any readable format) or start a new document, run commands in
      order, export each FILE in the format its extension picks (see Writable formats). Prints one
      JSON result per step.

  vectorcraft-cli commands
      Print the command catalogue as JSON.

  vectorcraft-cli convert IN OUT [--scale N] [--artboard I | --range R] [--outline-text] [--decimals N] [--replace-lossy] [--editing-data] [--ai-version 7|pdf] [--hidden-objects layer|omit|visible] [--transparent-images flatten|omit]
      Open IN (any readable format) and export OUT in the format its extension picks (see Writable
      formats). --artboard is 0-based, --range 1-based (\"1-3,5\"); a PDF gets every artboard
      unless one of them is given, EPS the bounds of the art, the other formats the first artboard.
      Live effects are kept, and hidden layers and objects (written hidden in SVG and PSD), with
      SVG's data-* attributes; --outline-text writes SVG text as paths, --decimals sets SVG's coordinate
      precision (1–7, default 3). OUT may not be IN when reading
      IN left things out (hidden text, art or layers it could not read; the notes say which) unless
      --replace-lossy is given. --editing-data opens a .ai from its editing copy even where that
      draws differently from its PDF part (which can be older), as long as it can be read. The
      result has the export's warnings, and importWarnings for what reading IN left out.
      --ai-version 7 writes a .ai as the legacy version 7 (layers, groups and type kept; one file
      per artboard, OUT-<artboard>.ai when there are several; default pdf, PDF-compatible), with
      --hidden-objects (default layer: a hidden layer above each layer's hidden objects; omit;
      visible), --transparent-images (default flatten, with the art under them; omit) and
      --outline-text (type as outlines instead of text).

  vectorcraft-cli info FILE
      Print a JSON summary: the import warnings (what didn't come in as it was, such as an EPS
      read from its preview and why), title, colour mode, units, artboards, object counts by
      kind, fonts.

  vectorcraft-cli bench FILE [--size WxH] [--iters N]
      Render FILE (any readable format) fitted to WxH (default 2880x1800) and print ms per frame
      (warm), multithreaded and single-threaded.

  vectorcraft-cli perf [--paths N] [--deep]
      Check the performance budgets (render, pan, hit test, save/load, SVG, Pathfinder, handle
      and segment drags) on a synthetic N-path document (default 50000). Exits non-zero if a
      budget is exceeded, unless the report says the machine is busy. --deep also times work with
      no budget yet and prints memory use (the working set on Windows), with the peak so far on
      Windows and Linux.
";

/// The usage text plus the formats `document.open` reads and `document.export` writes.
fn usage() -> String {
    use vectorcraft_engine::cmd::fileio::{OPEN_EXTS, export_extensions};
    let http = if cfg!(all(feature = "http", not(target_arch = "wasm32"))) {
        "\nExperimental local HTTP:\n  vectorcraft-cli mcp --http 127.0.0.1:33981 --http-token-file PATH --connect 127.0.0.1:33982\n  Requires a private 64-hex-character bearer token file. No public binding or headless fallback.\n"
    } else {
        ""
    };
    format!("{USAGE}\nReadable formats: .{}\nWritable formats: .{}\n{http}", OPEN_EXTS.join(", ."), export_extensions().join(", ."))
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--profile-dir") {
        let result = args
            .get(1)
            .ok_or_else(|| "--profile-dir needs an absolute path".to_string())
            .and_then(|p| prefs_dir::set_explicit_profile(std::path::PathBuf::from(p)));
        if let Err(error) = result {
            eprintln!("vectorcraft-cli: {error}");
            return ExitCode::FAILURE;
        }
        args.drain(..2);
    }
    // Before any font lookup: the fonts font services load (#579).
    #[cfg(all(windows, not(target_vendor = "win7")))]
    system_fonts::install();
    #[cfg(target_os = "macos")]
    mac_fonts::install();
    prefs_dir::install_fonts();
    let r = match args.first().map(String::as_str) {
        #[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
        Some("remote-configure") => remote_configure(&args[1..]),
        #[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
        Some("remote-http") => match args.as_slice() {
            [_, flag, path] if flag == "--config" => vectorcraft_mcp::remote::run(Path::new(path)),
            _ => Err("Use: vectorcraft-cli remote-http --config PATH".into()),
        },
        Some("mcp") => mcp(&args[1..]),
        Some("run") => run(&args[1..]),
        Some("commands") => commands(),
        Some("convert") => convert(&args[1..]),
        Some("info") => info(&args[1..]),
        Some("bench") => bench(&args[1..]),
        Some("perf") => perf::run(&args[1..]),
        Some("-h" | "--help" | "help") | None => {
            out!("{}", usage());
            Ok(())
        }
        Some("-V" | "--version") => {
            outln!("vectorcraft-cli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(other) => Err(format!("unknown subcommand `{other}`\n\n{}", usage())),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("vectorcraft-cli: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Noninteractive owner setup for local deployments and protocol tests. The passphrase is read
/// from stdin, never from process arguments. The desktop wizard writes the same settings format.
#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
fn remote_configure(args: &[String]) -> Result<(), String> {
    use std::io::Read as _;
    use vectorcraft_mcp::remote::config::{Settings, hash_phrase, secret};
    let mut fields = std::collections::BTreeMap::new();
    let mut stdin = false;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        if flag == "--passphrase-stdin" {
            if stdin {
                return Err("Passphrase input was specified twice".into());
            }
            stdin = true;
            continue;
        }
        if !["--profile", "--app", "--origin", "--bind", "--control", "--artwork-root"].contains(&flag.as_str()) {
            return Err(format!("Unknown remote setup flag: {flag}"));
        }
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        if fields.insert(flag.as_str(), value.as_str()).is_some() {
            return Err(format!("{flag} was specified twice"));
        }
    }
    if !stdin {
        return Err("Use --passphrase-stdin; passphrases must not appear on the command line".into());
    }
    let required = |key: &str| fields.get(key).copied().ok_or_else(|| format!("{key} is required"));
    let profile = std::path::PathBuf::from(required("--profile")?);
    if !profile.is_absolute() {
        return Err("Use an absolute profile path".into());
    }
    std::fs::create_dir_all(&profile).map_err(|e| e.to_string())?;
    let mut phrase = String::new();
    std::io::stdin().take(1025).read_to_string(&mut phrase).map_err(|e| e.to_string())?;
    let settings = Settings {
        version: 1,
        enabled: true,
        public_origin: required("--origin")?.trim_end_matches('/').into(),
        base_path: "/vectorcraft-native".into(),
        bind: required("--bind")?.into(),
        control_port: required("--control")?.parse().map_err(|_| "Invalid control port")?,
        app_path: required("--app")?.into(),
        profile_dir: profile.clone(),
        artwork_root: required("--artwork-root")?.into(),
        additional_artwork_roots: Vec::new(),
        owner_hash: hash_phrase(phrase.trim_end_matches(['\r', '\n']))?,
        management_token: secret(),
        allow_test_callbacks: false,
    };
    let path = profile.join("Remote Access/settings.json");
    if path.exists() {
        return Err("This profile already has remote settings; use its local connection wizard to update them".into());
    }
    settings.save(&path)?;
    outln!("{}", json!({"configured":true,"settings":path,"endpoint":settings.resource()}));
    Ok(())
}

/// A headless session that remembers what the user keeps between runs: the PDF presets saved, the
/// ICC profiles loaded and the plug-ins installed (#2168, #2174, #2541), in the folders next to the preferences that the desktop
/// app uses too. `VECTORCRAFT_NO_PREFS` and `VECTORCRAFT_CONFIG_DIR` apply as they do there.
fn with_stores(mut h: Headless) -> Headless {
    prefs_dir::attach_profile_store(&mut h.session);
    prefs_dir::attach_plugin_store(&mut h.session);
    prefs_dir::attach_pdf_preset_store(&mut h.session);
    h
}

fn mcp(args: &[String]) -> Result<(), String> {
    let mut connect: Option<String> = None;
    let mut headless = false;
    #[cfg(all(feature = "http", not(target_arch = "wasm32")))]
    let (mut http_address, mut http_token_file): (Option<String>, Option<String>) = (None, None);
    let (mut read_root, mut write_root): (Option<String>, Option<String>) = (None, None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        // The roots also come as `--flag=DIR`, as the other Craft apps take them.
        let (flag, inline) = match a.split_once('=') {
            Some((f, v)) if f.starts_with("--automation-") => (f, Some(v.to_string())),
            _ => (a.as_str(), None),
        };
        match flag {
            "--connect" => connect = Some(it.next().cloned().ok_or("--connect needs an address")?),
            "--headless" => headless = true,
            #[cfg(all(feature = "http", not(target_arch = "wasm32")))]
            "--http" => {
                if http_address.replace(it.next().cloned().ok_or("--http needs a loopback address")?).is_some() {
                    return Err("--http is given twice".into());
                }
            }
            #[cfg(all(feature = "http", not(target_arch = "wasm32")))]
            "--http-token-file" => {
                if http_token_file.replace(it.next().cloned().ok_or("--http-token-file needs a path")?).is_some() {
                    return Err("--http-token-file is given twice".into());
                }
            }
            "--automation-read-root" | "--automation-write-root" => {
                let dir = inline.or_else(|| it.next().cloned()).ok_or_else(|| format!("{flag} needs a folder"))?;
                let slot = if flag == "--automation-read-root" { &mut read_root } else { &mut write_root };
                if slot.replace(dir).is_some() {
                    return Err(format!("{flag} is given twice: it takes one folder"));
                }
            }
            other => return Err(format!("unknown mcp option `{other}`")),
        }
    }
    if headless && connect.is_some() {
        return Err("use either --connect or --headless".into());
    }
    let roots = AutomationRoots::new(read_root.as_deref().map(Path::new), write_root.as_deref().map(Path::new))?;
    #[cfg(all(feature = "http", not(target_arch = "wasm32")))]
    if http_address.is_some() || http_token_file.is_some() {
        let address = http_address.ok_or("--http-token-file requires --http")?;
        let token_file = http_token_file.ok_or("--http requires --http-token-file")?;
        if headless || roots.is_some() {
            return Err("the HTTP prototype requires a live desktop backend; set automation roots on the desktop app".into());
        }
        let desktop = connect.as_deref().unwrap_or(DEFAULT_ADDR);
        let desktop_address: std::net::SocketAddr = desktop.parse().map_err(|_| "HTTP --connect must be a loopback IP:port")?;
        if !desktop_address.ip().is_loopback() {
            return Err("HTTP --connect must target a loopback desktop address".into());
        }
        let options = vectorcraft_mcp::http::Options::from_file(&address, Path::new(&token_file))?;
        let endpoint = vectorcraft_mcp::http::Endpoint::bind(options)?;
        eprintln!("vectorcraft-cli: experimental local HTTP at http://{}/mcp (bearer token required)", endpoint.address());
        return endpoint.serve(Box::new(Remote::deferred(desktop)));
    }
    if roots.is_some() && connect.is_some() {
        return Err("--automation-read-root and --automation-write-root confine the headless server; a running app enforces its own roots:                     start it with `vectorcraft --control <port> --automation-read-root <dir> --automation-write-root <dir>`"
            .into());
    }
    let backend: Box<dyn Backend> = if let Some(roots) = roots {
        // Every thread of this process, not only the server's: nothing reaches past the roots.
        file_access::confine_process(roots.clone());
        let show = |root: Option<&Path>| root.map_or_else(|| "none".to_string(), |r| r.display().to_string());
        eprintln!("vectorcraft-cli: files confined: read root {}, write root {}", show(roots.read_root()), show(roots.write_root()));
        Box::new(Headless::with_document().with_automation_roots(Some(roots)))
    } else if headless {
        Box::new(with_stores(Headless::with_document()))
    } else if let Some(addr) = connect {
        // Explicit address: fail loudly if the app isn't there.
        Box::new(Remote::connect(&addr).map_err(|e| format!("cannot connect to {addr}: {e}"))?)
    } else {
        match Remote::connect(DEFAULT_ADDR) {
            Ok(r) => Box::new(r),
            Err(_) => Box::new(with_stores(Headless::with_document())),
        }
    };
    eprintln!("vectorcraft-cli: MCP server on stdio ({})", backend.describe());
    // The binary owns the logger, not the library: installing one here keeps an embedder that
    // uses `vectorcraft_mcp` free to bring its own. Silent until a client sends logging/setLevel.
    vectorcraft_mcp::logging::install();
    let stdin = std::io::stdin();
    // The server reads and writes from threads of its own, so it takes the handles, not locks held here.
    Server::new(backend).serve(std::io::BufReader::new(stdin), std::io::stdout()).map_err(|e| e.to_string())
}

fn commands() -> Result<(), String> {
    let mut h = Headless::new();
    let v = h.call("engine.commands", json!({}))?;
    outln!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    Ok(())
}

fn convert(args: &[String]) -> Result<(), String> {
    let mut files = vec![];
    let (mut scale, mut artboard, mut range, mut outline_text, mut replace_lossy, mut editing_data) =
        (1.0f64, None::<u64>, None::<String>, false, false, false);
    let mut decimals = None::<u64>;
    // Legacy `.ai` options, passed on as the export's own (`aiVersion`, `hiddenObjects`,
    // `transparentImages`); the engine checks their values.
    let mut legacy: Vec<(&str, String)> = vec![];
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--scale" | "-s" => scale = it.next().and_then(|v| v.parse().ok()).ok_or("--scale needs a number")?,
            "--artboard" | "-a" => artboard = Some(it.next().and_then(|v| v.parse().ok()).ok_or("--artboard needs an index")?),
            "--range" | "-r" => range = Some(it.next().cloned().ok_or("--range needs artboards such as 1-3,5")?),
            "--outline-text" => outline_text = true,
            "--decimals" => decimals = Some(it.next().and_then(|v| v.parse::<u64>().ok()).ok_or("--decimals needs a number")?),
            "--replace-lossy" => replace_lossy = true,
            "--editing-data" => editing_data = true,
            "--ai-version" => legacy.push(("aiVersion", it.next().cloned().ok_or("--ai-version needs 7 or pdf")?)),
            "--hidden-objects" => legacy.push(("hiddenObjects", it.next().cloned().ok_or("--hidden-objects needs layer, omit or visible")?)),
            "--transparent-images" => legacy.push(("transparentImages", it.next().cloned().ok_or("--transparent-images needs flatten or omit")?)),
            f => files.push(f.to_string()),
        }
    }
    let [input, output] = <[String; 2]>::try_from(files).map_err(|_| "convert needs IN and OUT")?;
    let mut h = Headless::new();
    let opened = if editing_data {
        let params = json!({"path": input, "editingData": "always"});
        h.call("engine.execute", json!({"command": "document.open", "params": params})).map_err(|e| format!("open {input}: {e}"))?
    } else {
        h.call("app.open", json!({"path": input})).map_err(|e| format!("open {input}: {e}"))?
    };
    // A conversion keeps hidden layers and objects, written hidden, where the format can (SVG,
    // PSD).
    let mut params = json!({"path": output, "scale": scale, "artboard": artboard, "range": range, "outlineText": outline_text, "hiddenLayers": true, "acknowledgeLoss": replace_lossy});
    if let Some(decimals) = decimals {
        params["decimals"] = json!(decimals);
    }
    for (key, value) in legacy {
        params[key] = json!(value);
    }
    let mut r = h.call("engine.execute", json!({"command": "document.export", "params": params})).map_err(|e| format!("export {output}: {e}"))?;
    // What reading the input left out (a missing linked file, an unsupported filter…) matters as
    // much as what writing the output did.
    if let (Some(w), Some(out)) = (opened.get("warnings").filter(|w| w.as_array().is_some_and(|a| !a.is_empty())), r.as_object_mut()) {
        out.insert("importWarnings".into(), w.clone());
    }
    outln!("{r}");
    Ok(())
}

fn info(args: &[String]) -> Result<(), String> {
    let file = args.first().ok_or("info needs a FILE")?;
    let mut h = Headless::new();
    let opened = h.call("app.open", json!({"path": file})).map_err(|e| format!("open {file}: {e}"))?;
    let base = h.call("engine.execute", json!({"command": "file.info", "params": {}}))?;
    let doc = h.session.doc().map_err(|e| e.to_string())?.doc.clone();
    let mut kinds: std::collections::BTreeMap<&'static str, usize> = Default::default();
    doc.walk(|n| *kinds.entry(n.kind_label()).or_default() += 1);
    let fonts = h.call("engine.execute", json!({"command": "text.fonts", "params": {}})).unwrap_or(Value::Null);
    let artboards: Vec<Value> =
        doc.artboards.iter().map(|a| json!({"name": a.name, "rect": [a.rect.x0, a.rect.y0, a.rect.width(), a.rect.height()]})).collect();
    // What didn't come in as it was (an EPS read from its preview says why).
    let v = json!({"file": file, "warnings": opened["warnings"], "info": base, "artboards": artboards, "kinds": kinds, "fonts": fonts});
    outln!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    Ok(())
}

enum Step {
    Cmd(String, Value),
    Export(String),
}

fn run(args: &[String]) -> Result<(), String> {
    let mut input: Option<String> = None;
    let mut steps: Vec<Step> = vec![];
    let mut scale = 1.0;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |what: &str| it.next().cloned().ok_or_else(|| format!("{a} needs {what}"));
        match a.as_str() {
            "--in" | "-i" => input = Some(val("a file")?),
            "--cmd" | "-c" => steps.push(Step::Cmd(val("a command id")?, json!({}))),
            "--params" | "-p" => {
                let raw = val("JSON")?;
                let p: Value = serde_json::from_str(&raw).map_err(|e| format!("--params {raw}: {e}"))?;
                if !p.is_object() {
                    return Err(format!("--params must be a JSON object, got {raw}"));
                }
                match steps.last_mut() {
                    Some(Step::Cmd(_, params)) => *params = p,
                    _ => return Err("--params must follow a --cmd".into()),
                }
            }
            "--export" | "-o" => steps.push(Step::Export(val("a file")?)),
            "--scale" | "-s" => {
                let raw = val("a number")?;
                scale = raw.parse::<f64>().map_err(|_| format!("--scale {raw}: not a number"))?;
            }
            other => return Err(format!("unknown run option `{other}`")),
        }
    }

    let mut h = with_stores(Headless::new());
    let mut out = std::io::stdout().lock();
    // A reader that stopped early (a closed pipe) isn't an error, and the batch goes on: its later
    // steps (an export…) still run, unprinted.
    let mut closed = false;
    let mut emit = |v: Value| {
        if closed {
            return Ok(());
        }
        match writeln!(out, "{v}") {
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                closed = true;
                Ok(())
            }
            r => r.map_err(|e| e.to_string()),
        }
    };
    if let Some(path) = &input {
        let r = h.call("app.open", json!({"path": path})).map_err(|e| format!("open {path}: {e}"))?;
        emit(json!({"step": "open", "path": path, "result": r}))?;
    } else if !matches!(steps.first(), Some(Step::Cmd(id, _)) if id == "file.new") {
        h.ensure_document();
    }
    // Each step's result, for later steps to refer to (`"$1.id"`, see `vectorcraft_engine::steps`).
    let mut results = vec![];
    for step in steps {
        let r = match step {
            Step::Cmd(id, params) => {
                let params = vectorcraft_engine::steps::resolve(&params, &results).map_err(|e| format!("{id}: {e}"))?;
                let r = h.call("engine.execute", json!({"command": id, "params": params})).map_err(|e| format!("{id}: {e}"))?;
                emit(json!({"step": "cmd", "command": id, "result": r}))?;
                r
            }
            Step::Export(path) => {
                let r = h.call("app.export", json!({"path": path, "scale": scale})).map_err(|e| format!("export {path}: {e}"))?;
                emit(json!({"step": "export", "result": r}))?;
                r
            }
        };
        results.push(r);
    }
    Ok(())
}

fn bench(args: &[String]) -> Result<(), String> {
    let mut file = None;
    let (mut w, mut h, mut iters) = (2880u32, 1800u32, 5u32);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--size" => {
                let v = it.next().ok_or("--size needs WxH")?;
                let (a, b) = v.split_once('x').ok_or("--size needs WxH")?;
                w = a.parse().map_err(|_| "bad width")?;
                h = b.parse().map_err(|_| "bad height")?;
            }
            "--iters" => iters = it.next().and_then(|v| v.parse().ok()).ok_or("--iters needs a number")?,
            f if file.is_none() => file = Some(f.to_string()),
            other => return Err(format!("unknown bench option `{other}`")),
        }
    }
    let file = file.ok_or("bench needs a FILE")?;
    let mut hl = Headless::new();
    hl.call("app.open", json!({ "path": file })).map_err(|e| format!("open {file}: {e}"))?;
    let doc = hl.session.doc().map_err(|e| e.to_string())?.doc.clone();
    let b = doc.artboards.first().map(|a| a.rect).ok_or("document has no artboard")?;
    let z = (w as f64 / b.width()).min(h as f64 / b.height()) * 0.95;
    let view = vectorcraft_geom::Affine::translate((w as f64 / 2.0, h as f64 / 2.0))
        * vectorcraft_geom::Affine::scale(z)
        * vectorcraft_geom::Affine::translate(-b.center().to_vec2());
    let opts = vectorcraft_render::RenderOptions::default();
    outln!("{file}: {} nodes, {w}x{h}", doc.layers.iter().map(|l| l.count()).sum::<usize>());
    for threads in [vectorcraft_render::default_threads(), 0] {
        let mut r = vectorcraft_render::Renderer::new();
        r.threads = threads;
        r.render(&doc, w, h, view, &opts);
        let t = std::time::Instant::now();
        for _ in 0..iters {
            r.render(&doc, w, h, view, &opts);
        }
        outln!(
            "  threads {threads}: {:.1} ms/frame (drawn {}, culled {})",
            t.elapsed().as_secs_f64() * 1000.0 / iters as f64,
            r.stats.drawn,
            r.stats.culled
        );
    }
    Ok(())
}
