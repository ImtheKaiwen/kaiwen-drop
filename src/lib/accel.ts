export interface KeyLike {
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "ShiftLeft",
  "ShiftRight",
  "MetaLeft",
  "MetaRight",
]);

/** `e.code` -> ana tus adi (Turkce klavyede AltGr nedeniyle e.key'e guvenilmez). */
export function keyName(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  if (code === "Space") return "Space";
  return null;
}

export type AccelResult =
  | { kind: "pending" } // yalniz degistirici basili
  | { kind: "needsMod" }
  | { kind: "ok"; accel: string }
  | { kind: "unsupported" };

/** Klavye olayindan "Ctrl+Alt+M" uretir. En az bir degistirici sart. */
export function eventToAccel(e: KeyLike): AccelResult {
  if (MODIFIER_CODES.has(e.code)) return { kind: "pending" };
  const key = keyName(e.code);
  if (!key) return { kind: "unsupported" };
  const mods: string[] = [];
  if (e.ctrlKey) mods.push("Ctrl");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (e.metaKey) mods.push("Win");
  if (mods.length === 0) return { kind: "needsMod" };
  return { kind: "ok", accel: [...mods, key].join("+") };
}

/** Kullaniciya teknik deger gostermeden: "Ctrl + Alt + M". */
export function formatAccel(accel: string): string {
  return accel
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean)
    .map((p) => {
      const l = p.toLowerCase();
      if (l === "control" || l === "commandorcontrol") return "Ctrl";
      if (l === "super" || l === "meta") return "Win";
      return p;
    })
    .join(" + ");
}
