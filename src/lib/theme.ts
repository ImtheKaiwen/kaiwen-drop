/** Ayar temasini <html data-theme> ve lang'e uygular. "system" => attr yok, CSS prefers-color-scheme belirler. */
export function applyTheme(theme: string, lang: string): void {
  const root = document.documentElement;
  if (theme === "light" || theme === "dark") root.dataset.theme = theme;
  else delete root.dataset.theme;
  root.lang = lang === "en-US" ? "en" : "tr";
}
