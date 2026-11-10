import { describe, expect, it } from "vitest";
import { eventToAccel, formatAccel, keyName } from "../src/lib/accel";
import { nextIndex } from "../src/lib/nav";
import { applyTheme } from "../src/lib/theme";
import { dict, toastText } from "../src/i18n";
import { baseName, moveRoot } from "../src/components/SettingsView";
import type { Root } from "../src/lib/ipc";

const ev = (code: string, mods: Partial<{ ctrlKey: boolean; altKey: boolean; shiftKey: boolean; metaKey: boolean }> = {}) => ({
  code,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...mods,
});

describe("nextIndex", () => {
  it("wraps down and up", () => {
    expect(nextIndex(-1, 3, "ArrowDown")).toBe(0);
    expect(nextIndex(2, 3, "ArrowDown")).toBe(0);
    expect(nextIndex(-1, 3, "ArrowUp")).toBe(2);
    expect(nextIndex(0, 3, "ArrowUp")).toBe(2);
  });
  it("home/end and empty list", () => {
    expect(nextIndex(1, 5, "Home")).toBe(0);
    expect(nextIndex(1, 5, "End")).toBe(4);
    expect(nextIndex(0, 0, "ArrowDown")).toBe(-1);
    expect(nextIndex(2, 5, "x")).toBe(2);
  });
});

describe("accelerator", () => {
  it("uses physical key (AltGr safe) and orders modifiers", () => {
    expect(eventToAccel(ev("KeyM", { ctrlKey: true, altKey: true }))).toEqual({ kind: "ok", accel: "Ctrl+Alt+M" });
    expect(eventToAccel(ev("F5", { ctrlKey: true, shiftKey: true }))).toEqual({ kind: "ok", accel: "Ctrl+Shift+F5" });
    expect(eventToAccel(ev("Digit7", { altKey: true }))).toEqual({ kind: "ok", accel: "Alt+7" });
  });
  it("requires a modifier and ignores bare modifiers", () => {
    expect(eventToAccel(ev("KeyM"))).toEqual({ kind: "needsMod" });
    expect(eventToAccel(ev("ControlLeft", { ctrlKey: true }))).toEqual({ kind: "pending" });
    expect(eventToAccel(ev("Escape", { ctrlKey: true }))).toEqual({ kind: "unsupported" });
  });
  it("formats for humans, never technical strings", () => {
    expect(formatAccel("Ctrl+Alt+M")).toBe("Ctrl + Alt + M");
    expect(formatAccel("CommandOrControl+Shift+B")).toBe("Ctrl + Shift + B");
    expect(keyName("KeyZ")).toBe("Z");
  });
});

describe("toastText", () => {
  const tr = dict("tr-TR");
  it("reports full success, partial and failure honestly", () => {
    expect(toastText(tr, { type: "result", moved: 3, total: 3, dest: "Documents / Together" }).text).toBe("3 öğe taşındı → Documents / Together");
    const partial = toastText(tr, { type: "result", moved: 2, total: 3, failed: 1, error: "inuse" });
    expect(partial.isError).toBe(true);
    expect(partial.text).toContain("3 öğeden 2 tanesi taşındı");
    expect(toastText(tr, { type: "result", moved: 0, total: 1, failed: 1, error: "accessdenied" }).text).toContain("Erişim reddedildi");
  });
  it("never says success when nothing moved", () => {
    expect(toastText(tr, { type: "result", moved: 0, total: 2, skipped: 2 }).text).toBe("2 öğe atlandı");
    expect(toastText(tr, { type: "result", moved: 0, total: 0 }).text).toBe("Hiçbir öğe taşınmadı");
  });
  it("english + error codes", () => {
    const en = dict("en-US");
    expect(toastText(en, { type: "error", code: "intoself" }).text).toBe("A folder cannot be moved into itself");
    expect(toastText(en, { type: "result", moved: 1, total: 1, dest: "D" }).text).toBe("Moved → D");
  });
  it("hotkey failure shows human shortcut", () => {
    expect(toastText(dict("tr-TR"), { type: "info", key: "hotkeyFailed", shortcut: "Ctrl+Alt+M" }).text).toContain("Ctrl + Alt + M");
  });
});

describe("theme", () => {
  it("system removes the attribute, explicit sets it", () => {
    applyTheme("dark", "tr-TR");
    expect(document.documentElement.dataset.theme).toBe("dark");
    applyTheme("system", "en-US");
    expect(document.documentElement.dataset.theme).toBeUndefined();
    expect(document.documentElement.lang).toBe("en");
  });
});

describe("settings helpers", () => {
  const r = (id: string): Root => ({ id, name: id, path: `C:\\${id}`, known: null });
  it("moveRoot swaps and clamps", () => {
    const list = [r("a"), r("b"), r("c")];
    expect(moveRoot(list, 1, -1).map((x) => x.id)).toEqual(["b", "a", "c"]);
    expect(moveRoot(list, 2, 1)).toBe(list);
    expect(moveRoot(list, 0, -1)).toBe(list);
    expect(list.map((x) => x.id)).toEqual(["a", "b", "c"]); // girdi degismez
  });
  it("baseName handles windows paths", () => {
    expect(baseName("D:\\Archive\\Fotoğraflar\\")).toBe("Fotoğraflar");
    expect(baseName("D:\\")).toBe("D:");
  });
});
