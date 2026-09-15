const NATIVE_HOST = "com.typomorph.native_host";

function sendNative(message) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendNativeMessage(NATIVE_HOST, message, (response) => {
      if (chrome.runtime.lastError) {
        reject(new Error(chrome.runtime.lastError.message));
      } else {
        resolve(response);
      }
    });
  });
}

async function loadStatus() {
  const statusEl = document.getElementById("status");
  try {
    const response = await sendNative({ action: "status" });
    statusEl.textContent = response.tier;
    statusEl.className = response.tier;
  } catch (error) {
    statusEl.textContent = "native host unavailable";
    statusEl.className = "error";
  }
}

async function loadSettings() {
  const { cloud, apiKey } = await chrome.storage.local.get(["cloud", "apiKey"]);
  document.getElementById("cloud").checked = Boolean(cloud);
  document.getElementById("apiKey").value = apiKey || "";
}

document.getElementById("save").addEventListener("click", async () => {
  const cloud = document.getElementById("cloud").checked;
  const apiKey = document.getElementById("apiKey").value.trim();
  await chrome.storage.local.set({ cloud, apiKey });
  window.close();
});

loadStatus();
loadSettings();
