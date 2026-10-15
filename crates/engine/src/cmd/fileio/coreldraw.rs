//! CorelDRAW `.cdr` import through the pure-Rust `cdr` crate: X4+ ZIP packages and bare version
//! 3–13 RIFF files, routed by their content ([`cdr::import::route`]).

use vectorcraft_doc::Document;

use super::super::*;

/// Is `bytes` a CorelDRAW file the reader routes (not just any ZIP archive)?
pub(super) fn sniff(bytes: &[u8]) -> bool {
    cdr::import::is_cdr(bytes)
}

/// A CorelDRAW file's document and what the import left out or approximated.
pub(super) fn import(bytes: &[u8]) -> Result<(Document, Vec<String>)> {
    let i = cdr::import::import(bytes, cdr::Limits::default()).map_err(super::err)?;
    Ok((i.document, i.warnings))
}
