import { rimraf } from 'rimraf';
import { mkdir, readFile } from 'node:fs/promises';
import { build } from 'esbuild';

// read version from package.json
const pkg = JSON.parse(await readFile('package.json'));
process.env.VANGUARD_ENGINE_VERSION = pkg.version;

const isDevelopment = process.argv.includes('--dev');

await rimraf('dist');
await mkdir('dist');

await build({
    platform: 'browser',
    format: "esm",
    sourcemap: isDevelopment,
    minify: !isDevelopment,
    entryPoints: ['./src/index.ts'],
    outfile: `./dist/engine.mjs`,
    define: {
        'process.env.VANGUARD_ENGINE_VERSION': JSON.stringify(
            process.env.VANGUARD_ENGINE_VERSION
        ),
    },
    bundle: true,
    treeShaking: true,
    metafile: isDevelopment,
    logLevel: 'info',
});