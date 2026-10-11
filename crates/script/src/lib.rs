//! JavaScript scripts over the command registry (File › Scripts › Other Script…,
//! `vectorcraft-cli script`). See `docs/scripting.md`.
#![deny(unsafe_code)]

// Used by run_script (next commit).
#[allow(dead_code)]
mod convert;

/// Evaluates `source` in a fresh context → its completion value as text (spike; replaced by
/// `run_script`).
pub fn eval_text(source: &str) -> Result<String, String> {
    let mut ctx = boa_engine::Context::builder().instructions_remaining(10_000_000).build().map_err(|e| e.to_string())?;
    let v = ctx.eval(boa_engine::Source::from_bytes(source.as_bytes())).map_err(|e| e.to_string())?;
    v.to_string(&mut ctx).map(|s| s.to_std_string_escaped()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn evaluates_javascript() {
        assert_eq!(super::eval_text("[1, 2, 3].map(x => x * 2).join()"), Ok("2,4,6".to_string()));
        assert!(super::eval_text("Date.now() > 0 && Math.random() < 1").is_ok_and(|v| v == "true"));
    }
}
