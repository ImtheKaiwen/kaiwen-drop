import type { Dict } from "../i18n";
import { h } from "../lib/dom";
import type { Folder, MenuApi, MenuCfg } from "../lib/ipc";
import { nextIndex } from "../lib/nav";
import { actionRow, folderRow } from "./FolderRow";

const DWELL_MS = 130; // alt menu acilmadan once kisa niyet gecikmesi

type Entry = { type: "action" } | { type: "folder"; item: Folder };

/**
 * Tek bir popup yuzeyi: satirlar, hover niyeti, klavye, alt menu istekleri.
 * Pencere/odak yasam dongusunu Rust (menu_session) yonetir; burasi yalniz goruntu ve etkilesim.
 */
export class MenuSurface {
  readonly surface: HTMLElement;
  private rows: HTMLElement[] = [];
  private entries: Entry[] = [];
  private active = -1;
  private openIdx = -1;
  private timer: number | undefined;
  private listEl: HTMLElement;

  constructor(
    private root: HTMLElement,
    private cfg: MenuCfg,
    private api: MenuApi,
    private d: Dict,
  ) {
    this.listEl = h("div", { class: "list", role: "menu", tabindex: "-1" });
    this.surface = h("div", { class: `surface menu density-${cfg.density}` }, this.listEl);
    this.render();
    root.replaceChildren(this.surface);
    if (cfg.keyboard && this.rows.length) this.setActive(0);
  }

  get depth(): number {
    return this.cfg.depth;
  }

  private render(): void {
    const { items, moveHere } = this.cfg;
    if (moveHere) {
      this.entries.push({ type: "action" });
      this.rows.push(actionRow(this.d.moveHere));
    }
    items.forEach((item) => {
      this.entries.push({ type: "folder", item });
      this.rows.push(folderRow(item));
    });
    this.rows.forEach((row, i) => {
      row.addEventListener("mouseenter", () => this.onEnter(i));
      row.addEventListener("mouseleave", () => this.cancelTimer());
      row.addEventListener("click", () => this.activate(i, false));
      row.dataset.index = String(i);
      this.listEl.append(row);
      if (moveHere && i === 0 && this.rows.length > 1) this.listEl.append(h("div", { class: "sep", role: "separator" }));
    });
  }

  private cancelTimer(): void {
    if (this.timer !== undefined) window.clearTimeout(this.timer);
    this.timer = undefined;
  }

  private setActive(i: number): void {
    this.rows[this.active]?.classList.remove("active");
    this.active = i;
    const row = this.rows[i];
    if (row) {
      row.classList.add("active");
      row.scrollIntoView({ block: "nearest" });
    }
  }

  private setOpen(i: number): void {
    this.rows[this.openIdx]?.classList.remove("open");
    this.openIdx = i;
    this.rows[i]?.classList.add("open");
  }

  /** Uzun ad kesildiyse tam yolu tooltip olarak goster (gereksiz tooltip yok). */
  private tooltipIfTruncated(i: number): void {
    const row = this.rows[i];
    const label = row?.querySelector<HTMLElement>(".label");
    const path = row?.dataset.path;
    if (!row || !label || !path) return;
    if (label.scrollWidth > label.clientWidth) row.title = path;
    else row.removeAttribute("title");
  }

  private onEnter(i: number): void {
    this.tooltipIfTruncated(i);
    this.setActive(i);
    this.cancelTimer();
    this.timer = window.setTimeout(() => {
      const e = this.entries[i];
      if (e?.type === "folder" && e.item.has_children) void this.open(i, false);
      else if (this.openIdx !== -1) {
        this.setOpen(-1);
        void this.api.closeBelow(this.cfg.label);
      }
    }, DWELL_MS);
  }

  private rowTop(i: number): number {
    const r = this.rows[i]!.getBoundingClientRect();
    return r.top - this.surface.getBoundingClientRect().top;
  }

  private async open(i: number, keyboard: boolean): Promise<void> {
    const e = this.entries[i];
    if (e?.type !== "folder") return;
    this.setOpen(i);
    await this.api.openChild(this.cfg.label, e.item.path, this.rowTop(i), keyboard);
  }

  /** Tikla / Enter: alt klasoru varsa alt menuyu ac; yoksa (veya "Buraya tasi") tasimayi baslat. */
  private activate(i: number, keyboard: boolean): void {
    this.cancelTimer();
    const e = this.entries[i];
    if (!e) return;
    if (e.type === "action") {
      if (this.cfg.path) void this.api.moveHere(this.cfg.label, this.cfg.path);
    } else if (e.item.has_children) {
      void this.open(i, keyboard);
    } else {
      void this.api.moveHere(this.cfg.label, e.item.path);
    }
  }

  /** Rust'tan yonlendirilen klavye olayi (yalniz en derin popup'a gelir). */
  handleKey(key: string): void {
    switch (key) {
      case "ArrowDown":
      case "ArrowUp":
      case "Home":
      case "End":
        this.setActive(nextIndex(this.active, this.rows.length, key));
        break;
      case "ArrowRight": {
        const e = this.entries[this.active];
        if (e?.type === "folder" && e.item.has_children) void this.open(this.active, true);
        break;
      }
      case "Enter":
        if (this.active >= 0) this.activate(this.active, true);
        break;
    }
  }

  destroy(): void {
    this.cancelTimer();
    this.root.replaceChildren();
  }
}
