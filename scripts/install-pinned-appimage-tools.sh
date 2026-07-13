#!/usr/bin/env bash
set -euo pipefail

# Upstream continuous build from commit 8c8c91f762b412a19f4e8d2c4b35afb98f2d7c81.
appimagetool_asset_id=324406882
appimagetool_sha256=a6d71e2b6cd66f8e8d16c37ad164658985e0cf5fcaa950c90a482890cb9d13e0
# Upstream type2-runtime build from commit 75849dce7cc37e4319b633df1f116ca895c71a12.
runtime_asset_id=456065460
runtime_sha256=1cc49bcf1e2ccd593c379adb17c9f85a36d619088296504de95b1d06215aebbf

if [[ $# -ne 1 || -z "$1" ]]; then
  printf 'Usage: %s DESTINATION_DIRECTORY\n' "$0" >&2
  exit 2
fi

destination=$1
mkdir -p "$destination"

download_verified_asset() {
  local repository=$1
  local asset_id=$2
  local expected_sha256=$3
  local output_name=$4
  local temporary
  temporary=$(mktemp "${destination}/.${output_name}.XXXXXX")

  if ! curl \
    --fail \
    --silent \
    --show-error \
    --location \
    --header 'Accept: application/octet-stream' \
    --header 'X-GitHub-Api-Version: 2022-11-28' \
    --output "$temporary" \
    "https://api.github.com/repos/${repository}/releases/assets/${asset_id}"; then
    rm -f "$temporary"
    return 1
  fi

  if ! printf '%s  %s\n' "$expected_sha256" "$temporary" | sha256sum --check --status; then
    rm -f "$temporary"
    printf 'Checksum verification failed for GitHub release asset %s/%s.\n' \
      "$repository" "$asset_id" >&2
    return 1
  fi

  install -m 0755 "$temporary" "${destination}/${output_name}"
  rm -f "$temporary"
  printf 'Installed verified %s/%s as %s\n' "$repository" "$asset_id" \
    "${destination}/${output_name}"
}

download_verified_asset \
  AppImage/appimagetool \
  "$appimagetool_asset_id" \
  "$appimagetool_sha256" \
  appimagetool
download_verified_asset \
  AppImage/type2-runtime \
  "$runtime_asset_id" \
  "$runtime_sha256" \
  runtime-x86_64
