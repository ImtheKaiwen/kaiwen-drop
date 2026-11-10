import { toastText, type Dict } from "../i18n";
import { h } from "../lib/dom";
import { icon } from "../lib/icons";
import type { DialogCfg, ToastCfg } from "../lib/ipc";

/** Kompakt tasima onayi / cakisma diyalogu. Enter = birincil, Esc = iptal. */
export function renderDialog(
  root: HTMLElement,
  cfg: DialogCfg,
  d: Dict,
  resolve: (choice: "move" | "skip" | "rename" | "cancel") => void,
): { surface: HTMLElement; focusPrimary: () => void } {
  const hasConflict = cfg.conflicts > 0;
  const more = cfg.count - cfg.names.length;
  const names = cfg.names.join(", ") + (more > 0 ? `, ${d.and_more(more)}` : "");

  const btn = (label: string, choice: "move" | "skip" | "rename" | "cancel", primary = false) =>
    h("button", { class: `btn${primary ? " primary" : ""}`, type: "button", onclick: () => resolve(choice) }, label);

  const primary = hasConflict ? btn(d.renameMove, "rename", true) : btn(d.move, "move", true);
  const actions = hasConflict
    ? [btn(d.cancel, "cancel"), btn(d.skip, "skip"), primary]
    : [btn(d.cancel, "cancel"), primary];

  const surface = h(
    "div",
    { class: "surface dialog", role: "alertdialog", "aria-label": d.confirmTitle(cfg.count, cfg.dest) },
    h("div", { class: "d-title", title: d.confirmTitle(cfg.count, cfg.dest) }, d.confirmTitle(cfg.count, cfg.dest)),
    h("div", { class: "d-sub", title: names }, names),
    hasConflict ? h("div", { class: "d-warn" }, d.conflictNote(cfg.conflicts)) : null,
    h("div", { class: "d-actions" }, ...actions),
  );
  root.replaceChildren(surface);
  return { surface, focusPrimary: () => primary.focus() };
}

export function renderToast(root: HTMLElement, cfg: ToastCfg, d: Dict): HTMLElement {
  const { text, isError } = toastText(d, cfg.data);
  const ic = h("span", {});
  ic.innerHTML = icon(isError ? "alert" : "check", isError ? "err" : "");
  const surface = h("div", { class: "surface toast", role: "status" }, ic, h("span", { class: "msg", title: text }, text));
  root.replaceChildren(surface);
  return surface;
}
