#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

version=$(awk -F '=' '$1 ~ /^[[:space:]]*version[[:space:]]*$/ { gsub(/[[:space:]"]/, "", $2); print $2; exit }' Cargo.toml)
[[ -n "$version" ]] || { printf '%s\n' 'Could not determine package version.' >&2; exit 1; }

case "$(uname -m)" in
  arm64) arch=arm64 ;;
  x86_64) arch=x86_64 ;;
  *) printf 'Unsupported macOS architecture: %s\n' "$(uname -m)" >&2; exit 1 ;;
esac

dist_dir=${KFNOTEPAD_DIST_DIR:-dist}
target_dir=${CARGO_TARGET_DIR:-target}
package_root="$target_dir/package/macos"
app="$package_root/kfnotepad.app"
dmg_root="$package_root/dmg"
dmg="$dist_dir/kfnotepad-$version-macos-$arch.dmg"

for tool in cargo codesign hdiutil iconutil sips shasum; do
  command -v "$tool" >/dev/null 2>&1 || { printf 'Required packaging tool not found: %s\n' "$tool" >&2; exit 1; }
done

cargo build --locked --release --no-default-features --features 'tui syntax' --bin kfnotepad
cargo build --locked --release --no-default-features --features 'gui syntax' --bin kfnotepad-gui

rm -rf "$package_root"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$dmg_root" "$dist_dir"
install -m 0755 "$target_dir/release/kfnotepad-gui" "$app/Contents/MacOS/kfnotepad-gui"

cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleDisplayName</key><string>kfnotepad</string>
  <key>CFBundleExecutable</key><string>kfnotepad-gui</string>
  <key>CFBundleIconFile</key><string>kfnotepad.icns</string>
  <key>CFBundleIdentifier</key><string>dev.kfnotepad.app</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>kfnotepad</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict></plist>
EOF

iconset="$package_root/kfnotepad.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" assets/kfnotepad-logo.png --out "$iconset/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z "$double" "$double" assets/kfnotepad-logo.png --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/kfnotepad.icns"

# Ad-hoc signing keeps the bundle internally consistent. Distribution signing and
# notarization can replace this step when Apple Developer credentials are configured.
codesign --force --deep --sign - "$app"

cp -R "$app" "$dmg_root/"
install -m 0755 "$target_dir/release/kfnotepad" "$dmg_root/kfnotepad"
install -m 0644 README.md CHANGELOG.md LICENSE "$dmg_root/"
ln -s /Applications "$dmg_root/Applications"

rm -f "$dmg" "$dmg.sha256"
hdiutil create -volname "kfnotepad $version" -srcfolder "$dmg_root" -ov -format UDZO "$dmg"
shasum -a 256 "$dmg" > "$dmg.sha256"
printf 'Created %s\nCreated %s\n' "$dmg" "$dmg.sha256"
