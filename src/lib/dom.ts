type Child = Node | string | null | undefined | false;

/** Kucuk DOM yardimcisi: h("div", {class:"x", onclick}, child...) */
export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Record<string, unknown> = {},
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (v === undefined || v === null || v === false) continue;
    if (k.startsWith("on") && typeof v === "function") {
      el.addEventListener(k.slice(2).toLowerCase(), v as EventListener);
    } else if (k === "class") {
      el.className = String(v);
    } else if (v === true) {
      el.setAttribute(k, "");
    } else {
      el.setAttribute(k, String(v));
    }
  }
  for (const c of children) {
    if (c === null || c === undefined || c === false) continue;
    el.append(typeof c === "string" ? document.createTextNode(c) : c);
  }
  return el;
}

export function nextFrame(): Promise<void> {
  return new Promise((r) => requestAnimationFrame(() => r()));
}

/** Iki kare bekler: ilk cizim tamamlandi (beyaz webview flash'i onlenir). */
export async function painted(): Promise<void> {
  await nextFrame();
  await nextFrame();
}

export function blockBrowserChrome(): void {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
  document.addEventListener(
    "wheel",
    (e) => {
      if (e.ctrlKey) e.preventDefault();
    },
    { passive: false },
  );
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey && ["+", "-", "=", "0"].includes(e.key)) e.preventDefault();
    if (e.key === "F5" || (e.ctrlKey && e.key.toLowerCase() === "r")) e.preventDefault();
  });
}
