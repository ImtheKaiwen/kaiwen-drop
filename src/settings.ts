import "./styles/tokens.css";
import "./styles/popup.css";
import "./styles/settings.css";
import { SettingsView } from "./components/SettingsView";
import { blockBrowserChrome, painted } from "./lib/dom";
import { ipc } from "./lib/ipc";
import { applyTheme } from "./lib/theme";

blockBrowserChrome();
const state = await ipc.getState();
applyTheme(state.settings.theme, state.settings.language);
new SettingsView(document.getElementById("app")!, state).mount();
await painted();
await ipc.windowReady("settings"); // Rust penceremi simdi gosterir: beyaz flash yok
