// png.mjs — minimal PNG decode/encode for the visual harness.
//
// Why hand-rolled instead of the browser's OffscreenCanvas (the original approach): the harness
// now launches Chrome with `--no-zygote --renderer-process-limit=1` to survive this box's
// 512-task cgroup cap, and canvas-based comparison stopped returning data under those flags.
// Node ships zlib, so decoding/encoding PNGs here is deterministic, dependency-free and
// unaffected by how the browser was launched.
//
// Scope: 8-bit, non-interlaced, colour types 2 (RGB) and 6 (RGBA) — what Chrome's
// Page.captureScreenshot emits. Anything else throws rather than silently mis-reporting.

import zlib from 'node:zlib';

export function decodePng(buf) {
  if (buf.readUInt32BE(0) !== 0x89504e47) throw new Error('not a PNG');
  let offset = 8;
  let width = 0, height = 0, colorType = 0, bitDepth = 0, interlace = 0;
  const idat = [];
  while (offset < buf.length) {
    const len = buf.readUInt32BE(offset);
    const type = buf.toString('ascii', offset + 4, offset + 8);
    const data = buf.subarray(offset + 8, offset + 8 + len);
    if (type === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      bitDepth = data[8];
      colorType = data[9];
      interlace = data[12];
    } else if (type === 'IDAT') {
      idat.push(data);
    } else if (type === 'IEND') {
      break;
    }
    offset += 12 + len;
  }
  if (bitDepth !== 8) throw new Error(`unsupported bit depth ${bitDepth}`);
  if (interlace !== 0) throw new Error('interlaced PNG unsupported');
  const channels = colorType === 6 ? 4 : colorType === 2 ? 3 : 0;
  if (!channels) throw new Error(`unsupported colour type ${colorType}`);

  const raw = zlib.inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const out = Buffer.alloc(height * stride);
  let prev = Buffer.alloc(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const line = raw.subarray(y * (stride + 1) + 1, y * (stride + 1) + 1 + stride);
    const cur = Buffer.from(line);
    for (let x = 0; x < stride; x++) {
      const a = x >= channels ? cur[x - channels] : 0;
      const b = prev[x];
      const c = x >= channels ? prev[x - channels] : 0;
      switch (filter) {
        case 0: break;
        case 1: cur[x] = (cur[x] + a) & 0xff; break;
        case 2: cur[x] = (cur[x] + b) & 0xff; break;
        case 3: cur[x] = (cur[x] + ((a + b) >> 1)) & 0xff; break;
        case 4: {
          const p = a + b - c;
          const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
          const pr = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
          cur[x] = (cur[x] + pr) & 0xff;
          break;
        }
        default: throw new Error(`unknown PNG filter ${filter}`);
      }
    }
    cur.copy(out, y * stride);
    prev = cur;
  }
  return { width, height, channels, data: out };
}

function crc32(buf) {
  let c, crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    c = (crc ^ buf[i]) & 0xff;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    crc = (crc >>> 8) ^ c;
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(td));
  return Buffer.concat([len, td, crc]);
}

/** Encode an RGB image (data = width*height*3) as a PNG. */
export function encodePng(width, height, rgb) {
  const stride = width * 3;
  const raw = Buffer.alloc(height * (stride + 1));
  for (let y = 0; y < height; y++) {
    raw[y * (stride + 1)] = 0; // filter none
    rgb.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; ihdr[9] = 2; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', zlib.deflateSync(raw, { level: 6 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

/** Sample an RGB pixel, tolerating RGBA input and missing rows. */
function sample(img, x, y) {
  const c = img.channels;
  const i = (y * img.width + x) * c;
  return [img.data[i], img.data[i + 1], img.data[i + 2]];
}

/**
 * Compare two decodings over their overlapping area.
 * Returns { percent, diffPixels, totalPixels, maskPng, overlayPng, width, height }.
 * maskPng: red = differs, white = matches. overlayPng: upstream grey, ours red-tinted where it
 * differs — the image a vision-capable iteration can look at to see *where* the pages diverge.
 */
export function compare(up, lx, { threshold = 16 } = {}) {
  const width = Math.min(up.width, lx.width);
  const height = Math.min(up.height, lx.height);
  const mask = Buffer.alloc(width * height * 3, 255);
  const overlay = Buffer.alloc(width * height * 3, 255);
  let diff = 0;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const a = sample(up, x, y);
      const b = sample(lx, x, y);
      const differs = Math.abs(a[0] - b[0]) > threshold || Math.abs(a[1] - b[1]) > threshold || Math.abs(a[2] - b[2]) > threshold;
      const i = (y * width + x) * 3;
      // overlay: light grey = upstream, dark = upstream ink; ours is drawn red where it differs
      const upLum = Math.round((a[0] * 0.299 + a[1] * 0.587 + a[2] * 0.114));
      overlay[i] = upLum; overlay[i + 1] = upLum; overlay[i + 2] = upLum;
      if (differs) {
        diff++;
        mask[i] = 255; mask[i + 1] = 70; mask[i + 2] = 70;
        overlay[i] = 215; overlay[i + 1] = 35; overlay[i + 2] = 35;
      } else {
        mask[i] = 255; mask[i + 1] = 255; mask[i + 2] = 255;
      }
    }
  }
  return {
    percent: Number(((diff / (width * height)) * 100).toFixed(2)),
    diffPixels: diff,
    totalPixels: width * height,
    width, height,
    maskPng: encodePng(width, height, mask),
    overlayPng: encodePng(width, height, overlay),
  };
}
