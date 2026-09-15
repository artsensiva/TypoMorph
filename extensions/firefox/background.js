// TypoMorph background script (Firefox). Owns the native messaging
// connection to the local `typomorph-native-host` process — no network calls
// happen here unless the user has explicitly enabled cloud prompt
// improvement in the popup (see popup.js), and even then only the selected
// text is sent. Firefox's `browser.*` API returns promises natively, so
// unlike the Chrome build this needs no callback-to-promise wrapping.

const NATIVE_HOST = "com.typomorph.native_host";

function readSelectionInPage() {
  const active = document.activeElement;
  if (active && (active.tagName === "TEXTAREA" || active.tagName === "INPUT")) {
    const { selectionStart, selectionEnd, value } = active;
    if (selectionStart !== selectionEnd) {
      return {
        text: value.slice(selectionStart, selectionEnd),
        kind: "input",
        start: selectionStart,
        end: selectionEnd,
      };
    }
  }
  const selection = window.getSelection();
  if (selection && selection.toString().length > 0) {
    return { text: selection.toString(), kind: "range" };
  }
  return null;
}

function writeReplacementInPage(payload) {
  const active = document.activeElement;
  if (payload.kind === "input" && active && typeof active.value === "string") {
    active.value =
      active.value.slice(0, payload.start) + payload.text + active.value.slice(payload.end);
    const caret = payload.start + payload.text.length;
    active.selectionStart = active.selectionEnd = caret;
    active.dispatchEvent(new Event("input", { bubbles: true }));
    return;
  }
  const selection = window.getSelection();
  if (selection && selection.rangeCount > 0) {
    const range = selection.getRangeAt(0);
    range.deleteContents();
    range.insertNode(document.createTextNode(payload.text));
  }
}

async function getSelection(tabId) {
  const [{ result }] = await browser.scripting.executeScript({
    target: { tabId },
    func: readSelectionInPage,
  });
  return result;
}

async function writeReplacement(tabId, selection, text) {
  await browser.scripting.executeScript({
    target: { tabId },
    func: writeReplacementInPage,
    args: [{ ...selection, text }],
  });
}

async function fixLayout(tabId) {
  const selection = await getSelection(tabId);
  if (!selection || !selection.text) return;
  try {
    const response = await browser.runtime.sendNativeMessage(NATIVE_HOST, {
      action: "correct_layout",
      text: selection.text,
    });
    if (response && response.ok && response.switched) {
      await writeReplacement(tabId, selection, response.corrected);
    }
  } catch (error) {
    console.error("TypoMorph: native host unavailable", error);
  }
}

async function improvePrompt(tabId) {
  const selection = await getSelection(tabId);
  if (!selection || !selection.text) return;
  try {
    const { cloud, apiKey } = await browser.storage.local.get(["cloud", "apiKey"]);
    const response = await browser.runtime.sendNativeMessage(NATIVE_HOST, {
      action: "improve_prompt",
      text: selection.text,
      cloud: Boolean(cloud),
      api_key: apiKey || null,
    });
    if (response && response.ok) {
      await writeReplacement(tabId, selection, response.improved);
    }
  } catch (error) {
    console.error("TypoMorph: native host unavailable", error);
  }
}

browser.commands.onCommand.addListener(async (command, tab) => {
  if (!tab || !tab.id) return;
  if (command === "fix-layout") await fixLayout(tab.id);
  if (command === "improve-prompt") await improvePrompt(tab.id);
});

browser.runtime.onInstalled.addListener(() => {
  browser.contextMenus.create({
    id: "typomorph-fix-layout",
    title: "Fix keyboard layout",
    contexts: ["selection", "editable"],
  });
  browser.contextMenus.create({
    id: "typomorph-improve-prompt",
    title: "Improve prompt (TypoMorph)",
    contexts: ["selection", "editable"],
  });
});

browser.contextMenus.onClicked.addListener(async (info, tab) => {
  if (!tab || !tab.id) return;
  if (info.menuItemId === "typomorph-fix-layout") await fixLayout(tab.id);
  if (info.menuItemId === "typomorph-improve-prompt") await improvePrompt(tab.id);
});
