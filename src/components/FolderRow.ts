import { h } from "../lib/dom";
import { icon } from "../lib/icons";
import type { Folder } from "../lib/ipc";

function iconEl(html: string): HTMLElement {
  const s = h("span", { class: "tail" });
  s.innerHTML = html;
  return s;
}

/** Klasor satiri: ikon, ad, (alt klasor varsa) chevron; yoksa tikla = tasi ipucu. */
export function folderRow(f: Folder): HTMLElement {
  const lead = h("span", { class: "tail" });
  lead.innerHTML = icon("folder");
  lead.style.color = "var(--text)";
  const tail = f.has_children ? iconEl(icon("chevronRight")) : iconEl(icon("moveDown"));
  if (!f.has_children) tail.classList.add("hint");
  return h(
    "div",
    { class: "row folder", role: "menuitem", "data-path": f.path, "aria-haspopup": f.has_children ? "menu" : undefined },
    lead,
    h("span", { class: "label" }, f.name),
    tail,
  );
}

export function actionRow(label: string): HTMLElement {
  const lead = h("span", { class: "tail" });
  lead.innerHTML = icon("moveDown");
  lead.style.color = "var(--text)";
  return h("div", { class: "row action", role: "menuitem" }, lead, h("span", { class: "label" }, label));
}
