# Security policy

Do not open a public issue containing a live exploit, credential, personal data, or production dump.
Until a private reporting channel is configured, contact the repository owner privately and include the minimum
reproduction needed. Never test against systems or data you do not own or have permission to assess.

See `docs/06-SECURITY.md` for the project's working threat model and release gate.

## Release gating policy

- Dependency and advisory checks are part of the local validation script (`scripts/check.sh`) via `scripts/security-check.sh`.
- In CI, the same check step runs on every matrix job.
