#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

package_id="kfnotepad"
package_title="kfnotepad"
package_summary="Local UTF-8 text-file editor with TUI and Iced GUI front ends."
maintainer="kfnotepad maintainers <noreply@example.invalid>"

version=$(
  awk -F '=' '
    $1 ~ /^[[:space:]]*version[[:space:]]*$/ {
      gsub(/[[:space:]"]/, "", $2)
      print $2
      exit
    }
  ' Cargo.toml
)

if [[ -z "$version" ]]; then
  printf '%s\n' 'Could not determine package version from Cargo.toml.' >&2
  exit 1
fi

host_arch=$(uname -m)
platform=${KFNOTEPAD_PACKAGE_PLATFORM:-cachyos-linux-${host_arch}}
package_name="${package_id}-${version}-${platform}"
dist_dir=${KFNOTEPAD_DIST_DIR:-dist}
target_dir=${CARGO_TARGET_DIR:-target}
package_root="${target_dir}/package"
staging_dir="${package_root}/${package_name}"
deb_arch=${KFNOTEPAD_DEB_ARCH:-}
appimage_arch=${KFNOTEPAD_APPIMAGE_ARCH:-}

case "$host_arch" in
  x86_64)
    deb_arch=${deb_arch:-amd64}
    appimage_arch=${appimage_arch:-x86_64}
    ;;
  aarch64|arm64)
    deb_arch=${deb_arch:-arm64}
    appimage_arch=${appimage_arch:-aarch64}
    ;;
  *)
    deb_arch=${deb_arch:-$host_arch}
    appimage_arch=${appimage_arch:-$host_arch}
    ;;
esac

tarball="${dist_dir}/${package_name}.tar.gz"
deb_file="${dist_dir}/${package_id}_${version}_${deb_arch}.deb"
appimage_file="${dist_dir}/${package_id}-${version}-${appimage_arch}.AppImage"
sha_file="${dist_dir}/SHA256SUMS"

require_tool() {
  local tool=$1
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf 'Required packaging tool not found: %s\n' "$tool" >&2
    exit 1
  fi
}

install_common_docs() {
  local doc_dir=$1
  mkdir -p "$doc_dir/assets"
  install -m 0644 LICENSE "$doc_dir/LICENSE"
  install -m 0644 README.md "$doc_dir/README.md"
  install -m 0644 assets/kfnotepad-logo.png "$doc_dir/assets/kfnotepad-logo.png"
  install -m 0644 docs/13-OPERATIONS.md "$doc_dir/13-OPERATIONS.md"
  install -m 0644 docs/16-CLI-CONTRACT.md "$doc_dir/16-CLI-CONTRACT.md"
  install -m 0644 docs/17-GUI-CONTRACT.md "$doc_dir/17-GUI-CONTRACT.md"
}

install_desktop_assets() {
  local root=$1
  mkdir -p \
    "$root/usr/share/applications" \
    "$root/usr/share/icons/hicolor/scalable/apps"

  cat > "$root/usr/share/applications/${package_id}.desktop" <<EOF_DESKTOP
[Desktop Entry]
Type=Application
Name=${package_title}
Comment=${package_summary}
Exec=kfnotepad-gui %F
Icon=${package_id}
Terminal=false
Categories=Utility;TextEditor;
MimeType=text/plain;text/markdown;
Keywords=editor;notepad;text;notes;
EOF_DESKTOP

  cat > "$root/usr/share/icons/hicolor/scalable/apps/${package_id}.svg" <<'EOF_SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect width="128" height="128" rx="24" fill="#07101f"/>
  <path d="M29 21h50l20 20v66H29z" fill="#0b1727" stroke="#5ee7ff" stroke-width="7" stroke-linejoin="round"/>
  <path d="M79 21v20h20" fill="none" stroke="#5ee7ff" stroke-width="7" stroke-linejoin="round"/>
  <path d="M43 59h43M43 76h35M43 93h25" stroke="#c8efff" stroke-width="7" stroke-linecap="round"/>
</svg>
EOF_SVG
}

build_tarball() {
  rm -rf "$staging_dir"
  mkdir -p "$staging_dir/bin" "$staging_dir/docs" "$dist_dir"

  install -m 0755 "${target_dir}/release/kfnotepad" "$staging_dir/bin/kfnotepad"
  install -m 0755 "${target_dir}/release/kfnotepad-gui" "$staging_dir/bin/kfnotepad-gui"
  install_common_docs "$staging_dir/docs"
  mkdir -p "$staging_dir/assets"
  install -m 0644 LICENSE "$staging_dir/LICENSE"
  install -m 0644 README.md "$staging_dir/README.md"
  install -m 0644 assets/kfnotepad-logo.png "$staging_dir/assets/kfnotepad-logo.png"

  rm -f "$tarball" "${tarball}.sha256"
  tar -C "$package_root" -czf "$tarball" "$package_name"
  sha256sum "$tarball" > "${tarball}.sha256"
  printf 'Created %s\n' "$tarball"
  printf 'Created %s\n' "${tarball}.sha256"
}

build_deb() {
  require_tool dpkg-deb

  local deb_root="${package_root}/deb/${package_id}_${version}_${deb_arch}"
  rm -rf "$deb_root"
  mkdir -p \
    "$deb_root/DEBIAN" \
    "$deb_root/usr/bin" \
    "$deb_root/usr/share/doc/${package_id}"

  install -m 0755 "${target_dir}/release/kfnotepad" "$deb_root/usr/bin/kfnotepad"
  install -m 0755 "${target_dir}/release/kfnotepad-gui" "$deb_root/usr/bin/kfnotepad-gui"
  install_common_docs "$deb_root/usr/share/doc/${package_id}"
  install_desktop_assets "$deb_root"
  install -m 0644 LICENSE "$deb_root/usr/share/doc/${package_id}/copyright"

  local installed_size
  installed_size=$(du -sk "$deb_root/usr" | awk '{print $1}')

  cat > "$deb_root/DEBIAN/control" <<EOF_CONTROL
Package: ${package_id}
Version: ${version}
Section: editors
Priority: optional
Architecture: ${deb_arch}
Maintainer: ${maintainer}
Installed-Size: ${installed_size}
Depends: libc6 (>= 2.31), libgcc-s1, libfontconfig1, libfreetype6, libx11-6, libx11-xcb1, libxcb1, libxkbcommon0, libwayland-client0, libgl1, libegl1
Description: ${package_summary}
 kfnotepad edits normal local UTF-8 files through a terminal UI and
 a separate tiled Iced GUI. It has no database, accounts, network
 service, telemetry, or sync layer.
EOF_CONTROL

  rm -f "$deb_file" "${deb_file}.sha256"
  dpkg-deb --build --root-owner-group "$deb_root" "$deb_file"
  sha256sum "$deb_file" > "${deb_file}.sha256"
  printf 'Created %s\n' "$deb_file"
  printf 'Created %s\n' "${deb_file}.sha256"
}

build_appimage() {
  require_tool appimagetool

  local appdir="${package_root}/AppDir"
  rm -rf "$appdir"
  mkdir -p "$appdir/usr/bin" "$appdir/usr/share/doc/${package_id}"

  install -m 0755 "${target_dir}/release/kfnotepad" "$appdir/usr/bin/kfnotepad"
  install -m 0755 "${target_dir}/release/kfnotepad-gui" "$appdir/usr/bin/kfnotepad-gui"
  install_common_docs "$appdir/usr/share/doc/${package_id}"
  install_desktop_assets "$appdir"
  cp "$appdir/usr/share/applications/${package_id}.desktop" "$appdir/${package_id}.desktop"
  cp "$appdir/usr/share/icons/hicolor/scalable/apps/${package_id}.svg" "$appdir/${package_id}.svg"

  cat > "$appdir/AppRun" <<'EOF_APPRUN'
#!/usr/bin/env bash
set -euo pipefail
here=$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")

if [[ "${1:-}" == "--cli" ]]; then
  shift
  exec "$here/usr/bin/kfnotepad" "$@"
fi

exec "$here/usr/bin/kfnotepad-gui" "$@"
EOF_APPRUN
  chmod 0755 "$appdir/AppRun"

  local appimage_tmp="${package_root}/${package_id}-${version}-${appimage_arch}.AppImage.tmp"
  rm -f "$appimage_tmp"
  local appimage_args=(-n)
  if [[ -n "${KFNOTEPAD_APPIMAGE_RUNTIME:-}" ]]; then
    appimage_args+=(--runtime-file "$KFNOTEPAD_APPIMAGE_RUNTIME")
  fi
  ARCH="$appimage_arch" appimagetool "${appimage_args[@]}" "$appdir" "$appimage_tmp"
  rm -f "$appimage_file" "${appimage_file}.sha256"
  mv "$appimage_tmp" "$appimage_file"
  chmod 0755 "$appimage_file"
  sha256sum "$appimage_file" > "${appimage_file}.sha256"
  printf 'Created %s\n' "$appimage_file"
  printf 'Created %s\n' "${appimage_file}.sha256"
}

cargo build --locked --release --no-default-features --features 'tui syntax' --bin kfnotepad
cargo build --locked --release --no-default-features --features 'gui syntax' --bin kfnotepad-gui

build_tarball
build_deb
build_appimage

rm -f "$sha_file"
sha256sum "$tarball" "$deb_file" "$appimage_file" > "$sha_file"
printf 'Created %s\n' "$sha_file"
