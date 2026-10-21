//! Reload a document when its own file changes on disk (#3282): a script, a build or an agent
//! rewriting the SVG that is open shows its new version without reopening it. Modelled on
//! PdfCraft's watcher (`pdfcraft/crates/ui-egui/src/file_watch.rs`), so the Crafting Apps behave
//! alike.
//!
//! Cheap by design. One worker thread looks at each open document's file metadata (no reading)
//! every [`IDLE`], and blocks without waking at all while no open document has a file. A changed
//! file is read only once it has stopped changing ([`SETTLE`] apart) and, for SVG and PDF, ends as
//! a whole file does: its last KiB is read first, so a file still being written costs one small
//! read. The new bytes go to the UI thread, which is woken for them: a document without unsaved
//! changes is reloaded in place (same tab and view, history cleared, as File › Revert does); one
//! with unsaved changes asks first, when it is in front, and Cancel keeps them. A version whose
//! contents are the same as before (only its metadata changed: `touch`, `chmod`, a sync or backup
//! tool's extended attributes) isn't a change: the watcher keeps a hash of the contents, taken
//! once when it starts watching a version VectorCraft read or wrote.
//!
//! VectorCraft's own saves move the document's [`file_version`](vectorcraft_engine::DocState::file_version)
//! on: the file is then stamped as it is, so they never reload, and a version the watcher found
//! before is older than the document and isn't applied. Documents restored by Data Recovery
//! aren't watched: their file isn't what they hold.

use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime};

use vectorcraft_engine::cmd::fileio;

use crate::VectorcraftApp;

/// How often an unchanged file is looked at.
pub const IDLE: Duration = Duration::from_millis(250);
/// How long a changed file must stay the same before it is read: writers pause between chunks.
pub const SETTLE: Duration = Duration::from_millis(40);
/// How much of the end of a changed SVG or PDF is read to see whether it is a whole file.
const TAIL: usize = 1024;

/// One version of a file, told apart without reading it: its length and modification time, and
/// what the platform adds (Unix: device, inode and change time; Windows: creation and last-write
/// times in 100 ns ticks). The modification time alone is coarse on FAT and some network
/// drives, and a rewrite can keep the length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    len: u64,
    modified: Option<SystemTime>,
    extra: [u64; 4],
}

impl Stamp {
    fn of(meta: &Metadata) -> Self {
        #[cfg(unix)]
        let extra = {
            use std::os::unix::fs::MetadataExt;
            [meta.dev(), meta.ino(), meta.ctime().cast_unsigned(), meta.ctime_nsec().cast_unsigned()]
        };
        #[cfg(windows)]
        let extra = {
            use std::os::windows::fs::MetadataExt;
            [meta.creation_time(), meta.last_write_time(), 0, 0]
        };
        #[cfg(not(any(unix, windows)))]
        let extra = [0; 4];
        Self { len: meta.len(), modified: meta.modified().ok(), extra }
    }

    /// The file at `path` now; `None` when it is missing, not a file or can't be looked at.
    pub fn at(path: &str) -> Option<Self> {
        std::fs::metadata(path).ok().filter(Metadata::is_file).map(|m| Self::of(&m))
    }
}

/// A file to watch, as the UI thread sees it.
#[derive(Clone, Debug, PartialEq)]
pub struct Watched {
    /// The document's uid.
    pub doc: u64,
    pub path: String,
    /// The version VectorCraft just read or wrote (`None`: keep the watcher's own; the first
    /// version the watcher sees of a file it didn't watch).
    pub known: Option<Stamp>,
    /// The document's `file_version` it is watched for.
    pub version: u64,
}

/// A new version of a watched file, read and ready to open.
#[derive(Clone, Debug)]
pub struct Changed {
    pub doc: u64,
    /// The file it was read from, and the `file_version` it was watched for (see [`Watched`]).
    pub path: String,
    pub version: u64,
    pub stamp: Stamp,
    pub bytes: Arc<Vec<u8>>,
}

/// The watcher's record of one file.
struct Entry {
    doc: u64,
    path: String,
    version: u64,
    known: Option<Stamp>,
    /// A changed version seen once, waiting to be seen unchanged again.
    pending: Option<Stamp>,
    /// A hash of the contents last read: of the version VectorCraft read or wrote, then of each
    /// new version found. `None`: not read yet, or couldn't be.
    content: Option<u64>,
    /// The version VectorCraft read or wrote was looked at for `content` (read, or found unreadable).
    baseline: bool,
}

/// Looks at the watched files: on the worker thread, or on the UI thread in tests.
#[derive(Default)]
pub struct Watcher {
    files: Vec<Entry>,
}

impl Watcher {
    /// Watch `list` from now on. A file watched already for the same version keeps what it knows
    /// and its pending change; a version the UI thread knows (it just read or wrote the file)
    /// replaces the watcher's own.
    pub fn set(&mut self, list: Vec<Watched>) {
        let old = std::mem::take(&mut self.files);
        self.files = list
            .into_iter()
            .map(|w| {
                // A version the UI thread knows is read again for `content`.
                let prev = old.iter().find(|e| e.doc == w.doc && e.path == w.path && e.version == w.version).filter(|_| w.known.is_none());
                Entry {
                    known: w.known.or(prev.and_then(|e| e.known)),
                    pending: prev.and_then(|e| e.pending),
                    content: prev.and_then(|e| e.content),
                    baseline: prev.is_some_and(|e| e.baseline),
                    doc: w.doc,
                    path: w.path,
                    version: w.version,
                }
            })
            .collect();
    }

    /// Look at every file once: the new versions found, and whether a file is still changing
    /// (look again after [`SETTLE`] rather than [`IDLE`]).
    pub fn tick(&mut self) -> (Vec<Changed>, bool) {
        let mut found = Vec::new();
        let mut settling = false;
        for e in &mut self.files {
            // Missing (deleted, or being replaced) or unreadable: the document stays as it is.
            let Some(now) = Stamp::at(&e.path) else {
                e.pending = None;
                continue;
            };
            let known = *e.known.get_or_insert(now);
            if now == known {
                e.pending = None;
                // Hash the version VectorCraft has, once, so that a later change of its metadata
                // alone can be told apart from a change of its contents.
                if !e.baseline {
                    match read_whole(&e.path, now, false) {
                        Version::Ready(bytes) => (e.content, e.baseline) = (Some(hash(&bytes)), true),
                        Version::Incomplete => e.baseline = true,
                        Version::Busy => {}
                    }
                }
                continue;
            }
            if e.pending != Some(now) {
                // Changed since the last look: read it once it stops changing.
                e.pending = Some(now);
                settling = true;
                continue;
            }
            e.pending = None;
            match read_whole(&e.path, now, true) {
                Version::Ready(bytes) => {
                    (e.known, e.baseline) = (Some(now), true);
                    // Only its metadata changed: the same contents as the version before.
                    let content = hash(&bytes);
                    if e.content == Some(content) {
                        continue;
                    }
                    e.content = Some(content);
                    found.push(Changed { doc: e.doc, path: e.path.clone(), version: e.version, stamp: now, bytes });
                }
                // Not a whole file in this version (a writer that failed or paused): wait for the
                // next one.
                Version::Incomplete => e.known = Some(now),
                // Changed again while being read, or held by its writer: look again soon.
                Version::Busy => settling = true,
            }
        }
        (found, settling)
    }
}

/// What a changed file turned out to be.
enum Version {
    /// A whole file, read.
    Ready(Arc<Vec<u8>>),
    /// Not a whole file: a writer that failed or paused.
    Incomplete,
    /// Changing, replaced or held by its writer: look again.
    Busy,
}

/// The text a whole file of `path`'s format ends with (within its last KiB), when it has one:
/// `</svg>` for SVG, `%%EOF` for PDF and PDF-compatible `.ai`. Other formats are read as they are.
fn ending(path: &str) -> Option<&'static [u8]> {
    let ext = std::path::Path::new(path).extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "svg" => Some(b"</svg>"),
        "pdf" | "ai" => Some(b"%%EOF"),
        _ => None,
    }
}

/// A hash of a file's contents, to tell a version whose metadata alone changed.
fn hash(bytes: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

/// Read `path` if it is still the version `stamp` and, with `whole`, for SVG and PDF, ends as a
/// whole file does (its last KiB read first). The whole file is then read into a buffer of exactly
/// its size.
fn read_whole(path: &str, stamp: Stamp, whole: bool) -> Version {
    // Windows refuses to open a file its writer holds exclusively: not ready yet.
    let Ok(mut file) = File::open(path) else { return Version::Busy };
    let Ok(meta) = file.metadata() else { return Version::Busy };
    if Stamp::of(&meta) != stamp {
        return Version::Busy;
    }
    let len = meta.len();
    if whole && let Some(ending) = ending(path) {
        let mut end = [0u8; TAIL];
        let tail = usize::try_from(len).map_or(TAIL, |l| l.min(TAIL));
        let (Some(end), Ok(back)) = (end.get_mut(..tail), i64::try_from(tail)) else { return Version::Incomplete };
        if file.seek(SeekFrom::End(-back)).is_err() || file.read_exact(end).is_err() {
            return Version::Busy;
        }
        if !end.windows(ending.len()).any(|w| w == ending) {
            return Version::Incomplete;
        }
    }
    let mut bytes = Vec::new();
    // Too big to hold: the open version stays.
    let Ok(size) = usize::try_from(len) else { return Version::Incomplete };
    if bytes.try_reserve_exact(size).is_err() {
        return Version::Incomplete;
    }
    if file.seek(SeekFrom::Start(0)).is_err() || file.read_to_end(&mut bytes).is_err() {
        return Version::Busy;
    }
    // Still that version, under that name: a writer may have replaced the file meanwhile.
    if bytes.len() != size || Stamp::at(path) != Some(stamp) {
        return Version::Busy;
    }
    Version::Ready(Arc::new(bytes))
}

/// The worker thread: watches the newest list the UI thread sent and hands back new versions,
/// waking the UI for them. It ends when the app (its inbox's sender) is gone.
fn run(inbox: mpsc::Receiver<Vec<Watched>>, out: Arc<Mutex<Vec<Changed>>>, ctx: egui::Context) {
    let mut watcher = Watcher::default();
    // `None`: nothing to watch, so block until there is.
    let mut wait = None;
    loop {
        let message = match wait {
            None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
            Some(wait) => inbox.recv_timeout(wait),
        };
        match message {
            Ok(list) => watcher.set(list),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        while let Ok(list) = inbox.try_recv() {
            watcher.set(list);
        }
        if watcher.files.is_empty() {
            wait = None;
            continue;
        }
        let (found, settling) = watcher.tick();
        if !found.is_empty() {
            out.lock().unwrap_or_else(PoisonError::into_inner).extend(found);
            ctx.request_repaint();
        }
        wait = Some(if settling { SETTLE } else { IDLE });
    }
}

/// The app's side of the watcher.
#[derive(Default)]
pub struct FileWatch {
    /// The worker's inbox (`None` until a document with a file is open).
    inbox: Option<mpsc::Sender<Vec<Watched>>>,
    found: Arc<Mutex<Vec<Changed>>>,
    /// What the watcher was last told: (document uid, path, file version).
    sent: Vec<(u64, String, u64)>,
    /// In tests there is no thread: each frame looks once with this.
    pub inline: Option<Watcher>,
    /// New versions not acted on yet: a dialog is open, the document is being dragged in, or a
    /// save of it is being written.
    held: Vec<Changed>,
    /// The version each document was just reloaded from: what the watcher is told it knows, not
    /// the file as it is by then (another change may have come meanwhile).
    reloaded: HashMap<u64, Stamp>,
    /// Documents with unsaved changes whose file changed: asked about when they are in front.
    ask: HashMap<u64, Changed>,
}

impl VectorcraftApp {
    /// Every frame (desktop): tell the watcher about documents opened, closed and saved (without
    /// allocating when nothing changed), and act on the files that changed on disk.
    pub(crate) fn tick_file_watch(&mut self, ctx: &egui::Context) {
        let watched = || self.session.documents().iter().filter(|d| !d.recovered).filter_map(|d| Some((d.uid, d.path.as_deref()?, d.file_version)));
        let mut sent = self.file_watch.sent.iter();
        let unchanged =
            watched().all(|(uid, path, version)| sent.next().is_some_and(|s| s.0 == uid && s.1 == path && s.2 == version)) && sent.next().is_none();
        if !unchanged {
            let now: Vec<(u64, String, u64)> = watched().map(|(uid, path, version)| (uid, path.to_owned(), version)).collect();
            let list = now
                .iter()
                .map(|(uid, path, version)| {
                    let same = self.file_watch.sent.iter().any(|s| s.0 == *uid && s.1 == *path && s.2 == *version);
                    // Opened, saved or read again since the watcher was last told: that version
                    // is VectorCraft's own.
                    let known = if same { None } else { self.file_watch.reloaded.remove(uid).or_else(|| Stamp::at(path)) };
                    Watched { doc: *uid, path: path.clone(), known, version: *version }
                })
                .collect();
            let open: Vec<u64> = self.session.documents().iter().map(|d| d.uid).collect();
            self.file_watch.reloaded.retain(|uid, _| open.contains(uid));
            self.file_watch.ask.retain(|uid, _| open.contains(uid));
            if self.send_to_watcher(list, ctx) {
                self.file_watch.sent = now;
            }
        }
        if let Some(inline) = &mut self.file_watch.inline {
            let (found, _) = inline.tick();
            self.file_watch.found.lock().unwrap_or_else(PoisonError::into_inner).extend(found);
        }
        let mut found = std::mem::take(&mut self.file_watch.held);
        found.extend(std::mem::take(&mut *self.file_watch.found.lock().unwrap_or_else(PoisonError::into_inner)));
        for c in found {
            // Not while a dialog may be working on the document, during a drag, or while a save of
            // it is being written (it then makes this version stale).
            if self.ui.dialog.is_some() || self.session.in_interaction() || self.background.busy_with(c.doc) {
                self.file_watch.held.push(c);
            } else {
                self.file_changed(c);
            }
        }
        self.ask_about_changed_file();
    }

    /// Hand `list` to the watcher, starting it when needed. `false`: not delivered (the worker
    /// couldn't start or ended); the next frame tries again.
    fn send_to_watcher(&mut self, list: Vec<Watched>, ctx: &egui::Context) -> bool {
        if let Some(inline) = &mut self.file_watch.inline {
            inline.set(list);
            return true;
        }
        if self.file_watch.inbox.is_none() {
            if list.is_empty() {
                return true;
            }
            let (tx, rx) = mpsc::channel();
            let (out, ctx) = (self.file_watch.found.clone(), ctx.clone());
            if std::thread::Builder::new().name("vectorcraft-watch".into()).spawn(move || run(rx, out, ctx)).is_err() {
                return false;
            }
            self.file_watch.inbox = Some(tx);
        }
        let delivered = self.file_watch.inbox.as_ref().is_some_and(|tx| tx.send(list).is_ok());
        if !delivered {
            // The worker ended: start a new one next frame.
            self.file_watch.inbox = None;
        }
        delivered
    }

    /// A watched file has a new version: reload its document, or ask first when it has unsaved
    /// changes.
    fn file_changed(&mut self, c: Changed) {
        let docs = self.session.documents();
        let Some(index) = docs.iter().position(|d| d.uid == c.doc) else { return };
        let Some(st) = docs.get(index) else { return };
        // Found for an earlier reading or writing of the file, or for another file than the
        // document's now (a save, Save As or a reload came between): older than the document.
        if st.path.as_deref() != Some(c.path.as_str()) || st.file_version != c.version || st.recovered {
            return;
        }
        if st.is_dirty() {
            self.file_watch.ask.insert(c.doc, c);
            return;
        }
        self.reload_changed(index, c);
    }

    /// Replace the document at `index` with the version `c` of its file.
    fn reload_changed(&mut self, index: usize, c: Changed) {
        match fileio::reload_with(&mut self.session, index, &c.bytes) {
            Ok(_) => {
                self.file_watch.reloaded.insert(c.doc, c.stamp);
                self.status(format!("Reloaded {}: the file changed on disk", fileio::file_name(&c.path)));
            }
            // Can't be read (broken, or written in a way the watcher couldn't tell was unfinished):
            // the document stays as it is until the file changes again.
            Err(e) => self.status(e.to_string()),
        }
    }

    /// The document in front has unsaved changes and its file changed: ask whether to read it
    /// again ([`crate::io::ask_reload_changed`]), once no dialog is open. A document saved or read
    /// again meanwhile has nothing to ask; one whose changes were undone meanwhile just reloads.
    fn ask_about_changed_file(&mut self) {
        if self.ui.dialog.is_some() {
            return;
        }
        let Some(index) = self.session.active_index() else { return };
        let Some(st) = self.session.documents().get(index) else { return };
        let Some(c) = self.file_watch.ask.remove(&st.uid) else { return };
        if st.file_version != c.version || st.path.as_deref() != Some(c.path.as_str()) {
            return;
        }
        if st.is_dirty() {
            crate::io::ask_reload_changed(self);
        } else {
            self.reload_changed(index, c);
        }
    }
}
