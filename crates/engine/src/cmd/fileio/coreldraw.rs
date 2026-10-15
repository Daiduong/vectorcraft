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
    let mut i = cdr::import::import(bytes, cdr::Limits::default()).map_err(super::err)?;
    // The package's own RGB and CMYK profiles, installed like those a native file carries; a
    // profile that can't be used leaves its space to the working space.
    let mut warnings = super::native::install_profiles(std::mem::take(&mut i.profiles));
    let p = &mut i.document.color_profiles;
    for name in [&mut p.rgb, &mut p.cmyk] {
        if name.as_deref().is_some_and(|n| vectorcraft_color::cms::profile(n).is_none()) {
            *name = None;
        }
    }
    warnings.append(&mut i.warnings);
    Ok((i.document, warnings))
}
