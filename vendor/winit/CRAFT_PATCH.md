# Craft's winit 0.30.13 snapshot

Upstream: <https://github.com/rust-windowing/winit>, crates.io release 0.30.13.
Upstream archive SHA-256:
`a6755fa58a9f8350bd1e472d4c3fcc25f824ec358933bba33306d0b63df5978d`.
License: Apache-2.0; the upstream `LICENSE`, notices, documentation and examples
are retained. This is third-party source, not Craft-authored windowing code.

Craft modification, 2026-10-21: `src/platform/wayland.rs` adds
`WindowExtWayland::wayland_surface()`, which clones winit's existing `WlSurface`
proxy. This additive accessor introduces no unsafe code, dependency, new socket,
window-system policy, or change to existing methods. It is not an upstream API.
The application must retain its window and must not destroy or repurpose the
borrowed surface. The accessor permits a safe same-connection KDE AppMenu
association without reconstructing pointers from raw-window-handle.

The host uses `surface.backend().upgrade()` and `Connection::from_backend`, then
its own event queue. Winit still reads the socket and owns surface dispatch;
Craft only dispatches its own pending protocol events. No blocking roundtrip is
needed. This follows wayland-client's documented guest-library integration.

Each app workspace patches crates.io winit to its bundled copy of this snapshot,
so a checkout remains independently buildable. The canonical source is
`shared/winit`; update copies together and verify the sole upstream source diff
before upgrading. Keep the version pinned until this accessor is upstream or an
equivalent safe API is available. Preserve this notice with modified copies.

Sources:
- <https://docs.rs/wayland-client/latest/wayland_client/struct.EventQueue.html>
- <https://github.com/KDE/plasma-wayland-protocols/blob/master/src/protocols/appmenu.xml>
