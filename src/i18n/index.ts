import { en } from "./en";
import { tr, type Dict } from "./tr";

export type { Dict };
export type Lang = "tr-TR" | "en-US";

export function dict(lang: string): Dict {
  return lang === "en-US" ? en : tr;
}

/** Backend'den gelen toast verisini kullaniciya gosterilecek metne cevirir. */
export interface ToastData {
  type: "result" | "error" | "info";
  moved?: number;
  total?: number;
  skipped?: number;
  failed?: number;
  dest?: string;
  error?: string | null;
  code?: string;
  key?: string;
  shortcut?: string;
}

export function toastText(d: Dict, data: ToastData): { text: string; isError: boolean } {
  if (data.type === "error") {
    return { text: d.errors[data.code ?? "other"] ?? d.errors.other!, isError: true };
  }
  if (data.type === "info") {
    switch (data.key) {
      case "noRoots":
        return { text: d.info.noRoots, isError: true };
      case "unsupported":
        return { text: d.info.unsupported, isError: true };
      case "hotkeyFailed":
        return { text: d.info.hotkeyFailed(data.shortcut ?? ""), isError: true };
      default:
        return { text: "", isError: false };
    }
  }
  const moved = data.moved ?? 0;
  const total = data.total ?? 0;
  const failed = data.failed ?? 0;
  const skipped = data.skipped ?? 0;
  if (failed > 0) {
    const reason = d.errors[data.error ?? "other"] ?? d.errors.other!;
    const head = moved > 0 ? d.toast.partial(moved, total) : d.toast.failed(failed);
    return { text: `${head} — ${reason}`, isError: true };
  }
  if (moved === 0) return { text: skipped > 0 ? d.toast.skipped(skipped) : d.toast.none, isError: false };
  if (skipped > 0) return { text: `${d.toast.partial(moved, total)} · ${d.toast.skipped(skipped)}`, isError: false };
  return { text: d.toast.movedAll(moved, data.dest ?? ""), isError: false };
}
