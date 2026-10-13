//! The tile cache of an infinite canvas (#1087): the art is rendered in square tiles fixed to the
//! document, one grid per zoom level, and kept. Panning back, zooming back and selecting reuse
//! them; only tiles the view newly reaches are rendered, in one region, off the UI thread. An edit
//! makes every tile stale, but stale tiles (and those of other zoom levels) keep standing in, so
//! the view never goes blank while the new ones render.
//!
//! Tiles are square in document space and turn with a rotated view (drawn as textured quads).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use egui::{Color32, Mesh, Pos2, TextureHandle, pos2};
use vectorcraft_doc::Document;
use vectorcraft_geom::{Affine, Point, Rect};
use vectorcraft_render::{RenderOptions, Rendered, Renderer};

/// A tile's side, in device pixels.
pub const TILE: u32 = 512;

/// Most tiles kept (each 1 MiB of texture).
#[cfg(not(target_arch = "wasm32"))]
const BUDGET: usize = 256;
#[cfg(target_arch = "wasm32")]
const BUDGET: usize = 64;

/// Most tiles one render covers along each side.
const MAX_JOB_SIDE: i64 = 6;

/// Zoom levels per doubling: a tile grid is reused only at its own zoom, give or take 0.01%.
const LEVELS_PER_OCTAVE: f64 = 8192.0;

/// A tile: its zoom level and its column and row in that level's grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileKey {
    pub level: i64,
    pub tx: i64,
    pub ty: i64,
}

/// The zoom level of `scale` (device pixels per point).
pub fn level_of(scale: f64) -> i64 {
    if scale.is_finite() && scale > 0.0 { (scale.log2() * LEVELS_PER_OCTAVE).round() as i64 } else { 0 }
}

/// The scale tiles of `level` render at.
pub fn scale_of(level: i64) -> f64 {
    (level as f64 / LEVELS_PER_OCTAVE).exp2()
}

impl TileKey {
    /// The document rectangle the tile covers.
    pub fn rect(self) -> Rect {
        let side = f64::from(TILE) / scale_of(self.level);
        Rect::new(self.tx as f64 * side, self.ty as f64 * side, (self.tx + 1) as f64 * side, (self.ty + 1) as f64 * side)
    }
}

/// The tiles of `level` that cover `view` (document rect), at most `cap` along each side (about
/// the centre). `None` when the view isn't a finite rect.
pub fn covering(level: i64, view: Rect, cap: i64) -> Option<(i64, i64, i64, i64)> {
    let side = f64::from(TILE) / scale_of(level);
    let f = |v: f64| (v / side).floor();
    let (x0, y0, x1, y1) = (f(view.x0), f(view.y0), f(view.x1), f(view.y1));
    if ![x0, y0, x1, y1].iter().all(|v| v.is_finite() && v.abs() < 9.0e15) {
        return None;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (x0 as i64, y0 as i64, x1 as i64, y1 as i64);
    let trim = |a: &mut i64, b: &mut i64| {
        if *b - *a + 1 > cap {
            let mid = (*a + *b) / 2;
            *a = mid - cap / 2;
            *b = *a + cap - 1;
        }
    };
    trim(&mut x0, &mut x1);
    trim(&mut y0, &mut y1);
    Some((x0, y0, x1, y1))
}

/// What the tiles show: the document (by its address, so any edit changes it) and the render
/// options' fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Content {
    /// The document's id in the session: another one clears the cache, so no tile of one
    /// document stands in for another's.
    pub uid: u64,
    pub doc: usize,
    pub opts: u64,
}

struct Entry {
    tex: TextureHandle,
    /// The content generation it was rendered for.
    generation: u64,
    used: u64,
}

struct Job {
    id: u64,
    generation: u64,
    level: i64,
    origin: (i64, i64),
    cols: i64,
    rows: i64,
    doc: Arc<Document>,
    opts: RenderOptions,
}

struct Done {
    id: u64,
    generation: u64,
    level: i64,
    origin: (i64, i64),
    cols: i64,
    rows: i64,
    img: Rendered,
}

/// Render the region of `job` (its tiles side by side).
fn render_job(r: &mut Renderer, job: &Job) -> Rendered {
    let s = scale_of(job.level);
    let (w, h) = ((job.cols as u32) * TILE, (job.rows as u32) * TILE);
    let view = Affine::translate((-(job.origin.0 as f64) * f64::from(TILE), -(job.origin.1 as f64) * f64::from(TILE))) * Affine::scale(s);
    r.render(&job.doc, w, h, view, &job.opts)
}

/// The tile cache and its render thread.
#[derive(Default)]
pub struct Tiles {
    map: HashMap<TileKey, Entry>,
    content: Option<Content>,
    generation: u64,
    frame: u64,
    next_id: u64,
    /// The job being rendered.
    pending: Option<u64>,
    tx: Option<Sender<Job>>,
    rx: Option<Receiver<Done>>,
    /// The renderer when there is no thread (wasm): one job a frame on the UI thread.
    local: Option<Renderer>,
    /// What the local renderer finished, picked up on the next frame.
    local_done: Option<Done>,
}

impl Tiles {
    /// Start the render thread (on wasm, render on the UI thread instead).
    fn start(&mut self, ctx: &egui::Context) {
        if self.tx.is_some() || self.local.is_some() {
            return;
        }
        if cfg!(target_arch = "wasm32") {
            self.local = Some(Renderer::new());
            return;
        }
        let (tx, jobs) = channel::<Job>();
        let (done_tx, rx) = channel::<Done>();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new().name("vectorcraft-tiles".into()).spawn(move || {
            let mut r = Renderer::new();
            while let Ok(job) = jobs.recv() {
                let img = render_job(&mut r, &job);
                let done = Done { id: job.id, generation: job.generation, level: job.level, origin: job.origin, cols: job.cols, rows: job.rows, img };
                if done_tx.send(done).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
        match spawned {
            Ok(_) => {
                self.tx = Some(tx);
                self.rx = Some(rx);
            }
            // No thread: render on this one.
            Err(_) => self.local = Some(Renderer::new()),
        }
    }

    /// Tiles cached (any generation), for tests and the status readout.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Forget every tile (another document, or the canvas leaves tiled drawing).
    pub fn clear(&mut self) {
        self.map.clear();
        self.content = None;
        self.pending = None;
    }

    /// Draw the art of `doc` for view `xf` (device pixels per point `ppp`) from tiles, asking for
    /// the missing ones. Returns how many visible tiles are current.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        quad: &dyn Fn(Rect) -> Vec<Pos2>,
        view: Rect,
        scale: f64,
        content: Content,
        doc: &Arc<Document>,
        opts: &RenderOptions,
    ) -> usize {
        self.start(ctx);
        self.frame += 1;
        if self.content.is_some_and(|c| c.uid != content.uid) {
            self.map.clear();
            self.pending = None;
        }
        if self.content != Some(content) {
            // Old tiles stay as stand-ins until new ones replace them.
            self.content = Some(content);
            self.generation += 1;
        }
        self.receive(ctx);
        let level = level_of(scale);
        let Some((x0, y0, x1, y1)) = covering(level, view, 64) else { return 0 };
        let mut current = 0;
        let mut missing = vec![];
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let key = TileKey { level, tx, ty };
                match self.map.get_mut(&key) {
                    Some(e) if e.generation == self.generation => {
                        e.used = self.frame;
                        current += 1;
                        textured(painter, e.tex.id(), corners4(&quad(key.rect())));
                    }
                    _ => missing.push(key),
                }
            }
        }
        // Stand-ins for the missing tiles: the stale tile at the same place, else the nearest
        // zoom level's tiles over it, each kept within the missing tile.
        for key in &missing {
            let r = key.rect();
            let screen = quad(r);
            let clip = egui::Rect::from_points(&screen);
            let screen = corners4(&screen);
            let p = painter.with_clip_rect(clip.intersect(painter.clip_rect()));
            if let Some(e) = self.map.get(key) {
                textured(&p, e.tex.id(), screen);
                continue;
            }
            let mut stand_ins: Vec<(i64, TileKey)> =
                self.map.keys().filter(|k| k.level != level && k.rect().overlaps(r)).map(|k| ((k.level - level).abs(), *k)).collect();
            stand_ins.sort_by_key(|(d, _)| std::cmp::Reverse(*d));
            for (_, k) in stand_ins.into_iter().rev().take(16).rev() {
                if let Some(e) = self.map.get(&k) {
                    textured(&p, e.tex.id(), corners4(&quad(k.rect())));
                }
            }
        }
        if !missing.is_empty() {
            self.request(level, &missing, view, doc, opts);
            ctx.request_repaint();
        }
        self.evict();
        current
    }

    /// Ask for the missing tiles nearest the view's centre: one region of them.
    fn request(&mut self, level: i64, missing: &[TileKey], view: Rect, doc: &Arc<Document>, opts: &RenderOptions) {
        if self.pending.is_some() {
            return;
        }
        let c = view.center();
        let Some(near) = missing.iter().min_by(|a, b| {
            let d = |k: &TileKey| k.rect().center().distance(c);
            d(a).total_cmp(&d(b))
        }) else {
            return;
        };
        // The missing tiles within a few tiles of the nearest one.
        let (mut x0, mut y0, mut x1, mut y1) = (near.tx, near.ty, near.tx, near.ty);
        for k in missing {
            if (k.tx - near.tx).abs() < MAX_JOB_SIDE && (k.ty - near.ty).abs() < MAX_JOB_SIDE {
                x0 = x0.min(k.tx);
                y0 = y0.min(k.ty);
                x1 = x1.max(k.tx);
                y1 = y1.max(k.ty);
            }
        }
        let (cols, rows) = ((x1 - x0 + 1).min(MAX_JOB_SIDE), (y1 - y0 + 1).min(MAX_JOB_SIDE));
        self.next_id += 1;
        let job = Job { id: self.next_id, generation: self.generation, level, origin: (x0, y0), cols, rows, doc: doc.clone(), opts: opts.clone() };
        self.pending = Some(job.id);
        if let Some(tx) = &self.tx {
            // A closed channel (the thread ended) falls back to rendering here.
            if tx.send(job).is_err() {
                self.tx = None;
                self.local = Some(Renderer::new());
                self.pending = None;
            }
        } else if let Some(r) = self.local.as_mut() {
            let img = render_job(r, &job);
            let done = Done { id: job.id, generation: job.generation, level, origin: job.origin, cols, rows, img };
            self.pending = None;
            self.local_done = Some(done);
        }
    }

    /// Take finished renders: cut them into tiles and upload them.
    fn receive(&mut self, ctx: &egui::Context) {
        let mut done: Vec<Done> = vec![];
        if let Some(rx) = &self.rx {
            while let Ok(d) = rx.try_recv() {
                done.push(d);
            }
        }
        if let Some(d) = self.local_done.take() {
            done.push(d);
        }
        for d in done {
            if self.pending == Some(d.id) {
                self.pending = None;
            }
            for row in 0..d.rows {
                for col in 0..d.cols {
                    let key = TileKey { level: d.level, tx: d.origin.0 + col, ty: d.origin.1 + row };
                    // A render for an older generation still replaces a missing or older tile.
                    if self.map.get(&key).is_some_and(|e| e.generation > d.generation) {
                        continue;
                    }
                    let Some(img) = cut(&d.img, col as u32, row as u32) else { continue };
                    let tex = ctx.load_texture(format!("tile-{}-{}-{}", key.level, key.tx, key.ty), img, egui::TextureOptions::LINEAR);
                    self.map.insert(key, Entry { tex, generation: d.generation, used: self.frame });
                }
            }
        }
    }

    /// Keep at most [`BUDGET`] tiles, dropping the least recently used.
    fn evict(&mut self) {
        if self.map.len() <= BUDGET {
            return;
        }
        let mut by_age: Vec<(u64, TileKey)> = self.map.iter().map(|(k, e)| (e.used, *k)).collect();
        by_age.sort_by_key(|(used, _)| *used);
        let extra = self.map.len() - BUDGET;
        for (_, k) in by_age.into_iter().take(extra) {
            self.map.remove(&k);
        }
    }
}

/// Tile (`col`, `row`) of a region render, as an egui image.
fn cut(img: &Rendered, col: u32, row: u32) -> Option<egui::ColorImage> {
    let (x0, y0) = (col * TILE, row * TILE);
    if x0 + TILE > img.width || y0 + TILE > img.height {
        return None;
    }
    let side = TILE as usize;
    let mut px = Vec::with_capacity(side * side * 4);
    for y in 0..side {
        let start = ((y0 as usize + y) * img.width as usize + x0 as usize) * 4;
        px.extend_from_slice(img.pixels.get(start..start + side * 4)?);
    }
    Some(egui::ColorImage::from_rgba_premultiplied([side, side], &px))
}

/// The four corners of a screen quad (missing ones at the origin).
fn corners4(q: &[Pos2]) -> [Pos2; 4] {
    let at = |i: usize| q.get(i).copied().unwrap_or(Pos2::ZERO);
    [at(0), at(1), at(2), at(3)]
}

/// Texture `tex` over the screen quad `q` (top-left, top-right, bottom-right, bottom-left).
fn textured(p: &egui::Painter, tex: egui::TextureId, q: [Pos2; 4]) {
    let mut mesh = Mesh::with_texture(tex);
    for (pos, uv) in q.into_iter().zip([pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0)]) {
        mesh.vertices.push(egui::epaint::Vertex { pos, uv, color: Color32::WHITE });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(egui::Shape::mesh(mesh));
}

/// A fingerprint of the render options, so tiles are redrawn when the way art draws changes.
pub fn opts_fingerprint(opts: &RenderOptions) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{opts:?}").hash(&mut h);
    h.finish()
}

/// The document rectangle around the view's corners (a rotated view's box).
pub fn view_rect(corners: [Point; 4]) -> Rect {
    corners.iter().skip(1).fold(Rect::from_points(corners[0], corners[0]), |r, p| r.union_pt(*p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_and_tiles_line_up() {
        // A level stands for one exact scale, and its tiles tile the plane without gaps.
        for s in [1e-6, 0.0313, 1.0, 1.5, 640.0, 1e9] {
            let l = level_of(s);
            assert!((scale_of(l) / s - 1.0).abs() < 1e-4, "{s}");
        }
        let a = TileKey { level: level_of(2.0), tx: 3, ty: -2 };
        let b = TileKey { tx: 4, ..a };
        assert_eq!(a.rect().x1, b.rect().x0);
        assert_eq!(a.rect().width(), 256.0, "512 px at 2 px per point");
        assert_eq!(level_of(f64::NAN), 0);
    }

    #[test]
    fn covering_tiles_span_the_view_within_the_cap() {
        let l = level_of(1.0);
        assert_eq!(covering(l, Rect::new(0.0, 0.0, 1000.0, 600.0), 64), Some((0, 0, 1, 1)));
        assert_eq!(covering(l, Rect::new(-1.0, -1.0, 1.0, 1.0), 64), Some((-1, -1, 0, 0)));
        // Far from the origin the indices stay exact: 9e8 is 1,757,812.5 tiles of 512 pt, so a
        // 1,000 pt view from there spans three tiles.
        let far = covering(l, Rect::new(9.0e8, 9.0e8, 9.0e8 + 1000.0, 9.0e8 + 1000.0), 64).unwrap();
        assert_eq!((far.0, far.2), (1_757_812, 1_757_814));
        // A capped span keeps its centre.
        let (x0, _, x1, _) = covering(l, Rect::new(0.0, 0.0, 512.0 * 100.0, 10.0), 10).unwrap();
        assert_eq!((x1 - x0 + 1, (x0 + x1) / 2), (10, 49));
        assert_eq!(covering(l, Rect::new(f64::NAN, 0.0, 1.0, 1.0), 64), None);
    }

    #[test]
    fn a_region_render_is_cut_into_its_tiles() {
        let side = TILE as usize;
        // Two tiles side by side: the left one red, the right one blue.
        let mut pixels = vec![0u8; side * 2 * side * 4];
        for y in 0..side {
            for x in 0..side * 2 {
                let i = (y * side * 2 + x) * 4;
                pixels[i..i + 4].copy_from_slice(if x < side { &[255, 0, 0, 255] } else { &[0, 0, 255, 255] });
            }
        }
        let img = Rendered { width: TILE * 2, height: TILE, pixels };
        let left = cut(&img, 0, 0).unwrap();
        let right = cut(&img, 1, 0).unwrap();
        assert_eq!(left.pixels[0], Color32::from_rgba_premultiplied(255, 0, 0, 255));
        assert_eq!(right.pixels[side * side - 1], Color32::from_rgba_premultiplied(0, 0, 255, 255));
        assert!(cut(&img, 2, 0).is_none(), "past the render");
    }

    /// One frame of `tiles` drawing `doc` over the view `view` at `scale` px per point; the
    /// number of current tiles drawn.
    fn frame(tiles: &mut Tiles, ctx: &egui::Context, doc: &Arc<Document>, uid: u64, view: Rect, scale: f64) -> usize {
        let opts = RenderOptions::default();
        let content = Content { uid, doc: Arc::as_ptr(doc) as *const u8 as usize, opts: opts_fingerprint(&opts) };
        let mut current = 0;
        let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };
        let mut out = ctx.run_ui(raw, |ui| {
            let painter = ui.painter().clone();
            let quad = |r: Rect| {
                vec![pos2(r.x0 as f32, r.y0 as f32), pos2(r.x1 as f32, r.y0 as f32), pos2(r.x1 as f32, r.y1 as f32), pos2(r.x0 as f32, r.y1 as f32)]
            };
            current = tiles.draw(ui.ctx(), &painter, &quad, view, scale, content, doc, &opts);
        });
        out.textures_delta.clear();
        current
    }

    /// The first frame asks for the view's tiles, the next draws them; a pan inside them renders
    /// nothing new; an edit makes them stale (stand-ins) until they render again; another
    /// document clears them.
    #[test]
    fn tiles_render_once_and_are_reused() {
        let ctx = egui::Context::default();
        let mut tiles = Tiles { local: Some(Renderer::new()), ..Default::default() };
        let doc = Arc::new(Document::new_infinite(vectorcraft_doc::ColorMode::Rgb));
        let view = Rect::new(0.0, 0.0, 700.0, 600.0);
        assert_eq!(frame(&mut tiles, &ctx, &doc, 1, view, 1.0), 0, "nothing yet");
        assert_eq!(frame(&mut tiles, &ctx, &doc, 1, view, 1.0), 4, "2 × 2 tiles of 512 px");
        let ids = tiles.next_id;
        // A small pan stays within the same tiles: no new render.
        assert_eq!(frame(&mut tiles, &ctx, &doc, 1, Rect::new(10.0, 10.0, 710.0, 610.0), 1.0), 4);
        assert_eq!(tiles.next_id, ids);
        // An edit: the tiles go stale and render again.
        let edited = Arc::new((*doc).clone());
        assert_eq!(frame(&mut tiles, &ctx, &edited, 1, view, 1.0), 0, "stale, shown as stand-ins");
        assert_eq!(tiles.len(), 4, "the stale tiles stay meanwhile");
        assert_eq!(frame(&mut tiles, &ctx, &edited, 1, view, 1.0), 4);
        // Another document starts afresh.
        frame(&mut tiles, &ctx, &edited, 2, view, 1.0);
        assert!(tiles.map.values().all(|e| e.generation == tiles.generation), "no tile of the other document");
    }
}
