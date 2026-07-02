#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

printf '%s\n' '== Lint/static checks =='
./scripts/lint.sh
printf '%s\n' '== Build =='
./scripts/build.sh
printf '%s\n' '== Tests =='
./scripts/test.sh
printf '%s\n' '== Documentation invariants =='
test -s README.md
test -s CONTRIBUTING.md
test -s SECURITY.md
test -s docs/README.md
test -s docs/06-SECURITY.md
test -s docs/13-OPERATIONS.md
test -s docs/16-CLI-CONTRACT.md
test -s docs/17-GUI-CONTRACT.md
test -s .github/workflows/ci.yml
grep -q './scripts/check.sh' .github/workflows/ci.yml
printf '%s\n' 'All configured checks passed.'
