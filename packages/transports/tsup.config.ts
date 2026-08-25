import { defineConfig } from "tsup";

const baseConfig = {
    entry: {
        bare: "src/bare.ts",
        epoxy: "src/epoxy.ts",
        libcurl: "src/libcurl.ts",
    },
    format: ["esm"] as any,
    target: "es2022" as any,
    platform: "neutral" as any,
    outDir: "dist",
    tsconfig: "tsconfig.json",
    sourcemap: true,
    splitting: false,
    noExternal: [/.*/]
};

export default defineConfig([
    {
        ...baseConfig,
        clean: true,
        dts: true,
        minify: false
    },
    {
        ...baseConfig,
        clean: false,
        dts: false,
        minify: true,
        outExtension() { return { js: ".min.js" }; }
    }
]);