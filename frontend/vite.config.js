import { defineConfig, transformWithOxc } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import fs from "fs"

function ignoreWasmNewUrl() {
    return {
        name: "ignore-wasm-new-url",
        enforce: "pre",
        transform(code, id) {
            if (!id.endsWith(".js")) return;
            if (!code.includes("import.meta.url")) return;
            const patched = code.replace(
                /new URL\((['"])([^'"]+\.wasm)\1,\s*import\.meta\.url\)/g,
                "/* @vite-ignore */ new URL($1$2$1, import.meta.url)"
            );
            return patched !== code ? { code: patched, map: null } : undefined;
        },
    };
}

function includeStr() {
    return {
        name: 'vite-plugin-include-ts-as-js',
        async load(id) {
            if (id.endsWith('?ts-to-js-str')) {
                const filePath = id.replace('?ts-to-js-str', '');
                const tsCode = fs.readFileSync(filePath, 'utf-8');
                const result = await transformWithOxc(tsCode, filePath, {
                    sourceType: 'script'
                });
                return {
                    code: `export default ${JSON.stringify(result.code)};`,
                    map: null
                };
            }
        }
    }
}

const dest = {
    target: "ws://localhost:3000",
    ws: true,
    changeOrigin: true,
};

export default defineConfig({
    server: {
        host: "127.0.0.1",
        proxy: {
            "/bare": dest,
            "/wisp": dest,
            "/wsproxy": dest,
        },
    },
    plugins: [
        react(),
        tailwindcss(),
        ignoreWasmNewUrl(),
        includeStr()
    ],
})