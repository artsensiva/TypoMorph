# Security

## Current status

TypoMorph is a Linux prototype with approved replacement requirements, not a verified secure cross-platform release. The diagnostic daemon currently prints input characters/buffers to stderr, which a service manager may retain. Legacy prompt commands and browser actions still include optional network text paths.

These are implementation gaps, not approved release behavior. See [PRIVACY.md](docs/PRIVACY.md) for the data boundary and [ARCHITECTURE.md](docs/ARCHITECTURE.md) for source evidence. Use synthetic input for authorized development checks.

## Required release boundaries

- Keep input and undo content in bounded transient RAM; never persist or transmit typed text/key sequences.
- Do not collect or correct protected/password or unknown-safety fields. Clear prior context on entry.
- Apply safety uniformly to trial, annual, and perpetual access; user application overrides cannot bypass it.
- Preserve genuine input and validate the target context before replacement or undo.
- No AI prompt feature, clipboard fallback, automatic telemetry, or automatic crash upload.
- Run as the ordinary user with explicit OS permission onboarding.
- Keep account/payment/update traffic separate from the input path.
- Authenticate offline entitlements; a local unkeyed checksum is not proof of service issuance.
- Verify update authenticity before installation, with no user-confirmed bypass for invalid/missing verification.

OS-controlled memory copies, metadata retention, service security, and signing/key recovery require design and validation. Do not claim those controls exist because they appear in this policy. [OPEN_QUESTIONS.md](docs/OPEN_QUESTIONS.md) tracks the unresolved details.

## Distribution and verification

The existing [release workflow](.github/workflows/release.yml) is configured to sign Debian-release checksum metadata using cosign. No artifact has been downloaded or verified during documentation work. The current website installer checks checksums and is not the approved signed updater.

The target distribution requires authenticated Debian packages, a signed Windows installer, and a signed/notarized macOS application. Exact verification procedures and identities must be documented alongside the actual release artifacts after implementation. See [TESTING.md](docs/TESTING.md).

## Reporting a vulnerability

Do not include real typed content, credentials, payment details, or personal account data in a public issue.

Use GitHub private vulnerability reporting if enabled for this repository, or the project's existing contact address, `support@typomorph.com`. Confirm the reporting route is operational before the public release. Reports should describe impact, affected version, and synthetic reproduction steps.

Support is best effort with no contractual SLA. Do not describe a response time or security certification as guaranteed.

## Scope and maintenance

The existing workspace, browser code, native host, packaging, website installer, and CI/release configuration are in scope for review. Dependencies are covered by the current CI audit configuration; an audit job does not guarantee that all vulnerabilities are absent.

The repository is pre-1.0 and has no committed long-term security-support branch or finalized post-release support-duration policy. The first-release commercial offers share one update stream; perpetual access includes future released updates, not a promise of perpetual development.
