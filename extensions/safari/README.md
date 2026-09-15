# Safari — not implemented

Safari Web Extensions cannot be a plain `manifest.json` + JS tree like Chrome/Firefox. They must be packaged inside a native macOS/iOS app target built with Xcode (`safari-web-extension-converter` or a native `SFSafariExtensionHandler`), and native messaging on Safari works through that wrapping app rather than a standalone stdio binary.

This repository's toolchain is Linux-only (the daemon depends on `evdev`/`uinput`/GNOME Shell D-Bus), so there is no macOS build here to host a Safari extension against. If Safari support is wanted later, it would need:

1. A macOS-side companion (out of scope for this repo).
2. Reusing `extensions/chrome`'s `background.js`/`popup.js` as a starting point — Safari's WebExtension API is Chromium/Firefox-compatible for the parts this extension uses (`storage`, `contextMenus`, `scripting`), but `nativeMessaging` is replaced by Safari's App Extension message-passing to the wrapping app.

No code is provided here beyond this note.
