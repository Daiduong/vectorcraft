#!/usr/bin/env bash
# Build a Craft app's iPad version: a native shell (Shared/) around its web build (apps/<app>-web).
#
#   packaging/ios/build.sh <app>             # trunk build, then generate build/<app>/<Name>.xcodeproj
#   packaging/ios/build.sh <app> --sim       # …then build and run on an iPad simulator
#   TEAM=ABCDE12345 packaging/ios/build.sh <app> --device   # …then sign, install and launch on a connected iPad
#
# <app> is vectorcraft or designcraft (apps/<app>.conf). DesignCraft's repo is looked for
# beside this one (CRAFT_ROOT, default: this repo's parent folder).
# Env: SKIP_WEB=1 reuses the last web build; BUNDLE_PREFIX (default dev.craft) → <prefix>.<app>;
# SIM_DEVICE / DEVICE pick a simulator / device by id.
# Needs: rustup with the wasm32-unknown-unknown target, trunk, xcodegen, Xcode.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
APP="${1:-}"; MODE="${2:-}"
if [ ! -f "$HERE/apps/$APP.conf" ]; then
  apps=""
  for f in "$HERE"/apps/*.conf; do b="${f##*/}"; apps="${apps:+$apps|}${b%.conf}"; done
  echo "usage: $0 <$apps> [--sim|--device]" >&2
  exit 2
fi
# shellcheck source=/dev/null
. "$HERE/apps/$APP.conf"
if [ "$APP" = vectorcraft ]; then REPO="$ROOT"; else REPO="${CRAFT_ROOT:-$(dirname "$ROOT")}/$APP"; fi
[ -d "$REPO/apps/$APP-web" ] || { echo "error: no $REPO/apps/$APP-web (clone storytold/$APP there or set CRAFT_ROOT)" >&2; exit 1; }
BUNDLE_ID="${BUNDLE_PREFIX:-dev.craft}.$APP"

# Homebrew's rust has no wasm32 std; prefer a rustup toolchain when there is one.
[ -d /opt/homebrew/opt/rustup/bin ] && export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
# Use the full Xcode even when xcode-select points at the Command Line Tools.
if ! xcodebuild -version >/dev/null 2>&1 && [ -d /Applications/Xcode.app ]; then
  export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
fi

if [ "${SKIP_WEB:-}" != 1 ]; then
  (cd "$REPO/apps/$APP-web" && trunk build --release)
fi
[ -f "$REPO/dist/web/index.html" ] || { echo "error: no $REPO/dist/web/index.html" >&2; exit 1; }

# Assemble build/<app>/: the shared sources, this app's Info.plist, icon and web site.
OUT="$HERE/build/$APP"
SRC="$OUT/$NAME"
rm -rf "$SRC" "$OUT/web" "$OUT/$NAME.xcodeproj"
mkdir -p "$SRC"
/bin/cp -f "$HERE"/Shared/*.swift "$SRC/"
/bin/cp -R "$HERE/Shared/Assets.xcassets" "$SRC/"
/bin/cp -f "$REPO/assets/app-icon/$APP-1024.png" "$SRC/Assets.xcassets/AppIcon.appiconset/AppIcon.png"
/bin/cp -R "$REPO/dist/web" "$OUT/web"
rm -f "$OUT/web/.trunk-lock"
strings() { for v in $1; do printf '\t\t\t\t<string>%s</string>\n' "$v"; done; }
DOC_EXTS_XML="$(strings "$DOC_EXTS")" IMPORT_UTIS_XML="$(strings "$IMPORT_UTIS")" NAME="$NAME" PORT="$PORT" JS_API="$JS_API" \
  perl -pe 's/\@APP_NAME\@/$ENV{NAME}/g; s/\@PORT\@/$ENV{PORT}/g; s/\@JS_API\@/$ENV{JS_API}/g;
            s/^\s*\@DOC_EXTS\@\n/$ENV{DOC_EXTS_XML}\n/; s/^\@IMPORT_UTIS\@\n/$ENV{IMPORT_UTIS_XML}\n/' \
  "$HERE/Shared/Info.plist.in" >"$SRC/Info.plist"
plutil -lint -s "$SRC/Info.plist"

cat >"$OUT/project.yml" <<YML
name: $NAME
options:
  deploymentTarget:
    iOS: "17.0"
settings:
  base:
    MARKETING_VERSION: "1.0"
    CURRENT_PROJECT_VERSION: "1"
    SWIFT_VERSION: "5.0"
    DEVELOPMENT_TEAM: "${TEAM:-}"
    CODE_SIGN_STYLE: Automatic
targets:
  $NAME:
    type: application
    platform: iOS
    sources:
      - path: $NAME
      # The static site from trunk, kept as a folder so its layout (fonts/…) survives. It sits
      # beside $NAME/, not inside it: some XcodeGen versions drop the parent of a nested folder.
      - path: web
        type: folder
        buildPhase: resources
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: $BUNDLE_ID
        PRODUCT_NAME: $NAME
        TARGETED_DEVICE_FAMILY: "2"
        INFOPLIST_FILE: $NAME/Info.plist
        GENERATE_INFOPLIST_FILE: NO
        ASSETCATALOG_COMPILER_APPICON_NAME: AppIcon
YML
(cd "$OUT" && xcodegen generate --quiet)
echo "Generated $OUT/$NAME.xcodeproj ($BUNDLE_ID)"

case "$MODE" in
  --sim)
    DEV="${SIM_DEVICE:-$(xcrun simctl list devices available | grep -E '^[[:space:]]+iPad' | grep -m1 -E 'Pro|Air' | sed -E 's/^ *(.*) \(([-0-9A-F]+)\).*/\2/')}"
    [ -n "$DEV" ] || { echo "error: no iPad simulator (Xcode › Settings › Components)" >&2; exit 1; }
    xcodebuild -project "$OUT/$NAME.xcodeproj" -scheme "$NAME" -configuration Debug \
      -destination "id=$DEV" -derivedDataPath "$OUT/derived" build -quiet
    xcrun simctl boot "$DEV" 2>/dev/null || true
    open "$(xcode-select -p)/Applications/Simulator.app" 2>/dev/null || true
    xcrun simctl install "$DEV" "$OUT/derived/Build/Products/Debug-iphonesimulator/$NAME.app"
    xcrun simctl launch "$DEV" "$BUNDLE_ID"
    ;;
  --device)
    [ -n "${TEAM:-}" ] || { echo "error: set TEAM to your Apple developer team id" >&2; exit 1; }
    DEV="${DEVICE:-$(xcrun devicectl list devices 2>/dev/null | awk '/iPad/ && /available/ {for (i=1;i<=NF;i++) if ($i ~ /^[0-9A-F]{8}-/) {print $i; exit}}')}"
    [ -n "$DEV" ] || { echo "error: no connected iPad (unlock it and trust this Mac)" >&2; exit 1; }
    xcodebuild -project "$OUT/$NAME.xcodeproj" -scheme "$NAME" -configuration Release \
      -destination "platform=iOS,id=$DEV" -derivedDataPath "$OUT/derived" -allowProvisioningUpdates build -quiet
    xcrun devicectl device install app --device "$DEV" "$OUT/derived/Build/Products/Release-iphoneos/$NAME.app"
    xcrun devicectl device process launch --device "$DEV" "$BUNDLE_ID" || \
      echo "Installed. If it won't open: on the iPad, Settings › General › VPN & Device Management › trust your developer certificate."
    ;;
esac
