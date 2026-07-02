#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if [[ -z "${DISPLAY:-}" ]]; then
  printf '%s\n' 'DISPLAY is not set; GUI visual smoke requires an X11 session.' >&2
  exit 2
fi

for tool in xprop magick; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf 'Missing required GUI smoke tool: %s\n' "$tool" >&2
    exit 2
  fi
done
if ! command -v maim >/dev/null 2>&1 && ! command -v import >/dev/null 2>&1; then
  printf '%s\n' 'Missing required GUI smoke screenshot tool: maim or import' >&2
  exit 2
fi

cargo build --quiet --locked --bin kfnotepad-gui

tmpdir=$(mktemp -d /tmp/kfnotepad-gui-visual.XXXXXX)
pid=''
cleanup() {
  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$tmpdir"
}
trap cleanup EXIT

first="$tmpdir/wrapped-gutter.md"
second="$tmpdir/second.rs"
existing_windows="$tmpdir/existing-windows.txt"
default_screenshot="target/gui-visual-smoke/kfnotepad-gui.png"
screenshot="${KFNOTEPAD_GUI_SMOKE_SCREENSHOT:-$default_screenshot}"
mkdir -p "$(dirname "$screenshot")"
mkdir -p "$tmpdir/config/kfnotepad"
cat > "$tmpdir/config/kfnotepad/config.toml" <<'CONFIG'
theme = "nocturne"
line_numbers = true
wrap = true
gui_restore_last_workspace = false
gui_font_family = "monospace"
gui_font_size = 16
gui_ui_font_size = 12
CONFIG
cat > "$first" <<'MARKDOWN'
# Wrapped gutter visual smoke

This intentionally long paragraph exercises the app-owned GUI replacement renderer with line numbers and word wrapping enabled so visual rows, continuation rows, cursor paint, syntax spans, and the line-number gutter share one deterministic row grid instead of relying on implicit paragraph height.

another-super-long-unbroken-token-that-must-fall-back-to-character-wrapping-because-there-is-no-safe-word-boundary-before-the-visible-editor-width

The smoke test only checks pixels, but this fixture makes the captured window cover the wrapped-gutter regression path.
MARKDOWN
cat > "$second" <<'RUST'
fn main() {
    println!("visual smoke with wrapping");
}
RUST

xprop -root _NET_CLIENT_LIST 2>/dev/null \
  | tr ',' '\n' \
  | sed -n 's/.*\(0x[0-9a-fA-F]\+\).*/\1/p' > "$existing_windows" || true

XDG_CONFIG_HOME="$tmpdir/config" target/debug/kfnotepad-gui "$first" "$second" >/dev/null 2>&1 &
pid=$!

window_id=''
window_title=''
deadline=$((SECONDS + 12))
while (( SECONDS < deadline )); do
  if ! kill -0 "$pid" 2>/dev/null; then
    printf '%s\n' 'kfnotepad-gui exited before a window could be captured.' >&2
    exit 1
  fi

  while read -r id; do
    if grep -qx "$id" "$existing_windows"; then
      continue
    fi
    title=$(xprop -id "$id" _NET_WM_NAME WM_NAME 2>/dev/null || true)
    if [[ "$title" == *"kfnotepad-gui"* && ( "$title" == *"$first"* || "$title" == *"$second"* ) ]]; then
      window_id="$id"
      window_title="$title"
      break
    fi
  done < <(
    xprop -root _NET_CLIENT_LIST 2>/dev/null \
      | tr ',' '\n' \
      | sed -n 's/.*\(0x[0-9a-fA-F]\+\).*/\1/p'
  )

  if [[ -n "$window_id" ]]; then
    break
  fi
  sleep 0.25
done

if [[ -z "$window_id" ]]; then
  printf '%s\n' 'Timed out waiting for a kfnotepad-gui X11 window.' >&2
  exit 1
fi

sleep 0.75
if command -v maim >/dev/null 2>&1; then
  timeout 8s maim -i "$window_id" "$screenshot" || {
    if command -v import >/dev/null 2>&1; then
      import -window "$window_id" "$screenshot"
    else
      exit 1
    fi
  }
else
  import -window "$window_id" "$screenshot"
fi

read -r width height colors deviation < <(
  magick "$screenshot" -colorspace RGB -format '%w %h %k %[fx:standard_deviation]\n' info:
)

if (( width < 500 || height < 300 )); then
  printf 'GUI screenshot too small: %sx%s\n' "$width" "$height" >&2
  exit 1
fi

if (( colors < 8 )); then
  printf 'GUI screenshot has too few colors: %s\n' "$colors" >&2
  exit 1
fi

awk -v deviation="$deviation" 'BEGIN { exit !(deviation > 0.01) }' || {
  printf 'GUI screenshot appears blank; standard deviation=%s\n' "$deviation" >&2
  exit 1
}

printf 'Captured GUI screenshot: %s\n' "$screenshot"
printf 'Captured GUI window: %s %s\n' "$window_id" "$window_title"
printf 'GUI screenshot stats: %sx%s, colors=%s, stddev=%s\n' "$width" "$height" "$colors" "$deviation"
printf '%s\n' 'GUI visual smoke fixture: line_numbers=true, wrap=true, long-line wrapped gutter.'
