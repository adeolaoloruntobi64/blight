import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

function ignoreWasmNewUrl() {
  return {
    name: "ignore-wasm-new-url",
    enforce: "pre", // must run before Vite's built-in asset-url transform
    transform(code, id) {
      if (!id.endsWith(".js")) return;
      if (!code.includes("import.meta.url")) return; // cheap early-out

      const patched = code.replace(
        /new URL\((['"])([^'"]+\.wasm)\1,\s*import\.meta\.url\)/g,
        "/* @vite-ignore */ new URL($1$2$1, import.meta.url)"
      );

      return patched !== code ? { code: patched, map: null } : undefined;
    },
  };
}

export default defineConfig({
  // Temporary config while the vite server is not hosting the actual server paths
  server: {
    host: "127.0.0.1",
    proxy: {
      "/wisp": {
        target: "ws://localhost:3000",
        ws: true,
        changeOrigin: true,
      },
    },
  },
  plugins: [
    react(),
    tailwindcss(),
    ignoreWasmNewUrl()
  ],
})