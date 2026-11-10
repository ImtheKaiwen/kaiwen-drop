import { invoke } from "@tauri-apps/api/core";

export interface Folder {
  name: string;
  path: string;
  has_children: boolean;
}

export interface Root {
  id: string;
  name: string;
  path: string;
  known: string | null;
}

export interface Settings {
  version: number;
  roots: Root[];
  language: "tr-TR" | "en-US";
  theme: "system" | "light" | "dark";
  shortcut: string;
  density: "normal" | "compact";
  show_hidden: boolean;
  hide_dev_folders: boolean;
  confirm_move: boolean;
  corner_dock: boolean;
}

export interface RootInfo {
  id: string;
  path: string;
  exists: boolean;
}

export interface AppState {
  settings: Settings;
  roots: RootInfo[];
  hotkey_error: string | null;
  autostart: boolean;
  system_dark: boolean;
}

export interface SaveResult {
  state: AppState;
  error: string | null;
}

/** Popup/diyalog/bildirim penceresinin Rust'tan aldigi yapilandirma. */
export interface BaseCfg {
  label: string;
  theme: string;
  lang: string;
  density: string;
  size: [number, number];
}
export interface MenuCfg extends BaseCfg {
  kind: "menu";
  session: number;
  depth: number;
  path?: string;
  items: Folder[];
  moveHere: boolean;
  keyboard: boolean;
}
export interface DialogCfg extends BaseCfg {
  kind: "dialog";
  count: number;
  names: string[];
  conflicts: number;
  dest: string;
}
export interface ToastCfg extends BaseCfg {
  kind: "toast";
  data: import("../i18n").ToastData;
}
export type PopupCfg = MenuCfg | DialogCfg | ToastCfg;

export interface MenuApi {
  openChild(label: string, path: string, rowTop: number, keyboard: boolean): Promise<void>;
  closeBelow(label: string): Promise<void>;
  moveHere(label: string, path: string): Promise<void>;
  key(key: string): Promise<void>;
  ready(label: string): Promise<void>;
}

export const menuApi: MenuApi = {
  openChild: (label, path, rowTop, keyboard) => invoke("popup_open_child", { label, path, rowTop, keyboard }),
  closeBelow: (label) => invoke("popup_close_below", { label }),
  moveHere: (label, path) => invoke("move_here", { label, path }),
  key: (key) => invoke("menu_key", { key }),
  ready: (label) => invoke("popup_ready", { label }),
};

export const ipc = {
  hello: (label: string) => invoke<PopupCfg | null>("popup_hello", { label }),
  dialogResolve: (choice: string) => invoke<void>("dialog_resolve", { choice }),
  sessionClose: () => invoke<void>("session_close"),
  getState: () => invoke<AppState>("get_state"),
  saveSettings: (s: Settings) => invoke<SaveResult>("save_settings", { new: s }),
  pickFolder: () => invoke<string | null>("pick_folder"),
  pauseShortcut: (paused: boolean) => invoke<void>("pause_shortcut", { paused }),
  setAutostart: (enabled: boolean) => invoke<boolean>("set_autostart", { enabled }),
  windowReady: (label: string) => invoke<void>("window_ready", { label }),
  windowClose: (label: string) => invoke<void>("window_close", { label }),
};
