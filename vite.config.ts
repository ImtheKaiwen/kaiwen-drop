import { defineConfig } from "vitest/config";

// Cok sayfali yapi: popup, settings, drop. Tauri frontendDist = dist.
export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    target: "chrome120",
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        popup: "popup.html",
        settings: "settings.html",
        drop: "drop.html",
      },
    },
  },
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts"],
  },
});
