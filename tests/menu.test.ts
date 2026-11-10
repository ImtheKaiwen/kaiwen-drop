import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MenuSurface } from "../src/components/MenuSurface";
import { renderDialog, renderToast } from "../src/components/MoveFeedback";
import { dict } from "../src/i18n";
import type { DialogCfg, MenuApi, MenuCfg, ToastCfg } from "../src/lib/ipc";

const d = dict("tr-TR");
const base = { label: "pw-1", theme: "light", lang: "tr-TR", density: "normal", size: [232, 120] as [number, number] };

type Calls = Record<string, unknown[][]>;
function api(): MenuApi & { calls: Calls } {
  const calls: Calls = { openChild: [], closeBelow: [], moveHere: [], key: [], ready: [] };
  const rec =
    (n: string) =>
    (...a: unknown[]) => {
      calls[n]!.push(a);
      return Promise.resolve();
    };
  return {
    calls,
    openChild: rec("openChild") as MenuApi["openChild"],
    closeBelow: rec("closeBelow") as MenuApi["closeBelow"],
    moveHere: rec("moveHere") as MenuApi["moveHere"],
    key: rec("key") as MenuApi["key"],
    ready: rec("ready") as MenuApi["ready"],
  };
}

const rootCfg = (over: Partial<MenuCfg> = {}): MenuCfg => ({
  ...base,
  kind: "menu",
  session: 1,
  depth: 0,
  moveHere: false,
  keyboard: false,
  items: [
    { name: "Dev", path: "C:\\Dev", has_children: true },
    { name: "Documents", path: "C:\\Docs", has_children: true },
    { name: "Leaf", path: "C:\\Leaf", has_children: false },
  ],
  ...over,
});

let root: HTMLElement;
beforeEach(() => {
  vi.useFakeTimers();
  document.body.innerHTML = '<div id="app"></div>';
  root = document.getElementById("app")!;
  Element.prototype.scrollIntoView = () => {};
});
afterEach(() => vi.useRealTimers());

describe("MenuSurface", () => {
  it("renders only folder rows at root; chevron only when children exist", () => {
    new MenuSurface(root, rootCfg(), api(), d);
    const rows = root.querySelectorAll(".row");
    expect(rows.length).toBe(3);
    expect(root.querySelector(".action")).toBeNull();
    expect(rows[0]!.querySelector(".tail.hint")).toBeNull(); // altklasoru var: chevron, ipucu yok
    expect(rows[2]!.querySelector(".tail.hint")).not.toBeNull(); // altklasorsuz: tasi ipucu
  });

  it("submenu has 'Buraya taşı' first, separated", () => {
    new MenuSurface(root, rootCfg({ depth: 1, moveHere: true, path: "C:\\Docs" }), api(), d);
    expect(root.querySelector(".row")!.textContent).toContain("Buraya taşı");
    expect(root.querySelector(".sep")).not.toBeNull();
  });

  it("hover dwell opens child only for folders with children; leaving cancels", () => {
    const a = api();
    new MenuSurface(root, rootCfg(), a, d);
    const rows = root.querySelectorAll<HTMLElement>(".row");
    rows[0]!.dispatchEvent(new Event("mouseenter"));
    rows[0]!.dispatchEvent(new Event("mouseleave"));
    vi.advanceTimersByTime(300);
    expect(a.calls.openChild!.length).toBe(0);
    rows[1]!.dispatchEvent(new Event("mouseenter"));
    vi.advanceTimersByTime(200);
    expect(a.calls.openChild!.length).toBe(1);
    expect(a.calls.openChild![0]![0]).toBe("pw-1");
    expect(a.calls.openChild![0]![1]).toBe("C:\\Docs");
  });

  it("click: opens submenu if children, moves if leaf", () => {
    const a = api();
    new MenuSurface(root, rootCfg(), a, d);
    const rows = root.querySelectorAll<HTMLElement>(".row");
    rows[0]!.click();
    expect(a.calls.openChild!.length).toBe(1);
    expect(a.calls.moveHere!.length).toBe(0);
    rows[2]!.click();
    expect(a.calls.moveHere![0]).toEqual(["pw-1", "C:\\Leaf"]);
  });

  it("'Buraya taşı' moves to this popup's own path", () => {
    const a = api();
    new MenuSurface(root, rootCfg({ depth: 1, moveHere: true, path: "C:\\Docs" }), a, d);
    root.querySelector<HTMLElement>(".action")!.click();
    expect(a.calls.moveHere![0]).toEqual(["pw-1", "C:\\Docs"]);
  });

  it("hovering a leaf closes the open child chain", () => {
    const a = api();
    new MenuSurface(root, rootCfg(), a, d);
    const rows = root.querySelectorAll<HTMLElement>(".row");
    rows[1]!.dispatchEvent(new Event("mouseenter"));
    vi.advanceTimersByTime(200);
    rows[2]!.dispatchEvent(new Event("mouseenter"));
    vi.advanceTimersByTime(200);
    expect(a.calls.closeBelow!.length).toBe(1);
  });

  it("keyboard: down/down/right opens child; enter on leaf moves", () => {
    const a = api();
    const m = new MenuSurface(root, rootCfg(), a, d);
    m.handleKey("ArrowDown");
    m.handleKey("ArrowDown");
    expect(root.querySelector(".row.active")!.textContent).toContain("Documents");
    m.handleKey("ArrowRight");
    expect(a.calls.openChild![0]![3]).toBe(true); // keyboard=true
    m.handleKey("ArrowDown");
    m.handleKey("Enter");
    expect(a.calls.moveHere![0]![1]).toBe("C:\\Leaf");
  });

  it("keyboard cfg highlights first row; destroy clears DOM and timers", () => {
    const m = new MenuSurface(root, rootCfg({ keyboard: true }), api(), d);
    expect(root.querySelector(".row.active")).not.toBeNull();
    m.destroy();
    expect(root.children.length).toBe(0);
  });
});

describe("MoveFeedback", () => {
  const dlg = (over: Partial<DialogCfg> = {}): DialogCfg => ({
    ...base,
    kind: "dialog",
    count: 3,
    names: ["a.txt", "b.txt", "c.txt"],
    conflicts: 0,
    dest: "Documents / Together",
    ...over,
  });

  it("confirm dialog summarises items and destination", () => {
    const choose = vi.fn();
    renderDialog(root, dlg(), d, choose);
    expect(root.textContent).toContain("3 öğe → Documents / Together");
    expect(root.textContent).toContain("a.txt, b.txt, c.txt");
    root.querySelector<HTMLElement>(".btn.primary")!.click();
    expect(choose).toHaveBeenCalledWith("move");
  });

  it("conflict dialog offers skip / rename / cancel, never overwrite", () => {
    const choose = vi.fn();
    renderDialog(root, dlg({ conflicts: 2 }), d, choose);
    const labels = [...root.querySelectorAll(".btn")].map((b) => b.textContent);
    expect(labels).toEqual(["İptal", "Atla", "Yeni adla taşı"]);
    expect(labels.some((l) => /yaz/i.test(l ?? ""))).toBe(false);
    root.querySelector<HTMLElement>(".btn.primary")!.click();
    expect(choose).toHaveBeenCalledWith("rename");
  });

  it("toast shows error style for failures", () => {
    const cfg: ToastCfg = { ...base, kind: "toast", data: { type: "result", moved: 1, total: 3, failed: 2, error: "inuse" } };
    renderToast(root, cfg, d);
    expect(root.querySelector(".ic.err")).not.toBeNull();
    expect(root.textContent).toContain("3 öğeden 1 tanesi taşındı");
  });
});
