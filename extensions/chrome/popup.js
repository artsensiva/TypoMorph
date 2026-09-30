chrome.runtime.sendNativeMessage("com.typomorph.native_host", { action: "status" }, () => {
  const missing = Boolean(chrome.runtime.lastError);
  document.getElementById("status").textContent = missing
    ? "Desktop connection unavailable."
    : "Correction unavailable: desktop integration is not ready.";
});
