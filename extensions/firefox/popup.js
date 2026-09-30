browser.runtime.sendNativeMessage("com.typomorph.native_host", { action: "status" })
  .then(() => { document.getElementById("status").textContent = "Correction unavailable: desktop integration is not ready."; })
  .catch(() => { document.getElementById("status").textContent = "Desktop connection unavailable."; });
