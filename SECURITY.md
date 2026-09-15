# Security

## Data flow: local vs cloud

TypoMorph's core feature — keyboard layout correction — runs entirely locally and offline, for every supported language, free, with no network access at all. The only feature that can leave the machine is prompt improvement, and only when explicitly requested:

- **Local (default):** `typomorph improve-prompt --stdin` runs a deterministic, offline rule-based cleanup. No network call is made.
- **Bring-your-own-key (`--cloud --api-key ...` or `$TYPOMORPH_ANTHROPIC_KEY`):** the prompt text is sent directly to `api.anthropic.com` using a key you supply. TypoMorph does not see or store this key beyond the current process's environment/argument.
- **Managed (`--cloud`, requires an active Pro license):** the prompt text is sent to TypoMorph's own hosted proxy, which forwards it to Anthropic on your behalf.

In all cases, only the prompt text itself is transmitted — never other keystrokes, buffers, or license key material. See [README: Free vs Pro](README.md#free-vs-pro-prompt-improvement) for the full comparison.

## Verifying releases

See [README: Verifying releases](README.md#verifying-releases) for checksum and cosign signature verification of published `.deb` packages.

## Supported versions

TypoMorph is pre-1.0. Only the latest tagged release is supported with security fixes; there is no long-term-support branch yet.

## Scope

In scope: the `daemon`, `core-engine`, `licensing`, `platform-linux`, `prompt-cloud`, and `native-host` crates in this repository, the browser extensions in `extensions/`, and the release/packaging pipeline (`.github/workflows/`, `scripts/`). Vulnerabilities in third-party dependencies should be reported upstream; this repo's `cargo-audit` CI job (`.github/workflows/ci.yml`) tracks known advisories against `Cargo.lock` on every PR.

## Reporting a vulnerability

Please do not open a public GitHub issue for a security vulnerability.

1. Preferred: use GitHub's private vulnerability reporting for this repository (the "Report a vulnerability" button under the repo's Security tab).
2. Alternative: email `support@typomorph.com` with a description, reproduction steps, and impact assessment.

This is a small project without a dedicated security team — expect a best-effort acknowledgment, not a contractual SLA. Please give us reasonable time to investigate and ship a fix before any public disclosure.
