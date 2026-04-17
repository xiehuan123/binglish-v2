import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

document.getElementById("exitBtn")?.addEventListener("click", () => {
  appWindow.close();
});
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") appWindow.close();
});
