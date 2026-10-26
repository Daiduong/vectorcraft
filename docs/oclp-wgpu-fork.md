# Proposal: share the OCLP wgpu compatibility fork

VectorCraft uses the wgpu 30.0.1 backport proposed for
[ArtCraft Launcher](https://github.com/storytold/craft-launcher/pull/917),
[LightCraft](https://github.com/storytold/lightcraft/pull/3087) and
[PdfCraft](https://github.com/storytold/pdfcraft/pull/4858).
It preserves configured internal indirect-call validation instead of clearing
that flag at startup. Adapter selection and startup recovery remain available.

## Dependency and ownership

- Prototype: <https://github.com/ITJamie/wgpu-fork>
- Branch: `codex/metal-msl-intel-v30.0.1`
- Immutable commit: `21c08e23135ba27ed1c6b68177ecc20c0d076dd9`
- Base: upstream `v30.0.1`, preserving the API used by the existing egui fork.
- Backports: <https://github.com/gfx-rs/wgpu/pull/10725> and
  <https://github.com/gfx-rs/wgpu/pull/10731>.

The backport probes supported Metal Shading Language versions, uses pipeline
workgroup limits when Intel's compiler rejects the shader attribute, and emits
runtime-sized storage arrays as pointers on Intel. Shader bounds checks remain
enabled. `WGPU_VALIDATION_INDIRECT_CALL` remains an explicit environment override.
The existing FXC default on Windows and backend selection policy are preserved.

Automatic MSL selection is the default. For diagnostics, the fork also accepts
`WGPU_METAL_MSL_VERSION=3.1` (exact, skips MSL probing) and
`WGPU_METAL_MAX_MSL=3.1` (caps automatic selection). An exact version takes
precedence over the cap and must be supported by the OS. Lower versions can
reduce advertised features; these overrides do not bypass the separate Intel
workgroup-size probe described below. No machine-wide setting is required.

A maintainer should decide whether to host this reviewed backport in
`storytold/wgpu`. Import the reviewed backport branch there, change the patch URL, retain an
immutable commit pin, and
regenerate `Cargo.lock`. The apps should share this dependency and return to an
upstream release once the fixes are released and validated.

Related report [#651](https://github.com/storytold/vectorcraft/issues/651) describes
an OCLP MacBook Pro with an NVIDIA GT 750M failing to start. It is already closed.
The current proposal targets the related Metal compiler compatibility class;
that NVIDIA configuration and native Monterey have not been tested.

## Validation

```sh
cargo xtask ci
cargo test --locked -p vectorcraft -j 2 -- --test-threads=1
cargo build --locked -p vectorcraft -p vectorcraft-cli -j 2
```

For an isolated native check, use a fresh scratch folder with an original SVG
containing only generated vector shapes. Choose a free loopback control port:

```sh
VECTORCRAFT_CONFIG_DIR=/private/tmp/vectorcraft-check/profile \
WGPU_BACKEND=metal WGPU_ADAPTER_NAME=Intel WGPU_VALIDATION_INDIRECT_CALL=1 \
MTL_DEBUG_LAYER=0 MTL_SHADER_VALIDATION=0 \
./vectorcraft --control 17979 \
  --automation-read-root /private/tmp/vectorcraft-check \
  --automation-write-root /private/tmp/vectorcraft-check \
  /private/tmp/vectorcraft-check/synthetic.svg
```

Use `WGPU_ADAPTER_NAME=AMD` for the discrete adapter on a dual-GPU Mac. Confirm
`graphicsAdapter` in `ui.inspect`, inspect `document.inspect`, request
`ui.screenshot` to check the native UI, and `ui.render` for the original art.
The control protocol is documented in [control-protocol.md](control-protocol.md).
It is enabled only for the test and confined to the scratch folder. External
Metal validation layers are disabled per process because they are incompatible
with this OCLP framework; wgpu's internal indirect-call validation is enabled.

### Results

- Apple M4 Max, macOS 27.0.1, Rust/Cargo 1.96.1: all 12 `cargo xtask ci`
  steps passed: formatting, strict workspace all-target Clippy, all 10,491
  workspace tests (14 skipped), doctests, no-bundled-fonts checks, PDF dependency,
  asset, brand, cursor, translation-report and layering gates, and wasm. All
  182 app-package tests also passed in a separate serial run. Native app/CLI
  builds and an inspected Metal UI/artwork smoke check also passed.
- The validation regression was observed failing with the old bypass and
  passing after its removal. It covers enabled, disabled and default flags,
  preserved Metal version options, and the existing backend/FXC choices,
  without changing process-global environment variables.
- MacBookPro11,5, OCLP macOS 15.8.1, Rust/Cargo 1.99.1: native app/CLI build
  and strict all-target app Clippy passed. Both AMD R9 M370X and Intel Iris Pro
  opened the original SVG correctly; native UI captures and rendered artwork
  were inspected. Both selected MSL 3.1 below the reported OS maximum of 3.2,
  exercising automatic fallback with no version overrides. Iris logged the
  expected rejected workgroup attribute and successfully used the workaround.
- The first Intel package-test command reached its 900-second compilation
  limit before any tests ran. The unfiltered retry from the populated Cargo build cache
  passed all 182 tests (175 unit and seven integration tests), with no skips,
  source changes or omitted cases. The initial timeout was during compilation,
  not a test assertion failure.
- Cargo emitted three unused egui patch-entry warnings. These are Cargo
  metadata warnings, not Clippy failures; the existing entries are unchanged.

| Hardware | Adapter confirmed by control channel | SVG active at startup (debug) | Native UI and original art |
|---|---|---|---|
| Apple M4 Max | Apple M4 Max | 2.3 s | Passed |
| MacBookPro11,5, OCLP macOS 15.8.1 | AMD Radeon R9 M370X | 2.3 s | Passed |
| Same Mac | Intel Iris Pro Graphics | 2.0 s | Passed |

These are individual debug runs using existing system compiler caches, not a
performance benchmark or a claim about cold-start timing. Each used fresh
settings, an original SVG containing only vector shapes, explicit Metal adapter
selection, internal indirect-call validation enabled, and external Metal layers
disabled per process. No assets or fonts were added to the repository.

### Known shared backport limits

Repeated enumeration can rerun the Intel workgroup-size compiler probe and stall
for about 30.5 seconds per repeat on the tested OCLP Iris Pro machine. This was
isolated during PdfCraft recovery tests and is tracked separately in
[wgpu #10742](https://github.com/gfx-rs/wgpu/issues/10742), with the cache follow-up
[#10743](https://github.com/gfx-rs/wgpu/pull/10743). The cache is not in this pin;
this proposal does not claim to fix repeated-enumeration delays.

The original shared backport's full M4 wgpu suite had eight failures, including
four untriaged capability/binding-array failures (`binding_array_storage_buffers`,
`partial_binding_array_storage_buffers`, `validate_capabilities`, and Computepass
Encoding), a subgroup expected-failure mismatch, two passthrough tests requiring
DXC, and a metallib test requiring offline Metal tools. This is limited hardware
qualification, not coverage of every OCLP GPU, native older macOS, Windows or
Linux hardware, or a clean full wgpu-suite pass.
