// https://github.com/rustwasm/wasm-pack/issues/1334
// MAKE SURE THE JS GENERATED IS BUILT FOR NODEJS ('nodejs')
import { readFile, writeFile, copyFile } from "node:fs/promises";

const table = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!#$%&()*+,./:;<=>?@[]^_`{|}~"';

/**
 * Encode data to basE91.
 *
 * @param  {Uint8Array} data - data to be encoded
 * @return {string} - basE91 encoded string
 * @api public
 */
function encodeBase91(data) {
  if (data == null) {
    throw new Error('base91: Missing data to encode.');
  }
  const len = data.length;
  let ret = '';

  let n = 0;
  let b = 0;

  for (let i = 0; i < len; i++) {
    b |= data[i] << n;
    n += 8;

    if (n > 13) {
      let v = b & 8191;
      if (v > 88) {
        b >>= 13;
        n -= 13;
      } else {
        v = b & 16383;
        b >>= 14;
        n -= 14;
      }
      ret += table[v % 91] + table[v / 91 | 0];
    }
  }

  if (n) {
    ret += table[b % 91];
    if (n > 7 || b > 90) ret += table[b / 91 | 0];
  }

  return ret;
};


// Running node <file> <actual-args>,
const baseName = process.argv[2];
const tsDecl = process.argv[3];
const content = await readFile(`${baseName}.js`, "utf8");
const wasmBinary = await readFile(`${baseName}_bg.wasm`);

const patched = content
  .replace('__wbg_init.__wbindgen_wasm_module', 'initSync.wasm_module')
  .split('\n')
  .slice(0, -28) // Last 28 lines are for async wbg_init and exports (was 22)
  .join('\n') +
`
var decodeBase91 = (data) => {
  const table = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!#$%&()*+,./:;<=>?@[]^_\`{|}~"';

  const len = data.length;
  const ret = [];

  let b = 0;
  let n = 0;
  let v = -1;

  for (let i = 0; i < len; i++) {
    const p = table.indexOf(data[i]);
    if (p === -1) continue;
    if (v < 0) {
      v = p;
    } else {
      v += p * 91;
      b |= v << n;
      n += (v & 8191) > 88 ? 13 : 14;
      do {
        ret.push(b & 0xff);
        b >>= 8;
        n -= 8;
      } while (n > 7);
      v = -1;
    }
  }

  if (v > -1) {
    ret.push((b | v << n) & 0xff);
  }

  return new Uint8Array(ret);
};
const bytes = decodeBase91('${encodeBase91(wasmBinary)}');
initSync(bytes);
`
;

await writeFile(`${baseName}-bundled.js`, patched);
if (tsDecl) {
  await copyFile(`${baseName}.d.ts`, `${baseName}-bundled.d.ts`);
}