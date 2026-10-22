# Native Wayland file-drop integration draft

Status: review prototype, 2026-10-22. This is not a release-ready dependency update.
The installed contributor build retains earlier local fixes; this focused change
contains the windowing API adaptation, dependency pins, native read guard and
raster-drop regressions. It does not include unrelated import or single-instance
changes. The read guard retains the contributor build's earlier pipe/device fix
at the application's shared Open/Place boundary.

## Behavior and scope

Files dropped onto the welcome screen use the existing image-opening behavior:
a raster gets an artboard with its aspect ratio. A raster dropped onto an active
canvas is placed without resizing its artboards. No new image-size policy is
introduced. The native bridge delivers asynchronous Wayland offers to that
existing application path, including file-portal capabilities when offered.

The draft keeps both window backends and the upstream Linux maximize retry. It
adapts dialog parents, monitor geometry and surface sizing to winit 0.31.
The current-main Linux pen wrapper forwards the new surface and wake callbacks
and owns the returning handler for shutdown cleanup. It maps only recognized
unknown barrel-button codes, preserving pointer metadata and other events.

## Pinned review dependencies

These immutable fork revisions make the prototype reproducible. They must be
replaced with an agreed upstream dependency set before a release.

| Component | Revision | Review / blocker |
|---|---|---|
| winit 0.31 prototype | `4c38d2fe338245fb7a20602466cab94973313792` | [Bounded-transfer and portal API proposal #4790](https://github.com/rust-windowing/winit/issues/4790); API not accepted yet |
| egui/eframe 0.36 compatibility bridge | `9aa2af7425a924d50775f2f82be03de7b2d83f3a` | [Public compatibility branch](https://github.com/mockydevs/egui/tree/compat/vectorcraft-winit-031); downstream bridge, not an upstream release |
| AccessKit compatibility adapter | `ee75aece90ead74bad77ee4d97d19ae37eafdfa5` | [Release coordination #654](https://github.com/AccessKit/accesskit/issues/654); no accepted beta adapter release |
| glutin-winit | `4afe28190f25c8f040d64eee7f245459adff6e0c` | [Window error adaptation #1767](https://github.com/rust-windowing/glutin/pull/1767); current prototype code before its changelog commit |
| Smithay client toolkit | `d78cab10c8945a83f7a39aaf264c3ae679ca9d1e` | [Dropped-offer lifetime #546](https://github.com/Smithay/client-toolkit/pull/546) |

The winit prototype also includes [UTF-8 MIME parsing #4788](https://github.com/rust-windowing/winit/pull/4788).
The mainline egui migration and this 0.36 bridge are distinct: the bridge preserves
the application's current UI API while that migration is reviewed upstream.
Linux portal support adds a libdbus development dependency (`libdbus-1-dev` on
Ubuntu); the runtime must provide libdbus. Debian/RPM metadata declares the
runtime dependency. [Shared release-build prerequisite #57](https://github.com/storytold/craft-common-actions/pull/57)
adds native and RISC-V target headers and must be coordinated before release.
No host paths are embedded in Cargo.

## Security boundaries

Dropped paths use the same application reader as Open and Place. On Linux it
refuses directories, pipes and devices before opening, uses nonblocking/no-ctty
open flags, checks the opened descriptor again, and limits reads to 512 MiB even
if a file grows or reports a zero length. Ordinary document symlinks remain
supported. Tests cover the actual desktop service, FIFO/descriptors, devices,
symlinks and streaming/size limits. This is separate from the 1 MiB transfer-list
budget below: the transfer contains paths, not the image contents.

The dependency prototype requests data after Drop, accepts Copy actions and local
file URIs, and bounds transfers to 1 MiB, 256 paths, eight pending operations and
a ten-second timeout. Portal calls run off the UI thread with bounded workers
and a five-second D-Bus timeout. These are prototype guarantees requiring
upstream review, not claims that every platform has been validated.

A file URI alone does not grant sandbox access. Ubuntu's installed Desktop Icons
source offered only URI lists in the contributor's test. The real Desktop test
passed after the contributor explicitly authorized the local Flatpak override
`--filesystem=xdg-desktop:ro`. **This PR does not add that permission to either
Flatpak manifest**, and it does not claim URI-only drops outside granted folders
will work. Maintainers must decide the packaging/access policy separately.

## Local evidence and limits

| Check | Result and scope |
|---|---|
| Linux path safety | Open and existing-canvas Place share descriptor validation and a 512 MiB input cap; special files and symlinks to devices are refused; script/shortcut prefix reads use the same nonblocking open |
| Raster drop regression | Landscape, portrait and resolution metadata; second drop preserves board size; Undo removes only that image |
| Native protocol harness | Valid/overlapping transfers, cancellation, unsupported actions, remote/NUL paths, oversized and stalled writers, hover cleanup and repeated clipboard paste passed locally |
| Portal-backed import | GTK file-list serialization through native transfer into an installed test sandbox imported contributor-owned square/landscape images; original paths were initially inaccessible |
| Actual regular-app Desktop gesture | Contributor confirmed a 4500 by 4500 JPEG dropped from Ubuntu Desktop opened on a square board, after the explicit read-only Desktop grant |
| Installed binary commands | Same JPEG opened and rendered; placement into a 400 by 300 document preserved the board; Undo/Redo passed |
| Same-window opening | Installed build kept one window and opened a second document tab |
| Sibling save-back | One contributor-owned embedded vector completed edit/save/update and PSD round-trip in the installed sibling application; source file hash unchanged |
| Additional portal image | Contributor confirmed a 1536 by 1024 WebP opened after portal recovery; installed CLI imported/rendered it without warnings |

The real-file and physical-gesture results are from the contributor's integrated
installed build, not a binary compiled from this focused draft. The committed
raster regression runs on this draft; its complete CI result belongs in the PR.
No private artwork or screenshots are included in the repository.

A private-session-bus Flatpak test disturbed the live document-portal mount.
The service was restored and the empty app window restarted; the contributor
confirmed recovery. That setup was disabled. Reproduce portal integration in a
separate OS test session; a private D-Bus address or XDG runtime alone is not a
sufficient isolation guarantee.

Physical file-manager gestures beyond the confirmed Desktop case, other Linux
desktops, Windows/macOS runtime, mobile lifecycle and the complete upstream
dependency/platform matrix remain outstanding. Local success does not clear the
merge blockers above or change parity scores.
