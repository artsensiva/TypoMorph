# Existing browser extensions

## Status and first-release decision

Chrome/Edge and Firefox extension sources exist, but no extension here has been published to a store. Their first-release inclusion is conditional on measured native browser coverage and an owner decision.

An absent extension must not disable all browser fields. The desktop application must use native field information where reliable and suspend in protected or unknown-safety contexts. Safari browser compatibility is required on macOS; a Safari extension is deferred.

See [COMPATIBILITY.md](../docs/COMPATIBILITY.md) and [OPEN_QUESTIONS.md](../docs/OPEN_QUESTIONS.md).

## Current implementation

The extension popups send only `{ "action": "status" }` to the local host. Their
manifests request only `nativeMessaging`; no page scripts, selection extraction,
context-menu correction, credential storage, or AI/cloud controls remain.

The host enforces a 4 KiB frame bound and returns fixed content-free responses.
`correction_available` is always false. Missing/stopped desktop integration does
not enable standalone correction. Browser-spawned host readiness is not proof
that the desktop application is running or that a shared lifecycle exists.

## Controlled local development

This readiness-only code does not read or correct selected text. Existing instructions are for development, not supported end-user installation.

1. Build the host with `cargo build --release -p native-host`.
2. Register a browser-specific Native Messaging manifest pointing to the actual `typomorph-native-host` executable.
3. For Chrome/Edge, load `extensions/chrome/` through the browser's unpacked-extension developer interface.
4. Match the assigned extension ID in the host manifest's `allowed_origins`.
5. For Firefox, load `extensions/firefox/manifest.json` through its temporary-add-on interface and match its ID in `allowed_extensions`.
6. Reload after manifest changes and verify the status-only local host connection.

The current Debian packaging includes native-host manifests with placeholder extension IDs. A package installation alone therefore does not establish a working browser connection. Firefox's current source ID is `typomorph@example.com`; production identity/signing and persistent delivery remain release-design work.

Typical per-user manifest directories for the existing Linux setup are `~/.config/google-chrome/NativeMessagingHosts/` and `~/.mozilla/native-messaging-hosts/`. Verify the actual browser/version's location during implementation. Do not infer macOS/Windows host registration from these Linux paths.

## Packaging

From the repository root:

```bash
bash extensions/build.sh
```

The script creates Chrome and Firefox ZIP packages under `extensions/dist/`. It does not publish them or prove compatibility. Store publication and additional repositories are deferred.

## Required design if extensions are selected

Before implementation, settle missing/stopped-desktop behavior, reliable context safety, correction ownership, shared settings/access state, and delivery. Prevent duplicate corrections and preserve the no-network/no-persistence input boundary.

Do not advertise the current extension as a standalone approved product or assume its independent host already obeys desktop pause and account rules. [ARCHITECTURE.md](../docs/ARCHITECTURE.md) records the distinction. See [Safari status](safari/README.md) for that deferred extension.
