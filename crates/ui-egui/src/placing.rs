//! Placing files off the UI thread (desktop): a file the user places (the Place dialog's OK, a file
//! dropped on the canvas, the OK of the PDF, DXF and text import options Place asks for) is read and
//! decoded on a thread that reads one file at a time while the status bar shows it, and is placed in
//! the document it was placed in once it is ready, as it would be at once: the same object, undo
//! step, selection and errors. The files the Place dialog loads the place cursor with are read, and
//! their thumbnails made, the same way, and so is the file each click or drag of the place cursor
//! places. A Paste the user gives decodes another app's bitmap on the same thread, and runs once it
//! is decoded ([`adopt_for_paste`]). Files are read and placed in the order they were started.
//! `file.place`, `file.place.queue` and `edit.paste*` from MCP and scripts run at once, and so does
//! every place and paste on wasm (no threads), with `VECTORCRAFT_SYNC_OPEN` set, and in unit tests
//! that do not set `off_the_ui_thread` ([`available`]).

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use vectorcraft_engine::cmd::clipboard::{Clipboard, Flavour, ImageRead, read_image};
use vectorcraft_engine::cmd::place::{PlaceRead, QueueRead};
use vectorcraft_engine::file_access::ScopeHandle;

use crate::VectorcraftApp;

/// What was read: one file to place, the files to load the place cursor with, or a bitmap to paste.
enum Read {
    Place(Box<PlaceRead>),
    Queue(QueueRead),
    Paste(Box<Paste>),
}

/// A paste whose bitmap was decoded off the UI thread ([`adopt_for_paste`]).
struct Paste {
    /// The paste command and its params.
    id: String,
    params: Value,
    /// What it pastes (`None`: a bitmap the host converts couldn't be converted, and the paste
    /// takes the internal clipboard as it was when the paste started, as at once).
    pasted: Option<Pasted>,
    /// The view's center when the paste started, which the pasted objects are centered on.
    center: Option<[f64; 2]>,
    /// The objects selected when the paste started, which Paste in Front and Paste in Back paste
    /// next to (`None`: the paste follows an earlier one of its chain, and takes the selection
    /// that one left, [`Chain::left`]).
    selection: Option<Vec<vectorcraft_doc::NodeId>>,
    /// The number of its chain in [`Placing::chains`].
    chain: u64,
    /// Where new art went when the paste started, which is where the paste goes.
    insertion: Insertion,
    /// The latest Copy or Cut when the paste started ([`crate::sysclip::Publishing::latest`]).
    copied: u64,
    /// The internal clipboard when the paste started, which it pastes when [`Self::pasted`] is
    /// `None`.
    clipboard: Clipboard,
}

/// Where new art goes in the active document: the current layer, the isolated group or object, and
/// the drawing mode with the object Draw Inside draws into.
#[derive(Clone, PartialEq)]
struct Insertion {
    layer: Option<vectorcraft_doc::NodeId>,
    isolation: Option<vectorcraft_doc::NodeId>,
    draw_mode: vectorcraft_engine::DrawMode,
    draw_inside: Option<vectorcraft_doc::NodeId>,
}

impl Insertion {
    /// Where new art goes in `app`'s active document now.
    fn of(app: &VectorcraftApp) -> Option<Self> {
        let st = app.session.doc().ok()?;
        Some(Self { layer: st.active_layer, isolation: st.isolation, draw_mode: app.session.draw_mode, draw_inside: app.session.draw_inside })
    }

    /// Make it where new art goes in `app`'s active document → where new art went before.
    fn set(self, app: &mut VectorcraftApp) -> Option<Self> {
        let st = app.session.doc_mut().ok()?;
        let (layer, isolation) = (std::mem::replace(&mut st.active_layer, self.layer), std::mem::replace(&mut st.isolation, self.isolation));
        let draw_mode = std::mem::replace(&mut app.session.draw_mode, self.draw_mode);
        let draw_inside = std::mem::replace(&mut app.session.draw_inside, self.draw_inside);
        Some(Self { layer, isolation, draw_mode, draw_inside })
    }

    /// Put back each part of `self` (where new art went before [`Self::set`] set `started`) that
    /// differs from `started`, what the user changed meanwhile; a part the paste itself changed
    /// stays as the paste left it.
    fn restore_changed(self, started: &Self, app: &mut VectorcraftApp) {
        if let Ok(st) = app.session.doc_mut() {
            if self.layer != started.layer {
                st.active_layer = self.layer;
            }
            if self.isolation != started.isolation {
                st.isolation = self.isolation;
            }
        }
        if self.draw_mode != started.draw_mode {
            app.session.draw_mode = self.draw_mode;
        }
        if self.draw_inside != started.draw_inside {
            app.session.draw_inside = self.draw_inside;
        }
    }
}

/// Pastes decoded off the UI thread that follow each other in one document: each was given while the
/// one before waited, with the selection and the insertion context that one started with or, once
/// it ran, left.
struct Chain {
    /// The selection and the insertion context a paste given now must have to join the chain.
    context: (Vec<vectorcraft_doc::NodeId>, Insertion),
    /// The selection the chain's latest paste left, which the next one takes.
    left: Option<Vec<vectorcraft_doc::NodeId>>,
    /// How many of its pastes wait.
    waiting: usize,
}

/// What a paste whose bitmap was decoded off the UI thread pastes.
enum Pasted {
    /// The bitmap's type and the image read from it.
    Image(&'static str, Result<ImageRead, String>),
    /// Another flavour: the text that names the picture, when the picture couldn't be made.
    Flavour(Flavour),
}

/// Files being read for Place.
pub struct Pending {
    /// The file's name ("photo.tif"; the names joined with ", " for the place cursor's files).
    pub(crate) name: String,
    /// What the status bar shows ("Placing photo.tif").
    pub label: String,
    /// The document they are placed in ([`vectorcraft_engine::DocState::uid`]).
    doc: u64,
    /// Whether the file was dropped on the canvas (its messages say so, as
    /// [`crate::place::drop_files`]'s do).
    dropped: bool,
    /// Whether it is a paste's bitmap ([`adopt_for_paste`]) rather than a file.
    paste: bool,
    /// The number of the paste's chain ([`Placing::chains`]).
    chain: Option<u64>,
    done: Receiver<Result<Read, String>>,
}

/// A read waiting for the thread that reads the files.
type Job = Box<dyn FnOnce() + Send>;

/// The files being read for Place, in the order they were started.
#[derive(Default)]
pub struct Placing {
    pub pending: Vec<Pending>,
    /// Set by unit tests whose app reads the files the user places off the UI thread.
    pub(crate) off_the_ui_thread: bool,
    /// Set while [`crate::menus::invoke`] runs a Paste the user gave.
    pub(crate) for_user: bool,
    /// Set while a paste whose bitmap was converted off the UI thread runs: it takes the internal
    /// clipboard as it is.
    adopted: bool,
    /// The chains of pastes decoded off the UI thread that have pastes waiting, by number.
    chains: HashMap<u64, Chain>,
    /// For each document, the number of its latest chain, which a paste given there may join.
    latest_chain: HashMap<u64, u64>,
    /// The number the next chain takes.
    next_chain: u64,
    /// The thread that reads the files, one at a time in the order they were started: started with
    /// the first read, it ends when the app does.
    reader: Option<Sender<Job>>,
    /// Unit tests: how many reads run now, and the most that ran at once.
    #[cfg(test)]
    pub(crate) reading: std::sync::Arc<[std::sync::atomic::AtomicUsize; 2]>,
}

/// Does `app` read a file the user places off the UI thread? Where documents open off it
/// ([`crate::opening::available`]), and in the unit tests that set `off_the_ui_thread`.
pub fn available(app: &VectorcraftApp) -> bool {
    app.placing.off_the_ui_thread || crate::opening::available()
}

/// `file.place` with `p` for the active document, its file read and decoded off the UI thread
/// ([`vectorcraft_engine::Session::prepare_place`], [`vectorcraft_engine::cmd::place::PlaceJob::read`])
/// → `{background: true, status}`; `dropped` when it was dropped on the canvas. The params are
/// checked here first, so a bad one is an error at once. When no thread could start, the file is
/// placed at once → `file.place`'s reply.
pub fn start(app: &mut VectorcraftApp, p: &Value, dropped: bool) -> Result<Value, String> {
    let job = app.session.prepare_place(p).map_err(|e| e.to_string())?;
    let name = job.name();
    let started = spawn(app, name.clone(), name, dropped, move || job.read().map(|r| Read::Place(Box::new(r))).map_err(|e| e.to_string()))?;
    started.map_or_else(
        || {
            let r = app.session.execute("file.place", p).map_err(|e| e.to_string());
            app.sync_views();
            r
        },
        Ok,
    )
}

/// `file.place.queue` with `p` for the active document, its files read and their thumbnails made off
/// the UI thread → `{background: true, status}`; the place cursor is loaded once they are ready. When no thread could start, the cursor is loaded at once → `file.place.queue`'s reply.
pub fn start_queue(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let job = app.session.prepare_place_queue(p).map_err(|e| e.to_string())?;
    let names = job.names().join(", ");
    let started = spawn(app, names.clone(), names, false, move || job.read().map(Read::Queue).map_err(|e| e.to_string()))?;
    match started {
        Some(r) => Ok(r),
        None => {
            let r = app.session.execute("file.place.queue", p).map_err(|e| e.to_string());
            app.sync_views();
            let r = r?;
            crate::place::show_thumbnails(app, &r);
            Ok(r)
        }
    }
}

/// Before paste command `id` with `p`: what another app put on the system clipboard replaces the
/// internal clipboard ([`VectorcraftApp::adopt_system_clipboard`]) → `None`, and the paste runs at
/// once. For a Paste the user gives ([`crate::menus::invoke`]) where placed files are read off the
/// UI thread ([`available`]), the bitmap's bytes are copied off the system clipboard at once and
/// decoded on the thread that reads placed files, after the files started before it →
/// `{background: true, status}`: a bitmap the host converts to PNG is converted there
/// ([`crate::SystemClipboard::read_deferred`]), and the bitmap is read as `clipboard.importImage`
/// reads it ([`read_image`]). The paste runs once it is decoded, in the document it was given in,
/// as it would at once ([`poll`]). When no thread could start, the bitmap is read at once.
pub fn adopt_for_paste(app: &mut VectorcraftApp, id: &str, p: &Value) -> Result<Option<Value>, String> {
    if std::mem::take(&mut app.placing.adopted) {
        app.clipboard_image_request = None;
        return Ok(None);
    }
    if !(app.placing.for_user && available(app)) {
        return app.adopt_system_clipboard().map(|()| None);
    }
    let center = app.view().map(|v| [v.center.x, v.center.y]);
    let Some(make) = app.adopt_system_clipboard_or_defer(true)? else { return Ok(None) };
    let copied = app.clipboard_publishing.latest();
    let (doc, objects) = app.session.doc().map(|st| (st.uid, st.selection.objects.clone())).map_err(|e| e.to_string())?;
    let insertion = Insertion::of(app).ok_or("no document")?;
    // A paste given while one of the document's latest chain waits, with the selection and the
    // insertion context that chain expects, joins it: it goes next to what the paste before it
    // leaves selected, as at once. Otherwise it starts a chain.
    let context = (objects, insertion.clone());
    let joins =
        app.placing.latest_chain.get(&doc).copied().filter(|c| app.placing.chains.get(c).is_some_and(|ch| ch.waiting > 0 && ch.context == context));
    let (chain, selection) = match joins {
        Some(chain) => (chain, None),
        None => (app.placing.next_chain, Some(context.0.clone())),
    };
    let clipboard = app.session.clipboard.clone();
    let (id, params) = (id.to_string(), p.clone());
    // The work stays here when no thread could start.
    let slot = Arc::new(Mutex::new(Some(make)));
    let work = slot.clone();
    let started = spawn_labeled(app, "clipboard".into(), "Pasting".into(), false, Some(chain), move || {
        let make = work.lock().ok().and_then(|mut m| m.take());
        let pasted = make.and_then(|m| m()).map(|f| {
            if crate::sysclip::imports_as_image(f.mime) {
                Pasted::Image(f.mime, read_image(&f.data).map_err(|e| e.to_string()))
            } else {
                Pasted::Flavour(f)
            }
        });
        Ok(Read::Paste(Box::new(Paste { id, params, pasted, center, selection, chain, insertion, copied, clipboard })))
    })?;
    if started.is_some() {
        if joins.is_none() {
            app.placing.next_chain += 1;
            app.placing.chains.insert(chain, Chain { context, left: None, waiting: 0 });
            app.placing.latest_chain.insert(doc, chain);
        }
        if let Some(ch) = app.placing.chains.get_mut(&chain) {
            ch.waiting += 1;
        }
        return Ok(started);
    }
    match slot.lock().ok().and_then(|mut m| m.take()).and_then(|m| m()) {
        Some(f) => app.adopt_flavour(&f, center).map(|()| None),
        None => Ok(None),
    }
}

/// Run `read` for the active document on the thread that reads the files, after the reads started
/// before it (so a batch of large pictures is decoded one at a time, as at once) →
/// `{background: true, status}`, or `None` when that thread could not start.
fn spawn(
    app: &mut VectorcraftApp,
    name: String,
    shown: String,
    dropped: bool,
    read: impl FnOnce() -> Result<Read, String> + Send + 'static,
) -> Result<Option<Value>, String> {
    spawn_labeled(app, name, format!("Placing {shown}"), dropped, None, read)
}

/// [`spawn`] with the status bar's `label`; `chain` for a paste's bitmap, its chain's number.
fn spawn_labeled(
    app: &mut VectorcraftApp,
    name: String,
    label: String,
    dropped: bool,
    chain: Option<u64>,
    read: impl FnOnce() -> Result<Read, String> + Send + 'static,
) -> Result<Option<Value>, String> {
    let doc = app.session.doc().map_err(|e| e.to_string())?.uid;
    let (tx, done) = channel();
    // The automation roots in force here are in force there.
    let scope = ScopeHandle::capture();
    #[cfg(test)]
    let reading = app.placing.reading.clone();
    let job: Job = Box::new(move || {
        #[cfg(test)]
        {
            let now = reading[0].fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            reading[1].fetch_max(now, std::sync::atomic::Ordering::SeqCst);
        }
        let r = scope
            .run(|| vectorcraft_engine::guard::catch_panic(read))
            .unwrap_or_else(|msg| Err(format!("internal error: {msg} (please report this bug)")));
        #[cfg(test)]
        reading[0].fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        // The app may have quit meanwhile: nothing to hand it to.
        let _ = tx.send(r);
    });
    // A reader that has ended gives the job back.
    let job = match &app.placing.reader {
        Some(reader) => reader.send(job).err().map(|e| e.0),
        None => Some(job),
    };
    if let Some(job) = job {
        let (reader, jobs) = channel::<Job>();
        let started = std::thread::Builder::new().name("vectorcraft-place".into()).spawn(move || {
            for job in jobs {
                job();
            }
        });
        if started.is_err() {
            return Ok(None);
        }
        let _ = reader.send(job);
        app.placing.reader = Some(reader);
    }
    app.placing.pending.push(Pending { name, label: label.clone(), doc, dropped, paste: chain.is_some(), chain, done });
    let status = format!("{label}…");
    app.status(status.clone());
    Ok(Some(json!({ "background": true, "status": status })))
}

/// Place the files that are ready (each frame), in the order they were started: a file read
/// sooner than one started before it waits for that one. Not while a dialog is open, an
/// interaction is open, or a tool is busy with a drag of its own (the place cursor's) while the
/// pointer button is held, as [`crate::opening::poll`]: a placed file's document becomes the
/// active one and the dialog's OK acts on the active document; a drag in another document is
/// canceled when the active document changes, and a drag or preview in the same one previews from
/// the document as it was before it began, without the placed art. A tool that is busy while the
/// button is not held is waiting between its clicks (the Dimension tool after its first point):
/// the file is placed unless the tool has a preview open (the Flare tool after its center drag).
pub fn poll(app: &mut VectorcraftApp) {
    while app.ui.dialog.is_none()
        && !app.session.in_interaction()
        && !(app.session.tool_busy() && app.session.pointer_held())
        && let Some(p) = app.placing.pending.first()
    {
        let r = match p.done.try_recv() {
            Ok(r) => r,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("the file stopped being read (please report this bug)".to_string()),
        };
        let p = app.placing.pending.remove(0);
        // The status bar no longer says it is placing.
        if app.ui.status == format!("{}…", p.label) {
            app.ui.status.clear();
        }
        finish(app, &p, r);
        // A chain none of whose pastes waits any more is dropped, whether its pastes ran or not.
        if let Some(chain) = p.chain
            && let Some(ch) = app.placing.chains.get_mut(&chain)
        {
            ch.waiting = ch.waiting.saturating_sub(1);
            if ch.waiting == 0 {
                app.placing.chains.remove(&chain);
                app.placing.latest_chain.retain(|_, c| *c != chain);
            }
        }
    }
}

/// Place files read off the UI thread in their document, which becomes the active one again, as
/// [`crate::place::run`] and [`crate::place::queue`] place files read at once; an error shows in
/// the status bar as it does then.
fn finish(app: &mut VectorcraftApp, p: &Pending, r: Result<Read, String>) {
    let Some(i) = app.session.documents().iter().position(|st| st.uid == p.doc) else {
        let name = &p.name;
        if p.paste {
            app.status("Couldn't paste: the document it was pasted in was closed");
        } else if p.dropped {
            app.status(format!("Couldn't place {name}: the document it was dropped on was closed"));
        } else {
            app.status(format!("Couldn't place {name}: the document it was placed in was closed"));
        }
        return;
    };
    app.session.set_active(i);
    let r = match r {
        Ok(Read::Place(read)) => app.session.place_read(*read).map(|_| ()).map_err(|e| e.to_string()),
        Ok(Read::Queue(read)) => app.session.place_queue_read(read).map(|r| crate::place::show_thumbnails(app, &r)).map_err(|e| e.to_string()),
        Ok(Read::Paste(paste)) => return finish_paste(app, *paste),
        Err(e) if p.paste => return app.status(format!("Couldn't paste from the clipboard: {e}")),
        Err(e) => Err(e),
    };
    app.sync_views();
    let name = &p.name;
    match r {
        Err(e) if p.dropped => app.status(format!("Couldn't place {name}: {e}")),
        Err(e) => app.status(e),
        Ok(()) => {}
    }
}

/// Run a paste whose bitmap was decoded off the UI thread, in its document, which is the active one:
/// the image replaces the internal clipboard, the objects selected when the paste started are
/// selected again, and the paste command runs as it runs at once ([`VectorcraftApp::run`]), into
/// the layer that was current when it started, with its errors in the status bar. A Copy or Cut
/// made since the paste started keeps the internal clipboard it made.
fn finish_paste(app: &mut VectorcraftApp, paste: Paste) {
    let kept = (app.clipboard_publishing.latest() != paste.copied).then(|| app.session.clipboard.clone());
    let adopted = match paste.pasted {
        Some(Pasted::Image(mime, read)) => app.adopt_image_read(mime, paste.center, read),
        Some(Pasted::Flavour(f)) => app.adopt_flavour(&f, paste.center),
        None => {
            app.session.clipboard = paste.clipboard;
            Ok(())
        }
    };
    let chain = paste.chain;
    let selection = paste.selection.or_else(|| app.placing.chains.get_mut(&chain).and_then(|ch| ch.left.take()));
    let left = match adopted {
        Err(e) => {
            app.status(e);
            selection
        }
        Ok(()) => {
            // Where new art goes now, before a selection changes the current layer.
            let current = Insertion::of(app);
            // The objects selected when the paste started (or that the paste it follows left) are
            // selected again (those that are left), as `Session::place_read` selects the object
            // Replace swaps.
            let now = app.session.doc().map(|st| st.selection.objects.clone()).unwrap_or_default();
            if let Some(selection) = selection.filter(|s| *s != now) {
                let left: Vec<u64> =
                    selection.iter().filter(|id| app.session.doc().is_ok_and(|st| st.doc.node(**id).is_some())).map(|id| id.0).collect();
                let _ = app.session.execute("select.set", &json!({ "ids": left }));
            }
            // The paste goes where new art went when it started (the current layer, the isolated
            // group, the object Draw Inside draws into), and what the user changed meanwhile is
            // kept.
            paste.insertion.clone().set(app);
            app.placing.adopted = true;
            // `run` shows an error in the status bar.
            let _ = app.run(&paste.id, paste.params);
            // Also when the paste did not reach the clipboard (the Type tool took it).
            app.placing.adopted = false;
            if let Some(current) = current {
                current.restore_changed(&paste.insertion, app);
            }
            app.session.doc().ok().map(|st| st.selection.objects.clone())
        }
    };
    // The chain's next paste takes the selection this one left, and a paste given with it and the
    // insertion context joins the chain while a paste of it waits ([`poll`] drops the chain).
    let context = app.session.doc().ok().map(|st| st.selection.objects.clone()).zip(Insertion::of(app));
    if let Some(ch) = app.placing.chains.get_mut(&chain) {
        ch.left = left;
        if let Some(context) = context {
            ch.context = context;
        }
    }
    if let Some(kept) = kept {
        app.session.clipboard = kept;
    }
}

#[cfg(test)]
mod tests {
    use image::ImageFormat::{Bmp, Png, Tiff};
    use vectorcraft_doc::{Document, NodeId};
    use vectorcraft_engine::Session;
    use vectorcraft_engine::cmd::clipboard::{BITMAP, Flavour, PNG, TEXT};

    use super::*;
    use crate::Services;
    use crate::place::{DropAt, DropTarget};

    /// An app reading files from disk, with a 400 × 300 pt document, that reads the files the user
    /// places off the UI thread when `background`.
    fn app(background: bool) -> VectorcraftApp {
        let services = Services { read: Some(Box::new(|p: &str| std::fs::read(p).map_err(|e| e.to_string()))), ..Default::default() };
        let mut app = VectorcraftApp::new(Session::new(), services);
        app.placing.off_the_ui_thread = background;
        app.run("file.new", json!({"width": 400, "height": 300})).unwrap();
        app
    }

    /// `bytes` written to a fresh file named `name` in a folder of `test`'s own → its path.
    fn saved(test: &str, name: &str, bytes: &[u8]) -> String {
        let dir = std::env::temp_dir().join(format!("vc-placing-{test}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path.to_string_lossy().into_owned()
    }

    /// A `w` × `h` picture of one color in `format`.
    fn picture(format: image::ImageFormat, w: u32, h: u32) -> Vec<u8> {
        let mut out = vec![];
        image::RgbImage::from_pixel(w, h, image::Rgb([200, 40, 40])).write_to(&mut std::io::Cursor::new(&mut out), format).unwrap();
        out
    }

    /// A flat `w` × `h` Photoshop document of one color (raw RGB, no layers).
    fn psd(w: u32, h: u32) -> Vec<u8> {
        let mut f = b"8BPS\0\x01\0\0\0\0\0\0\0\x03".to_vec();
        f.extend(h.to_be_bytes());
        f.extend(w.to_be_bytes());
        f.extend([0, 8, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        for v in [40u8, 40, 200] {
            f.extend(std::iter::repeat_n(v, (w * h) as usize));
        }
        f
    }

    /// A one-page PDF with a rectangle on it.
    fn pdf() -> Vec<u8> {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 100, "height": 50})).unwrap();
        s.execute("shape.rectangle", &json!({"x": 5, "y": 5, "width": 20, "height": 10})).unwrap();
        let v = s.execute("document.serialize", &json!({"format": "pdf"})).unwrap();
        vectorcraft_format::base64_decode(v["dataBase64"].as_str().unwrap()).unwrap()
    }

    /// Poll each frame until every file read for Place has been placed (at most 60 s).
    fn placed(app: &mut VectorcraftApp) {
        let t0 = std::time::Instant::now();
        while !app.placing.pending.is_empty() && t0.elapsed().as_secs() < 60 {
            poll(app);
            std::thread::yield_now();
        }
        assert!(app.placing.pending.is_empty(), "read within 60 s");
    }

    /// What a place changes: the active document (without the second it was made in), its
    /// selection, its undo steps and the last journal entry.
    fn state(app: &VectorcraftApp) -> (Document, Vec<NodeId>, usize, Option<(String, Value)>) {
        let st = app.session.doc().unwrap();
        let mut doc = (*st.doc).clone();
        doc.metadata.created = None;
        (doc, st.selection.objects.clone(), st.history.undo.len(), app.session.journal.last().cloned())
    }

    /// The names of the active document's top-level objects, bottom first.
    fn names(app: &VectorcraftApp) -> Vec<String> {
        let st = app.session.doc().unwrap();
        st.doc.layers.iter().flat_map(|l| l.children().into_iter().flatten()).filter_map(|n| n.name.clone()).collect()
    }

    /// A TIFF, a Photoshop document, a BMP, a PDF page and a PDF-compatible `.ai` page (both placed
    /// linked) read off the UI thread are placed as when they are read at once: the same object
    /// with its bounds and link, the same selection, one undo step and the same journal entry. Until
    /// then the document is unchanged, and the status bar and the busy cursor show the file is being
    /// placed.
    #[test]
    fn a_file_read_off_the_ui_thread_places_as_one_read_at_once() {
        let page = pdf();
        let files = [
            ("photo.tif", picture(Tiff, 300, 200)),
            ("photo.psd", psd(300, 200)),
            ("photo.bmp", picture(Bmp, 300, 200)),
            ("page.pdf", page.clone()),
            ("page.ai", page),
        ];
        for (name, bytes) in files {
            let p = json!({"path": saved("same", name, &bytes), "at": [200, 150]});
            let mut at_once = app(false);
            at_once.run("file.place", p.clone()).unwrap();
            let mut app = app(true);
            let before = state(&app);
            let r = start(&mut app, &p, false).unwrap();
            assert_eq!(r, json!({"background": true, "status": format!("Placing {name}…")}));
            assert_eq!(app.ui.status, format!("Placing {name}…"));
            assert!(crate::opening::long_operation_busy(&app), "{name}");
            assert!(state(&app) == before, "{name}: nothing is placed until the file is read");
            placed(&mut app);
            assert!(state(&app) == state(&at_once), "{name}: placed as at once");
            assert!(app.ui.status.is_empty(), "{name}: {}", app.ui.status);
            assert!(!crate::opening::long_operation_busy(&app), "{name}");
        }
    }

    /// Another app's bitmap on the system clipboard, and the text beside it, read as the desktop
    /// hosts read them: with `converts`, the bitmap as the work that makes its flavour
    /// ([`crate::ClipboardRead::Later`], `None`: it can't be made), which notes the thread it runs
    /// on in `thread`. The work of each read waits for its entry of `holds` to send, when it has
    /// one.
    struct OtherApps {
        bitmap: Option<Flavour>,
        text: Option<Flavour>,
        converts: bool,
        thread: Arc<Mutex<Option<String>>>,
        holds: Vec<Option<std::sync::mpsc::Receiver<()>>>,
    }

    impl crate::SystemClipboard for OtherApps {
        fn write(&mut self, _: &[Flavour]) -> Result<(), String> {
            Ok(())
        }
        fn holds_ours(&mut self) -> bool {
            false
        }
        fn read(&mut self, mimes: &[&'static str]) -> Option<Flavour> {
            self.read_deferred(mimes)?.flavour()
        }
        fn has(&mut self, mimes: &[&'static str]) -> bool {
            mimes.contains(&BITMAP) || (mimes.contains(&TEXT) && self.text.is_some())
        }
        fn read_deferred(&mut self, mimes: &[&'static str]) -> Option<crate::ClipboardRead> {
            if mimes.contains(&TEXT)
                && let Some(text) = &self.text
            {
                return Some(crate::ClipboardRead::Now(text.clone()));
            }
            if !mimes.contains(&BITMAP) {
                return None;
            }
            let (bitmap, thread) = (self.bitmap.clone(), self.thread.clone());
            if !self.converts {
                return bitmap.map(crate::ClipboardRead::Now);
            }
            let hold = if self.holds.is_empty() { None } else { self.holds.remove(0) };
            Some(crate::ClipboardRead::Later(Box::new(move || {
                if let Some(hold) = hold {
                    let _ = hold.recv_timeout(std::time::Duration::from_secs(10));
                }
                *thread.lock().unwrap() = std::thread::current().name().map(str::to_string);
                bitmap
            })))
        }
    }

    /// [`app`] with `bitmap` and `text` on the system clipboard ([`OtherApps`]), the view centered
    /// on (250, 180) → the app and the thread the bitmap was converted on.
    fn pasting_app(background: bool, bitmap: Option<Flavour>, text: Option<&str>, converts: bool) -> (VectorcraftApp, Arc<Mutex<Option<String>>>) {
        let thread = Arc::new(Mutex::new(None));
        let mut app = app(background);
        let text = text.map(|t| Flavour { mime: TEXT, data: t.as_bytes().to_vec() });
        app.services.system_clipboard = Some(Box::new(OtherApps { bitmap, text, converts, thread: thread.clone(), holds: vec![] }));
        let v = app.view_mut().unwrap();
        (v.center, v.fitted) = (vectorcraft_geom::Point::new(250.0, 180.0), true);
        (app, thread)
    }

    /// A Paste the user gives decodes another app's bitmap off the UI thread: a BMP as
    /// `clipboard.importImage` reads it, and a bitmap the host converts to PNG (Windows'
    /// device-independent bitmap, Linux's image) converted there too. Until then the document is
    /// unchanged, and the status bar and the busy cursor show the paste; then it is pasted as at
    /// once, where the view was centered when the paste was given: the same object, selection, undo
    /// step and internal clipboard. `edit.paste` from MCP and scripts reads it at once.
    #[test]
    fn a_bitmap_the_user_pastes_is_read_off_the_ui_thread() {
        let bmp = Flavour { mime: "image/bmp", data: picture(Bmp, 300, 200) };
        let png = Flavour { mime: PNG, data: picture(Png, 300, 200) };
        for (bitmap, converts) in [(bmp, false), (png, true)] {
            for id in ["edit.paste", "edit.pasteInPlace"] {
                let (mut at_once, _) = pasting_app(false, Some(bitmap.clone()), None, converts);
                crate::menus::invoke(&mut at_once, id, json!({}));
                let (mut app, _) = pasting_app(true, Some(bitmap.clone()), None, converts);
                crate::menus::invoke(&mut app, id, json!({}));
                // The view moves before the bitmap is read.
                app.view_mut().unwrap().center = vectorcraft_geom::Point::new(40.0, 30.0);
                placed(&mut app);
                assert!(state(&app) == state(&at_once), "{} {id}: pasted where the view was", bitmap.mime);
            }
            let (mut at_once, _) = pasting_app(false, Some(bitmap.clone()), None, converts);
            crate::menus::invoke(&mut at_once, "edit.paste", json!({}));
            let (mut app, thread) = pasting_app(true, Some(bitmap.clone()), None, converts);
            let before = state(&app);
            app.placing.for_user = true;
            let r = app.run("edit.paste", json!({}));
            app.placing.for_user = false;
            assert_eq!(r, Ok(json!({"background": true, "status": "Pasting…"})), "{}", bitmap.mime);
            assert_eq!(app.ui.status, "Pasting…");
            assert!(crate::opening::long_operation_busy(&app));
            assert!(state(&app) == before, "{}: nothing is pasted until the bitmap is read", bitmap.mime);
            placed(&mut app);
            assert!(state(&app) == state(&at_once), "{}: pasted as at once", bitmap.mime);
            assert!(app.session.clipboard == at_once.session.clipboard, "{}", bitmap.mime);
            assert!(app.ui.status.is_empty(), "{}", app.ui.status);
            if converts {
                assert_eq!(thread.lock().unwrap().as_deref(), Some("vectorcraft-place"));
            }
            // From the menu or the keys too.
            crate::menus::invoke(&mut app, "edit.paste", json!({}));
            assert_eq!(app.placing.pending.len(), 1);
            placed(&mut app);
            // MCP and scripts: at once.
            let r = app.run("edit.paste", json!({})).unwrap();
            assert!(r["ids"].is_array() && app.placing.pending.is_empty(), "{r}");
            assert_eq!(app.session.doc().unwrap().selection.objects.len(), 1);
        }
    }

    /// A bitmap that can't be read reports as at once and pastes nothing; one the host can't
    /// convert pastes the internal clipboard as it is, or the address beside it as text, as at
    /// once.
    #[test]
    fn a_pasted_bitmap_that_cant_be_read_off_the_ui_thread_reports_as_at_once() {
        let url = "https://example.com/images/cat.png";
        let cases = [(Some(Flavour { mime: PNG, data: b"not a picture".to_vec() }), None, false), (None, None, true), (None, Some(url), true)];
        for (bitmap, text, converts) in cases {
            let (mut at_once, _) = pasting_app(false, bitmap.clone(), text, converts);
            let (mut app, _) = pasting_app(true, bitmap, text, converts);
            for app in [&mut at_once, &mut app] {
                app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 20, "height": 20})).unwrap();
                app.run("edit.copy", json!({})).unwrap();
                crate::menus::invoke(app, "edit.paste", json!({}));
                assert_eq!(app.placing.pending.len(), usize::from(app.placing.off_the_ui_thread));
                placed(app);
            }
            assert_eq!(app.ui.status, at_once.ui.status);
            assert!(state(&app) == state(&at_once), "text: {text:?}, converts: {converts}");
            let st = app.session.doc().unwrap();
            let pasted = st.selection.objects.first().and_then(|id| st.doc.node(*id)).map(|n| &n.kind);
            match (text, converts) {
                (Some(_), _) => assert!(matches!(pasted, Some(vectorcraft_doc::NodeKind::Text(_))), "the address is pasted as text"),
                (None, true) => assert!(matches!(pasted, Some(vectorcraft_doc::NodeKind::Path { .. })), "the copied square is pasted"),
                (None, false) => assert!(app.ui.status.starts_with("Couldn't paste from the clipboard: "), "{}", app.ui.status),
            }
        }
    }

    /// A Copy made while a pasted bitmap is read keeps the internal clipboard it made, and a paste
    /// whose document was closed meanwhile pastes nothing, with a message.
    #[test]
    fn a_copy_or_a_close_while_a_pasted_bitmap_is_read() {
        let bitmap = Some(Flavour { mime: PNG, data: picture(Png, 60, 40) });
        let (mut app, _) = pasting_app(true, bitmap.clone(), None, true);
        crate::menus::invoke(&mut app, "edit.paste", json!({}));
        app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 20, "height": 20})).unwrap();
        app.run("edit.copy", json!({})).unwrap();
        let copied = app.session.clipboard.clone();
        placed(&mut app);
        let st = app.session.doc().unwrap();
        let pasted = st.doc.node(st.selection.objects[0]).unwrap();
        assert!(matches!(pasted.kind, vectorcraft_doc::NodeKind::Image(_)), "the bitmap is pasted");
        assert!(app.session.clipboard == copied, "the copy stays on the internal clipboard");
        // A bitmap that can't be converted pastes the internal clipboard as it was when the paste
        // was given, and the later copy stays on it.
        let (mut app, _) = pasting_app(true, None, None, true);
        let square = |app: &mut VectorcraftApp, x: u32| {
            app.run("shape.rectangle", json!({"x": x, "y": 10, "width": 20, "height": 20})).unwrap();
            app.run("edit.copy", json!({})).unwrap();
        };
        square(&mut app, 10);
        let first = app.session.clipboard.clone();
        crate::menus::invoke(&mut app, "edit.pasteInPlace", json!({}));
        square(&mut app, 200);
        let second = app.session.clipboard.clone();
        placed(&mut app);
        let st = app.session.doc().unwrap();
        let pasted = st.doc.node(st.selection.objects[0]).and_then(|n| n.geometric_bounds()).map(|b| b.x0);
        assert!(first != second && pasted == Some(10.0), "the first copy is pasted: {pasted:?}");
        assert!(app.session.clipboard == second, "the later copy stays on the internal clipboard");
        let (mut app, _) = pasting_app(true, bitmap, None, true);
        app.run("file.new", json!({"width": 100, "height": 100})).unwrap();
        crate::menus::invoke(&mut app, "edit.paste", json!({}));
        app.session.execute("file.close", &json!({})).unwrap();
        let left = state(&app);
        placed(&mut app);
        assert_eq!(app.ui.status, "Couldn't paste: the document it was pasted in was closed");
        assert!(state(&app) == left, "the other document is unchanged");
        assert!(app.placing.chains.is_empty() && app.placing.latest_chain.is_empty(), "no paste waits");
    }

    /// Paste in Front and Paste in Back put a bitmap decoded off the UI thread next to the objects
    /// selected when the paste was given, as at once, when another object was selected meanwhile;
    /// a second one given before the first is pasted goes next to the first, as at once.
    #[test]
    fn a_paste_in_front_read_off_the_ui_thread_keeps_its_place() {
        let bitmap = Some(Flavour { mime: PNG, data: picture(Png, 60, 40) });
        for id in ["edit.pasteInFront", "edit.pasteInBack"] {
            for twice in [false, true] {
                let (mut at_once, _) = pasting_app(false, bitmap.clone(), None, true);
                let (mut app, _) = pasting_app(true, bitmap.clone(), None, true);
                for app in [&mut at_once, &mut app] {
                    let rect = |app: &mut VectorcraftApp, x: u32| {
                        app.run("shape.rectangle", json!({"x": x, "y": 10, "width": 20, "height": 20})).unwrap()["id"].clone()
                    };
                    let (a, b, c) = (rect(app, 10), rect(app, 40), rect(app, 70));
                    app.run("select.set", json!({"ids": [b]})).unwrap();
                    crate::menus::invoke(app, id, json!({}));
                    if twice {
                        crate::menus::invoke(app, id, json!({}));
                    } else {
                        let other = if id.ends_with("Front") { a } else { c };
                        app.run("select.set", json!({"ids": [other]})).unwrap();
                    }
                    placed(app);
                }
                // Each object in paint order: `None` for a square, the paste order for a picture.
                let order = |app: &VectorcraftApp| {
                    let st = app.session.doc().unwrap();
                    let children = st.doc.layers[0].children().unwrap();
                    let mut pictures: Vec<NodeId> =
                        children.iter().filter(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Image(_))).map(|n| n.id).collect();
                    pictures.sort();
                    children.iter().map(|n| pictures.iter().position(|p| *p == n.id)).collect::<Vec<_>>()
                };
                assert_eq!(order(&app), order(&at_once), "{id}, twice: {twice}");
            }
        }
    }

    /// A second Paste in Front given before the first is pasted goes next to the first one's
    /// picture, also when another object was selected between the two pastes' ends, and a third
    /// given after the first was pasted, while the second waits, goes next to the second's.
    #[test]
    fn a_second_paste_in_front_follows_the_first() {
        for third in [false, true] {
            let (mut app, _) = pasting_app(true, None, None, true);
            let (go, hold) = channel();
            let thread = Arc::new(Mutex::new(None));
            let bitmap = Some(Flavour { mime: PNG, data: picture(Png, 60, 40) });
            app.services.system_clipboard = Some(Box::new(OtherApps { bitmap, text: None, converts: true, thread, holds: vec![None, Some(hold)] }));
            let rect = |app: &mut VectorcraftApp, x: u32| {
                app.run("shape.rectangle", json!({"x": x, "y": 10, "width": 20, "height": 20})).unwrap()["id"].clone()
            };
            let (a, b) = (rect(&mut app, 10), rect(&mut app, 40));
            app.run("select.set", json!({"ids": [b]})).unwrap();
            crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
            crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
            let t0 = std::time::Instant::now();
            while app.placing.pending.len() > 1 && t0.elapsed().as_secs() < 60 {
                poll(&mut app);
                std::thread::yield_now();
            }
            assert_eq!(app.placing.pending.len(), 1, "the first paste ran");
            if third {
                crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
            } else {
                app.run("select.set", json!({"ids": [a]})).unwrap();
            }
            let _ = go.send(());
            placed(&mut app);
            let st = app.session.doc().unwrap();
            let kinds: Vec<Option<NodeId>> =
                st.doc.layers[0].children().unwrap().iter().map(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Image(_)).then_some(n.id)).collect();
            let pictures: Vec<NodeId> = kinds.iter().flatten().copied().collect();
            assert!(pictures.len() == 2 + usize::from(third) && pictures.is_sorted(), "each picture is above the one before: {kinds:?}");
            assert_eq!(kinds.iter().position(Option::is_some), Some(2), "the pictures are above the square they were pasted on: {kinds:?}");
        }
    }

    /// A paste given beside the picture an independent paste left, while another paste waits, goes
    /// beside that picture.
    #[test]
    fn a_paste_beside_a_finished_picture_does_not_follow_another_chain() {
        let (mut app, _) = pasting_app(true, None, None, true);
        let (go, hold) = channel();
        let bitmap = Some(Flavour { mime: PNG, data: picture(Png, 60, 40) });
        let clip = OtherApps { bitmap, text: None, converts: true, thread: Arc::new(Mutex::new(None)), holds: vec![None, Some(hold)] };
        app.services.system_clipboard = Some(Box::new(clip));
        let rect =
            |app: &mut VectorcraftApp, x: u32| app.run("shape.rectangle", json!({"x": x, "y": 10, "width": 20, "height": 20})).unwrap()["id"].clone();
        let (x, y) = (rect(&mut app, 10), rect(&mut app, 40));
        app.run("select.set", json!({"ids": [x]})).unwrap();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        app.run("select.set", json!({"ids": [y]})).unwrap();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        let t0 = std::time::Instant::now();
        while app.placing.pending.len() > 1 && t0.elapsed().as_secs() < 60 {
            poll(&mut app);
            std::thread::yield_now();
        }
        assert_eq!(app.placing.pending.len(), 1, "the first paste ran");
        // The first picture is selected.
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        let _ = go.send(());
        placed(&mut app);
        let st = app.session.doc().unwrap();
        let ids: Vec<Option<NodeId>> =
            st.doc.layers[0].children().unwrap().iter().map(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Image(_)).then_some(n.id)).collect();
        let last = ids.iter().flatten().max().copied();
        assert_eq!(ids.iter().position(|id| *id == last && last.is_some()), Some(2), "the third picture is above the first: {ids:?}");
    }

    /// Pastes that follow each other in one document follow each other there, also when a paste in
    /// another document runs between them.
    #[test]
    fn pastes_in_two_documents_follow_their_own() {
        let (mut app, _) = pasting_app(true, None, None, true);
        let (go, hold) = channel();
        let bitmap = Some(Flavour { mime: PNG, data: picture(Png, 60, 40) });
        let clip = OtherApps { bitmap, text: None, converts: true, thread: Arc::new(Mutex::new(None)), holds: vec![None, Some(hold)] };
        app.services.system_clipboard = Some(Box::new(clip));
        let rect =
            |app: &mut VectorcraftApp, x: u32| app.run("shape.rectangle", json!({"x": x, "y": 10, "width": 20, "height": 20})).unwrap()["id"].clone();
        let (_, b) = (rect(&mut app, 10), rect(&mut app, 40));
        app.run("select.set", json!({"ids": [b]})).unwrap();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        let t0 = std::time::Instant::now();
        while app.placing.pending.len() > 1 && t0.elapsed().as_secs() < 60 {
            poll(&mut app);
            std::thread::yield_now();
        }
        assert_eq!(app.placing.pending.len(), 1, "the first paste ran");
        // A paste in another document, then one more in the first.
        app.run("file.new", json!({"width": 400, "height": 300})).unwrap();
        let x = rect(&mut app, 10);
        app.run("select.set", json!({"ids": [x]})).unwrap();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        app.session.set_active(0);
        app.sync_views();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        let _ = go.send(());
        placed(&mut app);
        let st = &app.session.documents()[0];
        let kinds: Vec<Option<NodeId>> =
            st.doc.layers[0].children().unwrap().iter().map(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Image(_)).then_some(n.id)).collect();
        let pictures: Vec<NodeId> = kinds.iter().flatten().copied().collect();
        assert!(pictures.len() == 3 && pictures.is_sorted(), "each picture is above the one before: {kinds:?}");
        assert_eq!(kinds.iter().position(Option::is_some), Some(2), "the pictures are above the square they were pasted on: {kinds:?}");
    }

    /// In Draw Inside in an isolated path, a bitmap decoded off the UI thread is pasted inside the path,
    /// whose new clipping group stays isolated, when the user went back to Draw Normal meanwhile.
    #[test]
    fn a_bitmap_pasted_inside_an_isolated_path_stays_isolated() {
        let (mut app, _) = pasting_app(true, Some(Flavour { mime: PNG, data: picture(Png, 60, 40) }), None, true);
        let path = app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 200, "height": 200})).unwrap()["id"].clone();
        app.run("object.isolate", json!({"id": path})).unwrap();
        app.run("select.set", json!({"ids": [path]})).unwrap();
        app.run("view.drawMode", json!({"mode": "inside"})).unwrap();
        crate::menus::invoke(&mut app, "edit.paste", json!({}));
        app.run("view.drawMode", json!({"mode": "normal"})).unwrap();
        placed(&mut app);
        let st = app.session.doc().unwrap();
        let pasted = st.selection.objects[0];
        let group = st.doc.parent_of(pasted);
        assert!(
            group.is_some_and(|g| st.doc.node(g).is_some_and(|n| n.children().is_some_and(|c| c.iter().any(|n| n.id.0 == path.as_u64().unwrap())))),
            "pasted inside the path"
        );
        assert_eq!(st.isolation, group, "the clipping group is isolated");
        assert_eq!(app.session.draw_mode, vectorcraft_engine::DrawMode::Normal, "Draw Normal stays");
    }

    /// A bitmap decoded off the UI thread is pasted into the layer that was current when the paste
    /// was given, and a layer made current meanwhile stays current.
    #[test]
    fn a_bitmap_read_off_the_ui_thread_is_pasted_into_its_layer() {
        let (mut app, _) = pasting_app(true, Some(Flavour { mime: PNG, data: picture(Png, 60, 40) }), None, true);
        let first = app.session.doc().unwrap().doc.layers[0].id;
        app.run("layer.new", json!({})).unwrap();
        let second = app.session.doc().unwrap().current_layer().unwrap();
        app.run("layer.setCurrent", json!({"id": first.0})).unwrap();
        crate::menus::invoke(&mut app, "edit.paste", json!({}));
        app.run("layer.setCurrent", json!({"id": second.0})).unwrap();
        placed(&mut app);
        let st = app.session.doc().unwrap();
        let pasted = st.selection.objects[0];
        assert_eq!(st.doc.layer_containing(pasted), Some(first), "pasted into the layer current at the paste");
        assert_eq!(st.current_layer(), Some(second), "the layer made current meanwhile stays current");
        // With nothing selected, a Paste in Front given in another layer while the first waits
        // goes into that layer.
        app.run("layer.setCurrent", json!({"id": first.0})).unwrap();
        app.run("select.none", json!({})).unwrap();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        app.run("layer.setCurrent", json!({"id": second.0})).unwrap();
        app.run("select.none", json!({})).unwrap();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        assert_eq!(app.placing.pending.len(), 2);
        placed(&mut app);
        let st = app.session.doc().unwrap();
        let tops: Vec<_> = [first, second]
            .iter()
            .map(|l| st.doc.node(*l).and_then(|n| n.children()).and_then(|c| c.last()).map(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Image(_))))
            .collect();
        assert_eq!(tops, [Some(true), Some(true)], "each layer has a picture on top");
        // A group isolated while the paste waits keeps its isolation, and the paste goes into the
        // layer, where it was given.
        let squares: Vec<Value> = (0..2)
            .map(|i| app.run("shape.rectangle", json!({"x": 10 + 30 * i, "y": 10, "width": 20, "height": 20})).unwrap()["id"].clone())
            .collect();
        app.run("select.set", json!({"ids": squares})).unwrap();
        let group = NodeId(app.run("object.group", json!({})).unwrap()["id"].as_u64().unwrap());
        app.run("select.none", json!({})).unwrap();
        crate::menus::invoke(&mut app, "edit.paste", json!({}));
        app.run("object.isolate", json!({"id": group.0})).unwrap();
        placed(&mut app);
        let st = app.session.doc().unwrap();
        let pasted = st.selection.objects[0];
        assert_eq!(st.doc.parent_of(pasted), Some(second), "pasted into the layer, not the group isolated meanwhile");
        assert_eq!(st.isolation, Some(group), "the group stays isolated");
        // An object selected in another layer while the paste waits makes that layer current, and
        // it stays current after the paste.
        app.run("object.exitIsolation", json!({})).unwrap();
        let in_second = app.run("shape.rectangle", json!({"x": 300, "y": 10, "width": 20, "height": 20})).unwrap()["id"].clone();
        app.run("layer.setCurrent", json!({"id": first.0})).unwrap();
        let in_first = app.run("shape.rectangle", json!({"x": 300, "y": 100, "width": 20, "height": 20})).unwrap()["id"].clone();
        app.run("select.set", json!({"ids": [in_first]})).unwrap();
        crate::menus::invoke(&mut app, "edit.pasteInFront", json!({}));
        app.run("select.set", json!({"ids": [in_second]})).unwrap();
        assert_eq!(app.session.doc().unwrap().current_layer(), Some(second));
        placed(&mut app);
        let st = app.session.doc().unwrap();
        assert_eq!(st.doc.layer_containing(st.selection.objects[0]), Some(first), "pasted next to the object selected at the paste");
        assert_eq!(st.current_layer(), Some(second), "the layer of the object selected meanwhile stays current");
    }

    /// The Place dialog's OK with one file, with several (the place cursor and its thumbnails), a
    /// drop on the canvas and [`crate::io::place_bytes`] read their files off the UI thread;
    /// `file.place` and `file.place.queue` from MCP and scripts read them at once, with the replies
    /// they have always had.
    #[test]
    fn what_the_user_places_is_read_off_the_ui_thread_and_commands_read_at_once() {
        let mut app = app(true);
        let (a, b) = (saved("routes", "a.tif", &picture(Tiff, 60, 40)), saved("routes", "b.bmp", &picture(Bmp, 60, 40)));
        let r = app.run("file.place", json!({"path": a})).unwrap();
        assert!(r["ids"].is_array() && app.placing.pending.is_empty(), "{r}");
        let r = app.run("file.place.queue", json!({"paths": [b]})).unwrap();
        assert!(r["count"] == 1 && app.placing.pending.is_empty(), "{r}");
        assert_eq!(app.place.thumbnails(), ["b.bmp"]);
        app.select_tool("selection");
        // The Place dialog's OK with one file.
        app.run("file.place", json!({"paths": [a]})).unwrap();
        assert_eq!(crate::dialogs::confirm(&mut app).unwrap()["background"], true);
        assert_eq!(app.placing.pending.len(), 1);
        placed(&mut app);
        assert_eq!(names(&app), ["a.tif", "a.tif"]);
        // With several: the place cursor is loaded once they are read.
        app.run("file.place", json!({"paths": [a, b]})).unwrap();
        assert_eq!(crate::dialogs::confirm(&mut app).unwrap()["background"], true);
        assert_eq!(app.ui.status, "Placing a.tif, b.bmp…");
        assert_eq!(app.session.tool_id(), "selection");
        let (inspect, _) = crate::control::ControlRequest::new("ui.inspect", json!({}));
        let crate::control::Outcome::Done(r) = crate::control::handle(&mut app, &egui::Context::default(), &inspect) else { panic!("not done") };
        assert_eq!(r["result"]["placing"], json!(["a.tif, b.bmp"]), "ui.inspect lists the files being read");
        placed(&mut app);
        assert_eq!((app.session.tool_id(), app.session.tool_options()["count"].as_u64()), ("place", Some(2)));
        assert_eq!(app.place.thumbnails(), ["a.tif", "b.bmp"]);
        app.select_tool("selection");
        // A drop and bytes placed without a path.
        let doc = app.session.doc().unwrap().uid;
        let drop = DropTarget::Place(DropAt { doc, at: vectorcraft_geom::Point::new(100.0, 100.0), embed: false });
        crate::place::drop_files(&mut app, vec![(drop, ("b.bmp".into(), Some(b.clone()), vec![]))]);
        crate::io::place_bytes(&mut app, "c.png", &picture(Png, 10, 10)).unwrap();
        assert_eq!(app.placing.pending.len(), 2);
        placed(&mut app);
        assert_eq!(names(&app), ["a.tif", "a.tif", "b.bmp", "c.png"]);
    }

    /// A click and a drag of the place cursor read their files off the UI thread, and place them as
    /// a click and a drag place them when they are read at once.
    #[test]
    fn the_place_cursors_clicks_read_their_files_off_the_ui_thread() {
        use vectorcraft_tools::{PointerEvent, PointerKind};
        let paths = [saved("cursor", "a.tif", &picture(Tiff, 60, 40)), saved("cursor", "b.bmp", &picture(Bmp, 60, 40))];
        let view = vectorcraft_engine::ViewInfo::default();
        let press = |app: &mut VectorcraftApp, from: (f64, f64), to: (f64, f64)| {
            for (kind, (x, y)) in [(PointerKind::Down, from), (PointerKind::Drag, to), (PointerKind::Up, to)] {
                crate::canvas::dispatch(app, &PointerEvent::new(kind, x, y), view);
            }
        };
        let (mut at_once, mut app) = (app(false), app(true));
        for app in [&mut at_once, &mut app] {
            app.run("file.place.queue", json!({"paths": paths})).unwrap();
            press(app, (10.0, 10.0), (10.0, 10.0));
        }
        assert_eq!(names(&at_once), ["a.tif"]);
        assert!(names(&app).is_empty() && app.placing.pending.len() == 1, "nothing is placed until the file is read");
        assert_eq!(app.ui.status, "Placing a.tif…");
        placed(&mut app);
        assert!(state(&app) == state(&at_once), "the click places as at once");
        // A drag places the last file at the dragged size, and the previous tool comes back.
        for app in [&mut at_once, &mut app] {
            press(app, (100.0, 100.0), (180.0, 110.0));
        }
        placed(&mut app);
        assert!(state(&app) == state(&at_once), "the drag places as at once");
        assert_eq!(names(&app), ["a.tif", "b.bmp"]);
        assert_eq!(app.session.tool_id(), at_once.session.tool_id());
    }

    /// The Place dialog's line about each file is read off the UI thread: the lines are empty when
    /// the dialog opens, and then show what the lines read at once show.
    #[test]
    fn the_place_dialogs_lines_are_read_off_the_ui_thread() {
        let p = json!({"paths": [saved("lines", "a.tif", &picture(Tiff, 60, 40)), saved("lines", "notes.xyz", b"hello")]});
        let mut at_once = app(false);
        at_once.run("file.place", p.clone()).unwrap();
        let want = at_once.ui.dialog.as_ref().unwrap().fields["__info"].clone();
        assert!(want[0].as_str().is_some_and(|l| l.starts_with("60 × 40 px")), "{want}");
        let mut app = app(true);
        app.run("file.place", p).unwrap();
        assert_eq!(app.ui.dialog.as_ref().unwrap().fields["__info"], json!(["", ""]));
        let t0 = std::time::Instant::now();
        loop {
            let Some(mut d) = app.ui.dialog.take() else { panic!("the dialog stays open") };
            let reading = crate::place::fill_lines(&mut app, &mut d);
            app.ui.dialog = Some(d);
            if !reading || t0.elapsed().as_secs() > 60 {
                break;
            }
            std::thread::yield_now();
        }
        assert_eq!(app.ui.dialog.as_ref().unwrap().fields["__info"], want);
    }

    /// Several files dropped together are placed in the order they were dropped, a large one first
    /// included, as when they are read at once.
    #[test]
    fn files_dropped_together_are_placed_in_the_order_dropped() {
        let large = saved("order", "large.tif", &picture(Tiff, 1500, 1500));
        let small = [saved("order", "small1.png", &picture(Png, 8, 8)), saved("order", "small2.png", &picture(Png, 8, 8))];
        let mut app = app(true);
        let doc = app.session.doc().unwrap().uid;
        let drop = DropTarget::Place(DropAt { doc, at: vectorcraft_geom::Point::new(200.0, 150.0), embed: false });
        let files = [&large, &small[0], &small[1]].map(|p| (drop, (vectorcraft_engine::cmd::fileio::file_name(p), Some(p.clone()), vec![])));
        crate::place::drop_files(&mut app, files.into());
        assert_eq!(app.placing.pending.len(), 3);
        placed(&mut app);
        assert_eq!(names(&app), ["large.tif", "small1.png", "small2.png"]);
    }

    /// Files dropped together are read one at a time, as when they are read at once, so a batch of
    /// large pictures is not decoded all at once.
    #[test]
    fn files_dropped_together_are_read_one_at_a_time() {
        let files: Vec<String> = (0..4).map(|i| saved("one-at-a-time", &format!("large{i}.tif"), &picture(Tiff, 1500, 1500))).collect();
        let mut app = app(true);
        let doc = app.session.doc().unwrap().uid;
        let drop = DropTarget::Place(DropAt { doc, at: vectorcraft_geom::Point::new(200.0, 150.0), embed: false });
        let dropped = files.iter().map(|p| (drop, (vectorcraft_engine::cmd::fileio::file_name(p), Some(p.clone()), vec![]))).collect();
        crate::place::drop_files(&mut app, dropped);
        placed(&mut app);
        assert_eq!(names(&app).len(), 4);
        assert_eq!(app.placing.reading[1].load(std::sync::atomic::Ordering::SeqCst), 1, "the most reads at once");
    }

    /// A file that can't be read shows the message it shows when read at once, dropped or placed
    /// from the dialog, and nothing is placed.
    #[test]
    fn a_file_that_cant_be_read_off_the_ui_thread_reports_as_at_once() {
        let gone = saved("unread", "gone.tif", b"");
        std::fs::remove_file(&gone).unwrap();
        let unknown = saved("unread", "notes.xyz", b"hello");
        for path in [gone, unknown] {
            let p = json!({"path": path});
            let mut at_once = app(false);
            crate::place::run_for_user(&mut at_once, &p).unwrap_err();
            let mut app = app(true);
            let before = state(&app);
            crate::place::run_for_user(&mut app, &p).unwrap();
            placed(&mut app);
            assert_eq!(app.ui.status, at_once.ui.status);
            assert!(state(&app) == before, "{path}: nothing placed");
            // Dropped on the canvas.
            let name = vectorcraft_engine::cmd::fileio::file_name(&path);
            for app in [&mut at_once, &mut app] {
                let doc = app.session.doc().unwrap().uid;
                let drop = DropTarget::Place(DropAt { doc, at: vectorcraft_geom::Point::new(100.0, 100.0), embed: false });
                crate::place::drop_files(app, vec![(drop, (name.clone(), Some(path.clone()), vec![]))]);
                placed(app);
            }
            assert_eq!(app.ui.status, at_once.ui.status);
            assert!(app.ui.status.starts_with(&format!("Couldn't place {name}: ")), "{}", app.ui.status);
        }
    }

    /// A file whose document closed before it was read is not placed, and the status bar says why.
    #[test]
    fn a_file_whose_document_closed_meanwhile_is_not_placed() {
        let path = saved("closed", "photo.tif", &picture(Tiff, 60, 40));
        for dropped in [false, true] {
            let mut app = app(true);
            app.run("file.new", json!({"width": 100, "height": 100})).unwrap();
            start(&mut app, &json!({"path": path}), dropped).unwrap();
            app.session.execute("file.close", &json!({})).unwrap();
            let left = state(&app);
            placed(&mut app);
            let why = if dropped { "dropped on" } else { "placed in" };
            assert_eq!(app.ui.status, format!("Couldn't place photo.tif: the document it was {why} was closed"));
            assert!(state(&app) == left, "the other document is unchanged");
        }
    }

    /// Replace swaps the object selected when Place started; when that object was deleted before the
    /// file was read, nothing is placed and the status bar says why.
    #[test]
    fn a_replace_whose_object_was_deleted_meanwhile_places_nothing() {
        let path = saved("replace", "photo.tif", &picture(Tiff, 60, 40));
        let mut app = app(true);
        app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 20, "height": 20})).unwrap();
        crate::place::run_for_user(&mut app, &json!({"path": path, "replace": true})).unwrap();
        app.run("edit.clear", json!({})).unwrap();
        let cleared = state(&app);
        placed(&mut app);
        assert!(app.ui.status.contains("the object to replace was changed or deleted"), "{}", app.ui.status);
        assert!(state(&app) == cleared, "nothing placed");
    }

    /// A file read while a dialog is open waits for the dialog to close, as a document that opens
    /// does (#1694): the dialog's OK acts on the active document.
    #[test]
    fn a_file_read_while_a_dialog_is_open_waits_for_it_to_close() {
        let path = saved("dialog", "photo.tif", &picture(Tiff, 60, 40));
        let mut app = app(true);
        start(&mut app, &json!({"path": path}), false).unwrap();
        app.ui.dialog = Some(crate::state::Dialog::new(crate::dialogs::save_options::KIND, json!({"path": "/art/First.vectorcraft"})));
        let t0 = std::time::Instant::now();
        while t0.elapsed().as_millis() < 400 {
            poll(&mut app);
            std::thread::yield_now();
        }
        assert!(names(&app).is_empty() && app.placing.pending.len() == 1, "the file waits for the dialog");
        app.ui.dialog = None;
        placed(&mut app);
        assert_eq!(names(&app), ["photo.tif"]);
    }

    /// A file read while an object is being dragged, in its document or in another one, waits for
    /// the drag to end: the drag is not canceled, and the placed art stays.
    #[test]
    fn a_file_read_during_a_drag_waits_for_it_to_end() {
        use vectorcraft_tools::{PointerEvent, PointerKind};
        let path = saved("drag", "photo.tif", &picture(Tiff, 60, 40));
        for other_document in [false, true] {
            let mut app = app(true);
            start(&mut app, &json!({"path": path, "at": [300, 200]}), false).unwrap();
            if other_document {
                app.run("file.new", json!({"width": 400, "height": 300})).unwrap();
            }
            let square = app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 20, "height": 20})).unwrap()["id"].as_u64().unwrap();
            let dragged = app.session.active_index();
            app.select_tool("selection");
            let view = vectorcraft_engine::ViewInfo::default();
            for (kind, x) in [(PointerKind::Down, 20.0), (PointerKind::Drag, 40.0), (PointerKind::Drag, 60.0)] {
                app.session.pointer(&PointerEvent::new(kind, x, 20.0), view).unwrap();
            }
            assert!(app.session.in_interaction());
            let t0 = std::time::Instant::now();
            while t0.elapsed().as_millis() < 400 {
                poll(&mut app);
                std::thread::yield_now();
            }
            assert!(app.placing.pending.len() == 1 && app.session.in_interaction(), "other document: {other_document}");
            // The drag moves on before it ends.
            for kind in [PointerKind::Drag, PointerKind::Up] {
                app.session.pointer(&PointerEvent::new(kind, 60.0, 20.0), view).unwrap();
            }
            placed(&mut app);
            let moved = app.session.documents()[dragged.unwrap()].doc.node(NodeId(square)).and_then(|n| n.geometric_bounds());
            assert_eq!(moved.map(|b| (b.x0, b.y0)), Some((50.0, 10.0)), "other document: {other_document}");
            assert_eq!(names(&app), ["photo.tif"], "other document: {other_document}");
        }
    }

    /// A file read for one document while the place cursor is dragged in another waits for the drag
    /// to end, which the place cursor keeps to itself: the drag still places its file.
    #[test]
    fn a_file_read_during_a_place_cursor_drag_waits_for_it_to_end() {
        use vectorcraft_tools::{PointerEvent, PointerKind};
        let path = saved("cursor-drag", "photo.tif", &picture(Tiff, 60, 40));
        let cursor = [saved("cursor-drag", "a.png", &picture(Png, 8, 8)), saved("cursor-drag", "b.png", &picture(Png, 8, 8))];
        let mut app = app(true);
        start(&mut app, &json!({"path": path, "at": [300, 200]}), false).unwrap();
        app.run("file.new", json!({"width": 400, "height": 300})).unwrap();
        app.run("file.place.queue", json!({"paths": cursor})).unwrap();
        let view = vectorcraft_engine::ViewInfo::default();
        for (kind, x) in [(PointerKind::Down, 20.0), (PointerKind::Drag, 60.0)] {
            crate::canvas::dispatch(&mut app, &PointerEvent::new(kind, x, 20.0), view);
        }
        assert!(app.session.tool_busy() && !app.session.in_interaction());
        let t0 = std::time::Instant::now();
        while t0.elapsed().as_millis() < 400 {
            poll(&mut app);
            std::thread::yield_now();
        }
        assert!(app.placing.pending.len() == 1 && app.session.tool_busy(), "the file waits for the drag");
        crate::canvas::dispatch(&mut app, &PointerEvent::new(PointerKind::Up, 60.0, 20.0), view);
        placed(&mut app);
        let placed_in = |i: usize| -> Vec<(String, Option<vectorcraft_geom::Rect>)> {
            let st = &app.session.documents()[i];
            st.doc
                .layers
                .iter()
                .flat_map(|l| l.children().into_iter().flatten())
                .map(|n| (n.name.clone().unwrap_or_default(), n.geometric_bounds()))
                .collect()
        };
        assert_eq!(placed_in(0).iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["photo.tif"]);
        assert_eq!(placed_in(1), [("a.png".to_string(), Some(vectorcraft_geom::Rect::new(20.0, 20.0, 60.0, 60.0)))], "the drag placed its file");
    }

    /// A file read while a tool waits between its clicks (the Dimension tool after its first point),
    /// in the file's document or in another one, is placed without waiting for the next click. In
    /// the same document the tool keeps its point; in another one it starts afresh, as when the user
    /// switches documents.
    #[test]
    fn a_file_read_between_a_tools_clicks_is_placed() {
        use vectorcraft_tools::{PointerEvent, PointerKind};
        let path = saved("clicks", "photo.tif", &picture(Tiff, 60, 40));
        for other_document in [false, true] {
            let mut app = app(true);
            start(&mut app, &json!({"path": path, "at": [300, 200]}), false).unwrap();
            if other_document {
                app.run("file.new", json!({"width": 400, "height": 300})).unwrap();
            }
            app.select_tool("dimension");
            let view = vectorcraft_engine::ViewInfo::default();
            for kind in [PointerKind::Down, PointerKind::Up] {
                crate::canvas::dispatch(&mut app, &PointerEvent::new(kind, 20.0, 20.0), view);
            }
            assert!(app.session.tool_busy() && !app.session.in_interaction(), "other document: {other_document}");
            placed(&mut app);
            assert_eq!(names(&app), ["photo.tif"], "other document: {other_document}");
            assert_eq!(app.session.tool_busy(), !other_document, "other document: {other_document}");
        }
    }
}
