# Security policy

Do not open a public issue containing a live exploit, credential, personal data, or production dump.
Until a private reporting channel is configured, contact the repository owner privately and include the minimum
reproduction needed. Never test against systems or data you do not own or have permission to assess.

See `docs/06-SECURITY.md` for the project's working threat model and release gate.

## Release gating policy

- Dependency and advisory checks are part of the local validation script (`scripts/check.sh`) via `scripts/security-check.sh`.
- Temporary advisory exceptions are version-pinned by `scripts/advisory-exceptions.sh`. Weekly Cargo Dependabot updates
  and the dependency-policy workflow surface compatible fixed releases and fail when a reviewed exception becomes stale.
- CI runs the same policy in a dedicated fail-closed security job, alongside the cross-platform build/test matrix.
