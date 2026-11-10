import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./styles/tokens.css";
import "./styles/popup.css";
import { MenuSurface } from "./components/MenuSurface";
import { renderDialog, renderToast } from "./components/MoveFeedback";
import { dict } from "./i18n";
import { blockBrowserChrome, painted } from "./lib/dom";
import { ipc, menuApi, type PopupCfg } from "./lib/ipc";
import { applyTheme } from "./lib/theme";

blockBrowserChrome();
const self = getCurrentWebviewWindow();
const label = self.label;
const root = document.getElementById("app")!;
let menu: MenuSurface | null = null;
let seq = 0; // eski (gecikmis) yapilandirmalar yeni olani ezmesin

const NAV_KEYS = new Set(["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight", "Home", "End", "Enter", "Escape"]);

async function show(cfg: PopupCfg): Promise<void> {
  const mine = ++seq;
  menu?.destroy();
  menu = null;
  window.removeEventListener("keydown", dialogKeys);
  applyTheme(cfg.theme, cfg.lang);
  const d = dict(cfg.lang);
  let surface: HTMLElement;

  if (cfg.kind === "menu") {
    menu = new MenuSurface(root, cfg, menuApi, d);
    surface = menu.surface;
  } else if (cfg.kind === "dialog") {
    const r = renderDialog(root, cfg, d, (choice) => void ipc.dialogResolve(choice));
    surface = r.surface;
    window.addEventListener("keydown", dialogKeys);
    queueMicrotask(r.focusPrimary);
  } else {
    surface = renderToast(root, cfg, d);
  }

  await painted();
  if (mine !== seq) return;
  await menuApi.ready(label); // Rust: yerlestir + goster (beyaz flash yok)
  surface.classList.add("show");
  if (cfg.kind === "menu" && cfg.depth === 0) window.focus();
}

function dialogKeys(e: KeyboardEvent): void {
  if (e.key === "Escape") {
    e.preventDefault();
    void ipc.dialogResolve("cancel");
  }
}

function reset(): void {
  seq++;
  menu?.destroy();
  menu = null;
  window.removeEventListener("keydown", dialogKeys);
  root.replaceChildren();
}

// Yalniz kok popup klavye odagina sahiptir; tuslar Rust uzerinden en derin popup'a yonlenir.
document.addEventListener("keydown", (e) => {
  if (!menu || menu.depth !== 0 || !NAV_KEYS.has(e.key)) return;
  e.preventDefault();
  void menuApi.key(e.key);
});

await self.listen<PopupCfg>("popup-config", (e) => void show(e.payload));
await self.listen<string>("popup-key", (e) => menu?.handleKey(e.payload));
await self.listen("popup-reset", reset);

const first = await ipc.hello(label);
if (first) void show(first);
