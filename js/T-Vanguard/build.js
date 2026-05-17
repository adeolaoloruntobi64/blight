import { spawn } from "node:child_process";
import { copyFileSync } from "node:fs";

process.chdir('../../');
const isDevelopment = process.argv.includes('--dev');

const wbgArgs = ['build', './crates/adblock-js/', '--out-dir', '../../js/T-Vanguard/', '--target', 'web'];

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
    const t2 = spawn('node', ['embed-wasm.js', './js/T-Vanguard/vanguard', 'true']);
    t2.stdout.on('data', (data) => {
        process.stdout.write(data);
    });

    t2.stderr.on('data', (data) => {
        process.stderr.write(data);
    });

    t2.on('close', (code) => {
        console.log(`child process exited with code ${code}`);
        copyFileSync("./crates/adblock-js/package.json", "./js/T-Vanguard/package.json");
        copyFileSync("./crates/adblock-js/.gitignore", "./js/T-Vanguard/.gitignore");
    });
});
