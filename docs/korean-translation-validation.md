# Korean translation validation

Catalog additions against `bc1ae0825264c5ca4b27bd718d6e04f2121b8bb3`: **374** entries. Existing Korean entries are unchanged. The provided translation data was checked against the current Rust sources; candidate keys with no matching production literal were excluded. No new translations were generated.

## Static checks

- Catalog structure, unique keys, original key preservation, named placeholders, trailing ellipses, and explicit shortcut tokens checked with Python 3.14.5.
- Existing catalog entries were compared with the baseline commit.
- New entries have source-location evidence in the local audit; dynamic labels use the existing translation paths.
- `git diff --check` passed.

## Checks not run

Cargo, rustc, and rustfmt are unavailable in this environment. Rust installation is awaiting user approval; no tools were installed. The following checks have **not** been run:

- `cargo test -p vectorcraft-ui-egui`
- `cargo fmt --all --check`
- `cargo clippy -p vectorcraft-ui-egui --all-targets -- -D warnings`
- `cargo xtask layers`, `cargo xtask assets`, `cargo xtask wasm`, and the full `cargo xtask ci` gate
- Native and web UI rendering, layout, runtime language switching, and font/glyph coverage

## Candidate filtering

- Excluded 95 supplied candidate keys without a matching production source literal.
- Excluded 0 supplied keys already proposed in open Korean translation PRs.

`Envelope` is the distortion object returned by `NodeKind::kind_label()` and translated in the Properties and Transform panels, so it uses the supplied contextual wording `외피 왜곡`. Print-image `Positive`/`Negative` retain the supplied `양화`/`음화` wording.
