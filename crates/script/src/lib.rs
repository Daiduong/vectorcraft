//! JavaScript scripts over the command registry (File › Scripts › Other Script…,
//! `vectorcraft-cli script`): a script is evaluated by Boa with a global `vc` that runs commands as
//! steps of one batch, so a run is one undo step per document it changes, and a failure (an
//! uncaught error, a limit) rolls every document back. See `docs/scripting.md`.
#![deny(unsafe_code)]

mod convert;
mod error;
mod host;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use boa_engine::{Context, Source};
use serde_json::Value;
use vectorcraft_engine::Session;

pub use error::ScriptError;

/// Largest script accepted (1 MiB).
pub const MAX_SOURCE_BYTES: usize = 1_048_576;

/// What stops a runaway script. Boa has no clock or memory cap: a script that builds one huge
/// string can still use a lot of memory (`docs/scripting.md`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    /// Loop iterations in the whole run (Boa counts every loop's together).
    pub loop_iterations: u64,
    /// Nested calls.
    pub recursion: usize,
    /// Instructions in the whole run.
    pub instructions: usize,
    /// `vc.run` calls in the whole run.
    pub commands: usize,
}

impl Default for Limits {
    fn default() -> Self {
        // `instructions`: about eight seconds of a release build (~250M instructions/s, measured
        // 2026-10-11).
        Self { loop_iterations: 10_000_000, recursion: 512, instructions: INSTRUCTION_BUDGET, commands: 100_000 }
    }
}

/// Instructions a run may take: Boa counts them only with its `fuzz` feature, which adds nothing
/// else.
const INSTRUCTION_BUDGET: usize = 2_000_000_000;

#[derive(Clone, Debug, Default)]
pub struct ScriptOptions {
    /// `vc.args` (the CLI's `--arg key=value`; empty from the menu).
    pub args: serde_json::Map<String, Value>,
    pub limits: Limits,
}

/// What a successful run returns.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptOutcome {
    /// The script's completion value (its last expression) as JSON, else as text.
    pub value: Value,
    /// `vc.log` lines.
    pub log: Vec<String>,
    /// `alert` messages, to show now the run is over.
    pub alerts: Vec<String>,
    /// Commands the script ran.
    pub commands: usize,
}

/// A script file's text: UTF-8 (a byte-order mark is dropped), at most [`MAX_SOURCE_BYTES`].
pub fn read_source(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(format!("the script is {} bytes; scripts are limited to 1 MiB", bytes.len()));
    }
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    String::from_utf8(bytes.to_vec()).map_err(|e| format!("the script isn't UTF-8 text (byte {} is not)", e.utf8_error().valid_up_to()))
}

/// Run `source` (named `name`, its file name) on `session`: one undo step labelled
/// `Script: <name>` per document it changes, or, when it fails, every document and the session's
/// paint, clipboard and Untitled-N count as they were.
pub fn run_script(session: &mut Session, source: &str, name: &str, opts: &ScriptOptions) -> Result<ScriptOutcome, ScriptError> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(ScriptError::new(name, format!("the script is {} bytes; scripts are limited to 1 MiB", source.len())));
    }
    let batch = session.begin_batch(&format!("Script: {name}"), None).map_err(|e| ScriptError::new(name, e.to_string()))?;
    // The session moves into the script's context for the run (its data must be 'static) and
    // comes back below, whatever happened.
    let shared: host::Shared = Rc::new(RefCell::new(Some(host::Host::new(std::mem::take(session), batch, opts.limits))));
    let result = vectorcraft_engine::guard::catch_panic(|| evaluate(&shared, source, name, opts));
    let host = match Rc::try_unwrap(shared) {
        Ok(cell) => cell.into_inner(),
        Err(shared) => shared.try_borrow_mut().ok().and_then(|mut h| h.take()),
    };
    // The context, the only other owner, was dropped inside `evaluate` (also when it panicked).
    let Some(host) = host else {
        return Err(ScriptError::new(name, "internal error: the script lost the session (please report this bug)"));
    };
    let host::Host { session: back, batch, log, alerts, commands, stopped, .. } = host;
    *session = back;
    match result {
        Ok(Ok(value)) => {
            session.finish_batch(batch);
            Ok(ScriptOutcome { value, log, alerts, commands })
        }
        Ok(Err(mut e)) => {
            session.roll_back_batch(batch);
            if let Some(why) = stopped {
                e.message = why;
            }
            e.log = log;
            Err(e)
        }
        Err(panic) => {
            session.roll_back_batch(batch);
            let mut e = ScriptError::new(
                name,
                format!("internal error: {panic} (the documents were left as they were before the script; please report this bug)"),
            );
            e.log = log;
            Err(e)
        }
    }
}

fn evaluate(shared: &host::Shared, source: &str, name: &str, opts: &ScriptOptions) -> Result<Value, ScriptError> {
    let mut ctx = Context::builder().instructions_remaining(opts.limits.instructions).build().map_err(|e| ScriptError::new(name, e.to_string()))?;
    let limits = ctx.runtime_limits_mut();
    limits.set_loop_iteration_limit(opts.limits.loop_iterations);
    limits.set_recursion_limit(opts.limits.recursion);
    ctx.insert_data(shared.clone());
    host::install(&mut ctx, &opts.args).map_err(|e| ScriptError::from_js(&e, &mut ctx, name))?;
    let src = Source::from_bytes(source.as_bytes()).with_path(Path::new(name));
    let value = ctx.eval(src).map_err(|e| ScriptError::from_js(&e, &mut ctx, name))?;
    // Promise callbacks the script queued run before it ends.
    ctx.run_jobs().map_err(|e| ScriptError::from_js(&e, &mut ctx, name))?;
    Ok(match convert::to_json(&value, &mut ctx) {
        Ok(v) => v,
        Err(_) => Value::String(value.to_string(&mut ctx).map(|s| s.to_std_string_escaped()).unwrap_or_default()),
    })
}
