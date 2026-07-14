#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

expected_advisories=(
    RUSTSEC-2024-0436
    RUSTSEC-2025-0141
    RUSTSEC-2026-0192
    RUSTSEC-2026-0194
    RUSTSEC-2026-0195
)

# cargo-audit reports vulnerability advisories. cargo-deny additionally tracks
# the unmaintained advisories that are reviewed and allowed in deny.toml.
expected_audit_advisories=(
    RUSTSEC-2025-0141
    RUSTSEC-2026-0194
    RUSTSEC-2026-0195
)

expected_packages=(
    bincode@1.3.3
    paste@1.0.15
    quick-xml@0.39.4
    ttf-parser@0.25.1
)

configured_advisories="$({
    grep -Eo 'RUSTSEC-[0-9]{4}-[0-9]{4}' deny.toml || true
} | sort -u | tr '\n' ' ' | sed 's/ $//')"
expected_advisory_list="${expected_advisories[*]}"

if [[ "$configured_advisories" != "$expected_advisory_list" ]]; then
    printf '%s\n' 'deny.toml advisory exceptions changed; review and update scripts/advisory-exceptions.sh.' >&2
    printf 'Expected: %s\n' "$expected_advisory_list" >&2
    printf 'Found:    %s\n' "$configured_advisories" >&2
    exit 1
fi

configured_audit_advisories="$({
    grep -Eo 'RUSTSEC-[0-9]{4}-[0-9]{4}' .cargo/audit.toml || true
} | sort -u | tr '\n' ' ' | sed 's/ $//')"
expected_audit_advisory_list="${expected_audit_advisories[*]}"

if [[ "$configured_audit_advisories" != "$expected_audit_advisory_list" ]]; then
    printf '%s\n' '.cargo/audit.toml advisory exceptions changed; review and update scripts/advisory-exceptions.sh.' >&2
    printf 'Expected: %s\n' "$expected_audit_advisory_list" >&2
    printf 'Found:    %s\n' "$configured_audit_advisories" >&2
    exit 1
fi

for package in "${expected_packages[@]}"; do
    if ! cargo tree --locked --all-features --target all --invert "$package" >/dev/null 2>&1; then
        printf 'Tracked advisory package %s changed or disappeared.\n' "$package" >&2
        printf '%s\n' 'Review the new dependency graph and remove or revise its deny.toml exception.' >&2
        exit 1
    fi
done

printf '%s\n' 'Tracked cargo-deny and cargo-audit exceptions still match the reviewed dependency versions.'
