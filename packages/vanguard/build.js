import { spawn } from "node:child_process";
import { copyFileSync } from "node:fs";

process.chdir('../../');
const isDevelopment = process.argv.includes('--dev');

const wbgArgs = ['build', './crates/vanguard/', '--out-dir', '../../packages/vanguard/', '--target', 'web'];

if (isDevelopment) {
    wbgArgs.push('--dev');
}

const t1 = spawn('wasm-pack', wbgArgs);
t1.stdout.on('data', (data) => {
    process.stdout.write(data);
});

t1.stderr.on('data', (data) => {
    process.stderr.write(data);
});

t1.on('close', (code) => {
    console.log(`child process exited with code ${code}`);
    copyFileSync("./crates/vanguard/package.json", "./packages/vanguard/package.json");
    copyFileSync("./crates/vanguard/.gitignore", "./packages/vanguard/.gitignore");
});
