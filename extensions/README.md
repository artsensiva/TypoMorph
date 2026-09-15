# TypoMorph browser extensions

Thin clients: a background script relays selected text to the local
`typomorph-native-host` process over Chrome/Firefox Native Messaging
(stdio, no network) for free layout correction and local prompt improvement.
Cloud prompt improvement only runs if you explicitly enable it in the popup
and either supply your own Anthropic API key or hold an active Pro license.

No extension in this directory has been published to any store. Loading
them is a manual, local step; publishing is a separate manual step for
later, after review.

## 1. Install the native messaging host

The `.deb` package (`scripts/build-deb.sh`) installs `typomorph-native-host`
to `/usr/bin/` and drops native-messaging manifest files for Chrome,
Chromium, Edge, and Firefox automatically. If you haven't installed the
`.deb` yet, or want to iterate without repackaging, register it manually:

```bash
cargo build --release -p native-host
mkdir -p ~/.config/google-chrome/NativeMessagingHosts
cat > ~/.config/google-chrome/NativeMessagingHosts/com.typomorph.native_host.json <<EOF
{
  "name": "com.typomorph.native_host",
  "description": "TypoMorph native messaging host",
  "path": "$(pwd)/target/release/typomorph-native-host",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://YOUR_EXTENSION_ID/"]
}
EOF
```

For Firefox, use `~/.mozilla/native-messaging-hosts/` and
`"allowed_extensions": ["typomorph@example.com"]` instead of
`allowed_origins` (matching `browser_specific_settings.gecko.id` in
`extensions/firefox/manifest.json`).

**Important:** `allowed_origins`/`allowed_extensions` must match the real
extension ID, which you only get after loading the extension (see below).
The system-wide manifests shipped by the `.deb` use placeholder IDs — edit
them (or the per-user copy above) once you know your actual ID.

## 2. Load the extension unpacked

**Chrome / Edge:**
1. Go to `chrome://extensions` (or `edge://extensions`).
2. Enable "Developer mode".
3. Click "Load unpacked" and select `extensions/chrome/`.
4. Copy the extension ID Chrome assigns it, and put it into the native
   messaging manifest's `allowed_origins` from step 1
   (`chrome-extension://<ID>/`).
5. Reload the extension after editing the manifest file.

**Firefox:**
1. Go to `about:debugging#/runtime/this-firefox`.
2. Click "Load Temporary Add-on" and select
   `extensions/firefox/manifest.json`.
3. The extension ID is fixed by `browser_specific_settings.gecko.id`
   (`typomorph@example.com`) — no copying needed, but the native messaging
   manifest's `allowed_extensions` must contain that same value.

Temporary Firefox add-ons are removed on browser restart; for persistent
local testing, package and sign through
[addons.mozilla.org](https://addons.mozilla.org) (self-distribution) — a
manual step, not automated here.

## 3. Use it

- Select text in any editable field, then either right-click → "Fix
  keyboard layout" / "Improve prompt (TypoMorph)", or use the keyboard
  shortcuts (`Ctrl+Shift+L` / `Ctrl+Shift+I` by default — configurable at
  `chrome://extensions/shortcuts`).
- Click the toolbar icon to see your Free/Pro tier and to set an Anthropic
  API key for the free bring-your-own-key cloud path.

## Building distributable packages

```bash
./build.sh
```

Produces `dist/typomorph-chrome.zip` and `dist/typomorph-firefox.zip` —
suitable for manual upload to the Chrome Web Store / addons.mozilla.org
after your own review. This script does not publish anything.

## Safari

Not implemented — see `safari/README.md` for why and what it would take.

## Permissions

`nativeMessaging`, `contextMenus`, `activeTab`, `scripting`, `storage`.
No host permissions, no `<all_urls>`, no persistent content script — text
is only read from the page when you explicitly trigger an action (keyboard
shortcut or context menu), via `activeTab` + on-demand `scripting`
injection.
