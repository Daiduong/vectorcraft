#!/usr/bin/env bash
# Build only the pinned upstream client library for the native AppImage.
# Usage: bundle-dbus.sh APPDIR WORKDIR
# DBUS_SOURCE_ARCHIVE optionally supplies the same checksum-verified archive offline.
set -euo pipefail
appdir="${1:?APPDIR required}"
work="${2:?WORKDIR required}"
for tool in curl sha256sum tar meson ninja readelf python3; do
  command -v "$tool" >/dev/null || { echo "error: $tool is required to bundle libdbus" >&2; exit 1; }
done
mkdir -p "$work"
work="$(cd "$work" && pwd)"
version=1.16.2
source_url="https://dbus.freedesktop.org/releases/dbus/dbus-$version.tar.xz"
archive="$work/dbus-$version.tar.xz"
if [ -n "${DBUS_SOURCE_ARCHIVE:-}" ]; then
  cp "$DBUS_SOURCE_ARCHIVE" "$archive"
else
  curl --proto '=https' --proto-redir '=https' -fsSL "$source_url" -o "$archive"
fi
printf '%s  %s\n' '0ba2a1a4b16afe7bceb2c07e9ce99a8c2c3508e5dec290dbb643384bd6beb7e2' "$archive" | sha256sum -c -
tar -xf "$archive" -C "$work"
source_dir="$work/dbus-$version"
meson setup "$work/build" "$source_dir" --buildtype=release --auto-features=disabled \
  -Ddefault_library=shared -Dmessage_bus=false -Dtools=false -Dchecks=true \
  -Dsystemd=disabled -Dx11_autolaunch=disabled -Dinstalled_tests=false
meson compile -C "$work/build" -j "${CARGO_BUILD_JOBS:-2}"
library="$work/build/dbus/libdbus-1.so.3"
# No host libsystemd chain: this build may link only the baseline C/compiler runtimes.
dynamic="$(LC_ALL=C readelf -d "$library")"
needed="$(printf '%s\n' "$dynamic" | sed -n 's/.*(NEEDED).*\[\([^]]*\)\].*/\1/p')"
while IFS= read -r dependency; do
  case "$dependency" in
    libc.so.6|libm.so.6|libdl.so.2|libpthread.so.0|librt.so.1|libgcc_s.so.1) ;;
    *) echo "error: unexpected private libdbus dependency: $dependency" >&2; exit 1 ;;
  esac
done <<< "$needed"
printf '%s\n' "$dynamic" | grep -q 'SONAME.*\[libdbus-1.so.3\]'
if printf '%s\n' "$dynamic" | grep -Eq '\((RPATH|RUNPATH)\)'; then
  echo 'error: private libdbus must not retain a build-time library search path' >&2
  exit 1
fi
machine() { LC_ALL=C readelf -h "$1" | sed -n 's/^ *Machine: *//p'; }
if [ "$(machine "$library")" != "$(machine "$appdir/usr/bin/vectorcraft")" ]; then
  echo 'error: private libdbus and app architectures differ' >&2
  exit 1
fi
install -Dm755 "$library" "$appdir/usr/lib/libdbus-1.so.3"
notice="$appdir/usr/share/doc/vectorcraft/libdbus"
mkdir -p "$notice"
for name in COPYING AUTHORS LICENSES/AFL-2.0.txt LICENSES/AFL-2.1.txt LICENSES/MIT.txt LICENSES/BSD-3-Clause.txt LICENSES/TCL.txt LICENSES/LicenseRef-GAP.txt LICENSES/LicenseRef-pycrypto-orig.txt; do
  install -m644 "$source_dir/$name" "$notice/$(basename "$name")"
done
# Retain upstream copyright/license notices, without shipping daemon or tool source.
python3 - "$source_dir/dbus" "$work/build/meson-info/intro-targets.json" "$notice/COPYRIGHT-NOTICES.txt" <<'PY'
from pathlib import Path
import json
import re
import sys
source, targets, output = map(Path, sys.argv[1:])
paths = set(source.glob('*.h'))
for target in json.loads(targets.read_text()):
    if target['name'] == 'dbus-1':
        for group in target['target_sources']:
            paths.update(map(Path, group.get('sources', [])))
notices = []
for path in sorted(paths):
    if path.suffix in ('.c', '.h'):
        blocks = re.findall(r'/\*.*?\*/', path.read_text(), re.S)
        selected = [block for block in blocks if re.search(
            r'copyright|SPDX-License|licen[cs]e|redistribut|distribute and use freely', block, re.I)]
        if selected:
            notices.append(path.name + '\n' + '\n'.join(selected))
document = '\n\n'.join(notices) + '\n'
for required in ('Red Hat', 'CodeFactory AB', 'Kuchling', 'LicenseRef-pycrypto-orig', 'TCL'):
    if required not in document:
        raise SystemExit('Missing upstream attribution: ' + required)
output.write_text(document)
PY
printf 'libdbus %s, used under AFL-2.1 with its compatible permissive notices.\nUnmodified source: %s\nSHA256: %s\n' \
  "$version" "$source_url" '0ba2a1a4b16afe7bceb2c07e9ce99a8c2c3508e5dec290dbb643384bd6beb7e2' > "$notice/SOURCE.txt"
