//! Background rendering (native): heavy documents render off the UI thread while the canvas shows
//! the previous frame reprojected to the current view. Light documents render synchronously so
//! edits never lag behind overlays.

use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Mutex};

use vectorcraft_doc::Document;
use vectorcraft_geom::Affine;
use vectorcraft_render::{RenderOptions, Rendered, Renderer};

use crate::CacheKey;

pub struct Job {
    pub key: CacheKey,
    pub doc: Arc<Document>,
    pub w: u32,
    pub h: u32,
    pub view: Affine,
    pub opts: RenderOptions,
    #[cfg(feature = "gpu-canvas")]
    pub gpu_limits: vectorcraft_render::gpu::CanvasLimits,
}

pub struct Done {
    pub key: CacheKey,
    pub img: Option<Rendered>,
    #[cfg(feature = "gpu-canvas")]
    pub scene: Option<vectorcraft_render::gpu::PreparedScene>,
    pub reason: Option<String>,
    pub ms: f64,
    /// [`vectorcraft_render::FrameStats::moved`] of the frame.
    pub moved: usize,
}

pub struct Worker {
    tx: SyncSender<()>,
    jobs: Arc<Mutex<Option<Job>>>,
    done: Arc<Mutex<Option<Done>>>,
    pub pending: Option<CacheKey>,
}

impl Worker {
    /// Spawn the worker thread (None on wasm, where rendering stays synchronous).
    pub fn spawn(ctx: egui::Context) -> Option<Self> {
        if cfg!(target_arch = "wasm32") {
            return None;
        }
        // One pending snapshot and one completed result, even while the UI is stalled.
        let (tx, wakes) = sync_channel::<()>(1);
        let jobs = Arc::new(Mutex::new(None::<Job>));
        let done = Arc::new(Mutex::new(None));
        let worker_jobs = jobs.clone();
        let worker_done = done.clone();
        std::thread::Builder::new()
            .name("vectorcraft-render".into())
            .spawn(move || {
                let mut r = Renderer::new();
                while wakes.recv().is_ok() {
                    let Some(job) = worker_jobs.lock().unwrap_or_else(|e| e.into_inner()).take() else { continue };
                    let t0 = crate::now_ms();
                    let reason = None;
                    #[cfg(feature = "gpu-canvas")]
                    let reason = if job.key.gpu {
                        match r.prepare_gpu_with_limits(&job.doc, job.w, job.h, job.view, &job.opts, job.gpu_limits) {
                            Ok(scene) => {
                                *worker_done.lock().unwrap_or_else(|e| e.into_inner()) =
                                    Some(Done { key: job.key, img: None, scene: Some(scene), reason: None, ms: crate::now_ms() - t0, moved: 0 });
                                ctx.request_repaint();
                                continue;
                            }
                            Err(reason) => Some(reason),
                        }
                    } else {
                        reason
                    };
                    let img = r.render(&job.doc, job.w, job.h, job.view, &job.opts);
                    *worker_done.lock().unwrap_or_else(|e| e.into_inner()) = Some(Done {
                        key: job.key,
                        img: Some(img),
                        reason,
                        #[cfg(feature = "gpu-canvas")]
                        scene: None,
                        ms: crate::now_ms() - t0,
                        moved: r.stats.moved,
                    });
                    ctx.request_repaint();
                }
            })
            .ok()?;
        Some(Self { tx, jobs, done, pending: None })
    }

    /// A controlled worker: tests receive jobs and decide when a frame completes.
    #[cfg(test)]
    pub(crate) fn test_channel() -> (Self, TestJobs, TestDone) {
        let (tx, wakes) = sync_channel::<()>(1);
        let jobs = Arc::new(Mutex::new(None));
        let done = Arc::new(Mutex::new(None));
        (Self { tx, jobs: jobs.clone(), done: done.clone(), pending: None }, TestJobs { jobs, wakes }, TestDone(done))
    }

    pub fn submit(&mut self, job: Job) {
        if self.pending.as_ref() == Some(&job.key) {
            return;
        }
        self.pending = Some(job.key.clone());
        *self.jobs.lock().unwrap_or_else(|e| e.into_inner()) = Some(job);
        // A full wake queue already wakes the worker for the newest snapshot.
        if let Err(std::sync::mpsc::TrySendError::Disconnected(())) = self.tx.try_send(()) {
            self.pending = None;
        }
    }

    pub fn poll(&mut self) -> Option<Done> {
        let last = self.done.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(d) = &last
            && self.pending.as_ref() == Some(&d.key)
        {
            self.pending = None;
        }
        last
    }
}

/// The job side of [`Worker::test_channel`]: the newest submitted job, as the worker thread sees it.
#[cfg(test)]
pub(crate) struct TestJobs {
    jobs: Arc<Mutex<Option<Job>>>,
    wakes: std::sync::mpsc::Receiver<()>,
}

#[cfg(test)]
impl TestJobs {
    pub(crate) fn try_recv(&self) -> Option<Job> {
        let _ = self.wakes.try_recv(); // the wake only signals the job slot below
        self.jobs.lock().unwrap_or_else(|e| e.into_inner()).take()
    }
}

/// The result side of [`Worker::test_channel`]: completes a frame as the worker thread would.
#[cfg(test)]
pub(crate) struct TestDone(Arc<Mutex<Option<Done>>>);

#[cfg(test)]
impl TestDone {
    pub(crate) fn send(&self, done: Done) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(done);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn rapid_submissions_keep_only_the_latest_pending_document() {
        let mut worker = Worker::spawn(egui::Context::default()).unwrap();
        let doc = Arc::new(Document::new(32.0, 32.0));
        for revision in 0..100 {
            worker.submit(Job {
                key: CacheKey { doc: 1, revision, ..Default::default() },
                doc: doc.clone(),
                w: 32,
                h: 32,
                view: Affine::IDENTITY,
                opts: RenderOptions::default(),
                #[cfg(feature = "gpu-canvas")]
                gpu_limits: Default::default(),
            });
        }
        // At most one queued job, one running job, and this owning reference.
        assert!(Arc::strong_count(&doc) <= 3);
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(done) = worker.poll()
                && done.key.revision == 99
            {
                assert!(done.img.is_some());
                assert!(worker.pending.is_none());
                break;
            }
            assert!(Instant::now() < deadline, "latest render was not delivered");
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
