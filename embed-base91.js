// https://github.com/Equim-chan/base91 with some changes

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

/**
 * Decode basE91 string into Uint8Array.
 *
 * @param  {String} data - basE91 string to be decoded
 * @return {Uint8Array} - decoded data
 * @api public
 */
function decodeBase91(data) {
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