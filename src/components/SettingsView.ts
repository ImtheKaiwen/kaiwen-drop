import { dict, type Dict } from "../i18n";
import { eventToAccel, formatAccel } from "../lib/accel";
import { h } from "../lib/dom";
import { icon } from "../lib/icons";
import { ipc, type AppState, type Root, type Settings } from "../lib/ipc";
import { applyTheme } from "../lib/theme";

function svg(name: Parameters<typeof icon>[0], cls = ""): HTMLElement {
  const s = h("span", { class: "tail", style: "display:flex;align-items:center" });
  s.innerHTML = icon(name, cls);
  return s;
}

export function newRootId(): string {
  return "r-" + Math.random().toString(36).slice(2, 10);
}

/** Saf: koku bir adim yukari/asagi tasir. */
export function moveRoot(roots: Root[], idx: number, delta: -1 | 1): Root[] {
  const j = idx + delta;
  if (idx < 0 || j < 0 || j >= roots.length) return roots;
  const out = roots.slice();
  [out[idx], out[j]] = [out[j]!, out[idx]!];
  return out;
}

export function baseName(path: string): string {
  const p = path.replace(/[\\/]+$/, "");
  return p.split(/[\\/]/).pop() || p;
}

/**
 * Ayarlar gorunumu. Degisiklikler aninda (kisa debounce ile) Rust'a kaydedilir ve uygulanir.
 */
export class SettingsView {
  private state: AppState;
  private d: Dict;
  private editingId: string | null = null;
  private menuId: string | null = null;
  private recording = false;
  private shortcutError: string | null = null;
  private saveTimer: number | undefined;
  private content!: HTMLElement;

  constructor(
    private host: HTMLElement,
    state: AppState,
  ) {
    this.state = state;
    this.d = dict(state.settings.language);
  }

  private get s(): Settings {
    return this.state.settings;
  }

  mount(): void {
    document.addEventListener("click", (e) => {
      if (this.menuId && !(e.target as HTMLElement).closest(".menu-pop, .row-menu-btn")) {
        this.menuId = null;
        this.render();
      }
    });
    document.addEventListener("keydown", (e) => this.onKey(e), true);
    this.render();
  }

  /** Kayit modunu bitirir ve duraklatilan global kisayolu geri yukler. */
  private stopRecording(): void {
    if (!this.recording) return;
    this.recording = false;
    void ipc.pauseShortcut(false);
  }

  private onKey(e: KeyboardEvent): void {
    if (this.recording) {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        this.stopRecording();
        this.shortcutError = null;
        this.render();
        return;
      }
      const r = eventToAccel(e);
      if (r.kind === "ok") {
        this.stopRecording();
        this.shortcutError = null;
        this.s.shortcut = r.accel;
        this.commit(true);
      } else if (r.kind === "needsMod") {
        this.shortcutError = this.d.settings.shortcutNeedMod;
        this.render();
      } else if (r.kind === "unsupported") {
        this.shortcutError = this.d.settings.shortcutUnsupported;
        this.render();
      }
      return;
    }
    if (e.key === "Escape") {
      if (document.querySelector(".dd-pop")) return; // acik liste Esc'i kendisi yonetir
      if (this.menuId) {
        this.menuId = null;
        this.render();
      } else if (this.editingId) {
        this.editingId = null;
        this.render();
      } else {
        void ipc.windowClose("settings");
      }
    }
  }

  /** Degisikligi kaydet; kisayol hatasinda Rust eskisini korur ve hata doner. */
  private commit(immediate = false): void {
    window.clearTimeout(this.saveTimer);
    const run = async () => {
      try {
        const res = await ipc.saveSettings(this.s);
        this.state = res.state;
        this.shortcutError = res.error;
        this.d = dict(this.s.language);
        applyTheme(this.s.theme, this.s.language);
      } catch {
        this.shortcutError = this.d.settings.saveError;
      }
      this.render();
    };
    if (immediate) void run();
    else this.saveTimer = window.setTimeout(run, 150);
    this.render();
  }

  private rootInfo(id: string) {
    return this.state.roots.find((r) => r.id === id);
  }

  private render(): void {
    const t = this.d.settings;
    const scroll = this.content?.scrollTop ?? 0;
    const close = h("button", { class: "icon-btn", type: "button", "aria-label": this.d.close, onclick: () => void ipc.windowClose("settings") });
    close.innerHTML = icon("x");
    const titlebar = h("div", { class: "titlebar", "data-tauri-drag-region": true }, h("h1", {}, t.title), close);

    this.content = h(
      "div",
      { class: "content" },
      h("h2", {}, t.roots),
      this.renderRoots(),
      h("h2", {}, t.general),
      this.renderGeneral(),
      h("h2", {}, t.menuSection),
      this.renderMenuSection(),
    );
    this.host.replaceChildren(h("div", { class: "win" }, titlebar, this.content));
    this.content.scrollTop = scroll;
    if (this.editingId) this.content.querySelector<HTMLInputElement>("input.field")?.focus();
  }

  // ---------------------------------------------------------------- kokler
  private renderRoots(): HTMLElement {
    const t = this.d.settings;
    const card = h("div", { class: "card", role: "list" });
    this.s.roots.forEach((r, idx) => {
      const info = this.rootInfo(r.id);
      const path = info?.path || r.path;
      const exists = info ? info.exists : true;
      const lead = svg("folder");

      let meta: HTMLElement;
      if (this.editingId === r.id) {
        const input = h("input", { class: "field", type: "text", value: r.name, "aria-label": t.edit });
        const finish = (save: boolean) => {
          if (this.editingId !== r.id) return;
          this.editingId = null;
          const v = input.value.trim();
          if (save && v) {
            r.name = v;
            this.commit();
          } else this.render();
        };
        input.addEventListener("keydown", (e) => {
          e.stopPropagation();
          if (e.key === "Enter") finish(true);
          if (e.key === "Escape") finish(false);
        });
        input.addEventListener("blur", () => finish(true));
        meta = h("div", { class: "meta" }, input);
      } else {
        meta = h(
          "div",
          { class: "meta" },
          h("div", { class: "name", title: r.name }, r.name),
          h("div", { class: `path${exists ? "" : " missing"}`, title: path }, exists ? path : `${path} — ${t.pathMissing}`),
        );
      }

      const more = h("button", {
        class: "icon-btn row-menu-btn",
        type: "button",
        "aria-label": t.rowMenu,
        "aria-haspopup": "menu",
        onclick: (e: Event) => {
          e.stopPropagation();
          this.menuId = this.menuId === r.id ? null : r.id;
          this.render();
        },
      });
      more.innerHTML = icon("ellipsis");

      const row = h("div", { class: "root-row", role: "listitem" }, lead, meta, more);
      if (this.menuId === r.id) row.append(this.renderRowMenu(r, idx));
      card.append(row);
    });

    const add = h("button", { class: "add-btn", type: "button", onclick: () => void this.addRoot() });
    add.innerHTML = icon("plus");
    add.append(t.addRoot);
    if (this.s.roots.length === 0) {
      // Bos durum: ince bos kutu yerine aciklama + belirgin ekle dugmesi.
      add.classList.add("solo");
      const ic = h("div", { class: "empty-ic" });
      ic.innerHTML = icon("folder");
      return h("div", { class: "empty-roots" }, ic, h("div", { class: "empty-title" }, t.rootsEmpty), h("div", { class: "empty-sub" }, t.rootsEmptySub), add);
    }
    return h("div", {}, card, add);
  }

  private renderRowMenu(r: Root, idx: number): HTMLElement {
    const t = this.d.settings;
    const item = (ic: Parameters<typeof icon>[0], label: string, fn: () => void, disabled = false) => {
      const b = h("button", { type: "button", role: "menuitem", disabled, onclick: (e: Event) => { e.stopPropagation(); fn(); } });
      b.innerHTML = icon(ic);
      b.append(label);
      return b;
    };
    const close = () => (this.menuId = null);
    return h(
      "div",
      { class: "menu-pop", role: "menu" },
      item("pencil", t.edit, () => { close(); this.editingId = r.id; this.render(); }),
      item("up", t.moveUp, () => { close(); this.s.roots = moveRoot(this.s.roots, idx, -1); this.commit(true); }, idx === 0),
      item("down", t.moveDown, () => { close(); this.s.roots = moveRoot(this.s.roots, idx, 1); this.commit(true); }, idx === this.s.roots.length - 1),
      item("trash", t.remove, () => { close(); this.s.roots = this.s.roots.filter((x) => x.id !== r.id); this.commit(true); }), // yalniz kayit; diskteki klasor silinmez
    );
  }

  private async addRoot(): Promise<void> {
    const picked = await ipc.pickFolder();
    if (!picked) return;
    const norm = (p: string) => p.replace(/[\\/]+$/, "").toLowerCase();
    if (this.s.roots.some((r) => !r.known && norm(r.path) === norm(picked))) return;
    this.s.roots.push({ id: newRootId(), name: baseName(picked), path: picked, known: null });
    this.commit(true);
  }

  // ---------------------------------------------------------------- genel
  /** Ozel acilir liste: yerel <select> acilinca isletim sistemi stilinde cikiyordu. */
  private select<T extends string>(value: T, options: [T, string][], on: (v: T) => void, label: string): HTMLElement {
    const current = options.find(([v]) => v === value)?.[1] ?? "";
    const btn = h("button", { class: "dd", type: "button", "aria-label": label, "aria-haspopup": "listbox", "aria-expanded": "false" }) as HTMLButtonElement;
    btn.innerHTML = `<span>${current.replace(/</g, "&lt;")}</span>${icon("down", "dd-ic")}`;
    let pop: HTMLElement | null = null;
    const close = (focus = false) => {
      if (!pop) return;
      pop.remove();
      pop = null;
      btn.setAttribute("aria-expanded", "false");
      document.removeEventListener("mousedown", away, true);
      if (focus) btn.focus();
    };
    const away = (e: Event) => {
      if (pop && !pop.contains(e.target as Node) && !btn.contains(e.target as Node)) close();
    };
    const open = () => {
      const items = options.map(([v, text]) => {
        const it = h("button", { class: "dd-item", type: "button", role: "option", "aria-selected": String(v === value) }) as HTMLButtonElement;
        it.innerHTML = `<span>${text.replace(/</g, "&lt;")}</span>${v === value ? icon("check", "dd-check") : ""}`;
        it.addEventListener("click", () => {
          close();
          if (v !== value) on(v);
        });
        return it;
      });
      pop = h("div", { class: "dd-pop", role: "listbox" }, ...items);
      document.body.append(pop);
      const r = btn.getBoundingClientRect();
      const ph = pop.offsetHeight;
      const below = window.innerHeight - r.bottom - 8;
      const up = below < ph && r.top - 8 > below;
      pop.style.minWidth = `${Math.round(r.width)}px`;
      pop.style.left = `${Math.max(8, Math.min(r.left, window.innerWidth - pop.offsetWidth - 8))}px`;
      pop.style.top = `${Math.round(up ? r.top - ph - 4 : r.bottom + 4)}px`;
      btn.setAttribute("aria-expanded", "true");
      document.addEventListener("mousedown", away, true);
      (items.find((i) => i.getAttribute("aria-selected") === "true") ?? items[0])?.focus();
    };
    btn.addEventListener("click", () => (pop ? close() : open()));
    btn.addEventListener("keydown", (e) => {
      if (!pop && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
        e.preventDefault();
        open();
      }
    });
    // Liste acikken ok tuslari / Esc yalniz listeyi yonetir (Ayarlar'i kapatmaz).
    const onKeys = (e: KeyboardEvent) => {
        if (!btn.isConnected) {
          document.removeEventListener("keydown", onKeys, true);
          return;
        }
        if (!pop) return;
        const its = [...pop.querySelectorAll<HTMLButtonElement>(".dd-item")];
        const i = its.indexOf(document.activeElement as HTMLButtonElement);
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopImmediatePropagation();
          close(true);
        } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
          e.preventDefault();
          e.stopImmediatePropagation();
          its[(i + (e.key === "ArrowDown" ? 1 : -1) + its.length) % its.length]?.focus();
        } else if (e.key === "Tab") {
          close();
        }
    };
    document.addEventListener("keydown", onKeys, true);
    return btn;
  }

  private toggle(on: boolean, set: (v: boolean) => void, label: string): HTMLElement {
    return h("button", {
      class: "switch",
      type: "button",
      role: "switch",
      "aria-checked": String(on),
      "aria-label": label,
      onclick: () => set(!on),
    });
  }

  private row(label: string, control: HTMLElement, sub?: string): HTMLElement {
    return h(
      "div",
      { class: "setting" },
      h("div", {}, h("div", { class: "lbl" }, label), sub ? h("div", { class: "sub" }, sub) : null),
      control,
    );
  }

  private renderGeneral(): HTMLElement {
    const t = this.d.settings;
    const accel = h(
      "button",
      {
        class: `field accel${this.recording ? " rec" : ""}`,
        type: "button",
        "aria-label": t.shortcut,
        onclick: () => {
          if (!this.recording) void ipc.pauseShortcut(true);
          this.recording = true;
          this.shortcutError = null;
          this.render();
        },
        onblur: () => {
          // DOM yeniden cizilirken kaldirilan dugmenin blur'u kaydi bitirmesin: odak gercekten baska yere gittiyse bitir.
          setTimeout(() => {
            if (this.recording && !document.activeElement?.classList.contains("accel")) {
              this.stopRecording();
              this.render();
            }
          }, 0);
        },
      },
      this.recording ? t.shortcutPress : formatAccel(this.s.shortcut),
    );
    const err = this.shortcutError || this.state.hotkey_error;
    const card = h(
      "div",
      { class: "card" },
      this.row(
        t.language,
        this.select(this.s.language, [["tr-TR", "Türkçe"], ["en-US", "English"]], (v) => { this.s.language = v; this.commit(true); }, t.language),
      ),
      this.row(
        t.theme,
        this.select(this.s.theme, [["system", t.themes.system], ["light", t.themes.light], ["dark", t.themes.dark]], (v) => { this.s.theme = v; applyTheme(v, this.s.language); this.commit(true); }, t.theme),
      ),
      this.row(t.shortcut, accel),
      err ? h("div", { class: "err-text", role: "alert" }, err) : null,
      this.row(
        t.autostart,
        this.toggle(this.state.autostart, async (v) => {
          this.state.autostart = await ipc.setAutostart(v).catch(() => !v);
          this.render();
        }, t.autostart),
      ),
    );
    if (this.recording) queueMicrotask(() => card.querySelector<HTMLElement>(".accel")?.focus());
    return card;
  }

  private renderMenuSection(): HTMLElement {
    const t = this.d.settings;
    const flag = (key: "show_hidden" | "hide_dev_folders" | "confirm_move" | "corner_dock", label: string, sub?: string) =>
      this.row(label, this.toggle(this.s[key], (v) => { this.s[key] = v; this.commit(true); }, label), sub);
    return h(
      "div",
      { class: "card" },
      this.row(
        t.density,
        this.select(this.s.density, [["normal", t.densities.normal], ["compact", t.densities.compact]], (v) => { this.s.density = v; this.commit(true); }, t.density),
      ),
      flag("show_hidden", t.showHidden),
      flag("hide_dev_folders", t.hideDev, t.hideDevSub),
      flag("confirm_move", t.confirmMove, t.confirmMoveSub),
      flag("corner_dock", t.cornerDock, t.cornerDockSub),
    );
  }
}
