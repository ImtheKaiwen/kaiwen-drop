import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./styles/tokens.css";
import "./styles/popup.css";
import { dict } from "./i18n";
import { blockBrowserChrome, h, painted } from "./lib/dom";
import { icon } from "./lib/icons";
import { ipc } from "./lib/ipc";
import { applyTheme } from "./lib/theme";

blockBrowserChrome();
const self = getCurrentWebviewWindow();
const label = self.label;
const dock = new URLSearchParams(location.search).get("mode") === "dock";
const state = await ipc.getState();
const d = dict(state.settings.language);
applyTheme(state.settings.theme, state.settings.language);
document.documentElement.classList.add("opaque"); // opak pencere: kenar boslugu/golge yok

const ic = h("span", {});
ic.innerHTML = icon("drop");
const surface = h("div", { class: `surface drop${dock ? " dock" : ""}`, role: "region", "aria-label": d.dropZoneAria });
surface.append(ic);

if (!dock) {
  surface.append(h("div", {}, d.dropZone));
  const x = h("button", { class: "icon-btn close", type: "button", "aria-label": d.close, onclick: () => void ipc.windowClose(label) });
  x.innerHTML = icon("x");
  surface.append(x);
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") void ipc.windowClose(label);
  });
}
document.getElementById("app")!.append(surface);

// Surukle-birak yollari Rust tarafinda yerel olaydan alinir; burada yalniz gorsel geri bildirim var.
await self.listen<boolean>("drop-hover", (e) => surface.classList.toggle("hover", e.payload));

await painted();
await ipc.windowReady(label);
surface.classList.add("show");
