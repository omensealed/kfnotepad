#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

printf '%s\n' '== Lint/static checks =='
./scripts/lint.sh
printf '%s\n' '== Build =='
./scripts/build.sh
printf '%s\n' '== Tests =='
./scripts/test.sh
./scripts/feature-check.sh
printf '%s\n' '== Security checks =='
./scripts/security-check.sh
printf '%s\n' '== Documentation invariants =='
test -s README.md
test -s CHANGELOG.md
test -s CONTRIBUTING.md
test -s SECURITY.md
test -s docs/README.md
test -s docs/06-SECURITY.md
test -s docs/13-OPERATIONS.md
test -s docs/16-CLI-CONTRACT.md
test -s docs/17-GUI-CONTRACT.md
test -s .github/dependabot.yml
test -s .github/workflows/ci.yml
test -s .github/workflows/dependency-policy.yml
test -s .github/workflows/release.yml
test -x scripts/advisory-exceptions.sh
test -x scripts/install-rust-toolchain.sh
test -x scripts/security-tool-versions.sh
grep -q 'cargo clippy --locked --all-targets --all-features -- -D warnings' .github/workflows/ci.yml
grep -q 'cargo test --locked --all-features' .github/workflows/ci.yml
grep -q 'package-ecosystem: "cargo"' .github/dependabot.yml
grep -q './scripts/advisory-exceptions.sh' .github/workflows/ci.yml
grep -q './scripts/advisory-exceptions.sh' .github/workflows/dependency-policy.yml
grep -q './scripts/install-rust-toolchain.sh' .github/workflows/ci.yml
grep -q './scripts/install-rust-toolchain.sh' .github/workflows/release.yml
grep -q 'source ./scripts/security-tool-versions.sh' .github/workflows/ci.yml
grep -q 'source ./scripts/security-tool-versions.sh' .github/workflows/dependency-policy.yml
if grep -REn 'rustup toolchain install stable' .github/workflows; then
    printf '%s\n' 'Workflows must install the version declared by rust-toolchain.toml.' >&2
    exit 1
fi
grep -q 'publish_release:' .github/workflows/release.yml
grep -q "if: github.event_name == 'push' || inputs.publish_release" .github/workflows/release.yml
if grep -REn 'uses:[[:space:]]+[^[:space:]]+@(v[0-9]+|main|master)([[:space:]]|$)' .github/workflows; then
    printf '%s\n' 'GitHub Actions must use immutable commit SHAs.' >&2
    exit 1
fi
if grep -REn '(^|[[:space:]])(ubuntu|windows|macos)-latest([[:space:]]|$)' .github/workflows; then
    printf '%s\n' 'GitHub Actions must use explicit hosted runner images.' >&2
    exit 1
fi
printf '%s\n' 'All configured checks passed.'
