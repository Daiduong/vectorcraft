# iPad app (iOS wrapper)

A native iPad shell around the web build: a full-screen `WKWebView` that runs `dist/web` from a
small HTTP server inside the app. The Rust app is unchanged; everything here is packaging. The same
shell also wraps the sibling apps' web builds (PhotoCraft, DesignCraft), one config file each.

```sh
packaging/ios/build.sh vectorcraft --sim                       # build, run on an iPad simulator
TEAM=ABCDE12345 packaging/ios/build.sh vectorcraft --device    # sign, install, launch on a connected iPad
packaging/ios/build.sh vectorcraft                             # only generate build/vectorcraft/VectorCraft.xcodeproj
```

`<app>` is a file name in `apps/` (`vectorcraft`, `photocraft`, `designcraft`). The other apps'
repos are looked for beside this one (`../photocraft`, `../designcraft`; set `CRAFT_ROOT` to
change it).

| Variable | Meaning |
|---|---|
| `TEAM` | Apple developer team id, needed for `--device` (Xcode › Settings › Accounts). A free Personal Team works. |
| `BUNDLE_PREFIX` | Bundle id prefix, default `dev.craft` (→ `dev.craft.vectorcraft`). A free team needs one of its own, e.g. `com.yourname`. |
| `SKIP_WEB=1` | Reuse the last `dist/web` instead of running `trunk build --release`. |
| `SIM_DEVICE`, `DEVICE` | Simulator / device id (`xcrun simctl list devices`, `xcrun devicectl list devices`). Default: the first iPad Pro/Air simulator, the first connected iPad. |

Needs: Xcode with the iOS platform, [XcodeGen](https://github.com/yonaskolb/XcodeGen)
(`brew install xcodegen`), and the [web build](../../docs/development.md#web-build) tools (`trunk`,
the `wasm32-unknown-unknown` target). Homebrew's `rust` has no wasm32 standard library: use rustup
(`brew install rustup`; the script puts `/opt/homebrew/opt/rustup/bin` first on `PATH`). The script
uses `/Applications/Xcode.app` when `xcode-select` points at the Command Line Tools.

Everything it generates goes to `packaging/ios/build/` (gitignored): the Xcode project, a copy of
the site, the app icon (copied from `assets/app-icon/<app>-1024.png`, so no asset is duplicated
here), and the derived data.

## How it works

| File | What it does |
|---|---|
| `Shared/LocalServer.swift` | Serves the bundled site on `http://localhost:<port>` (loopback only, GET/HEAD, nothing outside the site folder). `localhost` is a secure context, so WebGPU and the async clipboard work, and `.wasm` goes out as `application/wasm`. Each app has a fixed port (`CraftServerPort`), so the page's origin, and with it Data Recovery's `localStorage` and preferences, survives relaunches; if the port is taken it falls back to any free one. |
| `Shared/WebViewController.swift` | The web view. Save / Export downloads (blob URLs from `Services::download`) become `WKDownload`s, then a "Save to Files" sheet. Files opened from the Files app, or shared to the app, go to `window.<CraftOpenFileAPI>.openFile(name, bytes)` (VectorCraft's page API, `apps/vectorcraft-web/src/api.rs`), or, for an app without one, arrive as a file dropped on the canvas (eframe's `dropped_files`). It also adds the touch CSS VectorCraft's `index.html` already has (`touch-action: none` on the canvas, no selection or callouts), reloads the page if WebKit kills it for memory (Data Recovery then offers the unsaved work), and opens links to other sites in Safari. Debug builds forward the page's console to the app's log (`xcrun simctl launch --console-pty …`) and make the page inspectable from Safari's Develop menu. |
| `Shared/Info.plist.in` | The template build.sh fills from `apps/<app>.conf`: name, port, page API, the app's own document type (exported) and the file types it opens. iPad only, all orientations, files visible in Files › On My iPad. |
| `apps/<app>.conf` | Per app: `NAME`, `PORT`, `JS_API` (empty: open files by drop), `DOC_EXTS`, `IMPORT_UTIS`. |

## Notes

- Apple Pencil pressure, hover and palm rejection come from the web app itself
  (`apps/vectorcraft-web/src/pencil.rs`), so VectorCraft gets them in the wrapper as in Safari.
  PhotoCraft and DesignCraft don't have that module yet: there a pen acts like a finger.
- iOS gives a web view less memory than a native app, and wasm32 can't address more than 4 GB:
  very large documents can make WebKit reload the page.
- With a free Personal Team, an installed app stops opening after 7 days (install it again) and
  a device holds at most 3 such apps. The first install needs Settings › General › VPN & Device
  Management › trust the developer, and iPadOS 16+ needs Developer Mode (Settings › Privacy &
  Security).
- Tested on the iPad Pro 11-inch (M4) simulator (iOS 18.1): all three apps start, take touch
  input, create or open documents and draw. Installed and used on an iPad Pro 12.9-inch (4th
  generation). Not yet tested: saving to Files and opening from Files.
