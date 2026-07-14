#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
source ./scripts/security-tool-versions.sh

require_security_tools=false
if [[ "${CI:-}" == "true" || "${KFNOTEPAD_STRICT_SECURITY_CHECKS:-}" == "1" ]]; then
    require_security_tools=true
fi

printf '%s\n' '== Advisory exception expiry checks =='
./scripts/advisory-exceptions.sh

if command -v cargo-deny >/dev/null 2>&1; then
    printf '%s\n' '== Dependency and policy checks (cargo-deny) =='
    installed_version="$(cargo deny --version | awk '{print $NF}')"
    if [[ "$installed_version" != "$CARGO_DENY_VERSION" ]]; then
        printf 'cargo-deny %s is required; found %s.\n' "$CARGO_DENY_VERSION" "$installed_version" >&2
        exit 1
    fi
    cargo deny check
elif [[ "$require_security_tools" == "true" ]]; then
    printf '%s\n' 'cargo-deny is required for CI/security checks but was not found.' >&2
    exit 1
else
    printf '%s\n' 'cargo-deny not available; skipping dependency policy check.'
fi

if command -v cargo-audit >/dev/null 2>&1; then
    printf '%s\n' '== Advisory checks (cargo-audit) =='
    installed_version="$(cargo audit --version | awk '{print $NF}')"
    if [[ "$installed_version" != "$CARGO_AUDIT_VERSION" ]]; then
        printf 'cargo-audit %s is required; found %s.\n' "$CARGO_AUDIT_VERSION" "$installed_version" >&2
        exit 1
    fi
    cargo audit
elif [[ "$require_security_tools" == "true" ]]; then
    printf '%s\n' 'cargo-audit is required for CI/security checks but was not found.' >&2
    exit 1
else
    printf '%s\n' 'cargo-audit not available; skipping advisory scan.'
fi
