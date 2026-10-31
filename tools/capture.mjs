// capture.mjs — render per-level preview videos from the real core.
// Loads the wasm core in node, starts demo mode for the level, steps
// 60 steps/s, samples every 6th step (10 fps x 12 s = 120 frames),
// rasterises with the shared rasteriser, and encodes with ffmpeg.
// Usage: node tools/capture.mjs [--level N] [--seed N]   (default: all)
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { rasterise, writePNG } from "./raster.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SLUG = "handfast";
const LEVELS = [1, 2, 3];
const FPS_STEPS = 6; // 60 sim steps/s -> one frame every 6 steps
const CAPTURE_FRAMES = 120; // 12 s at 10 fps

const args = process.argv.slice(2);
function argValue(name, fallback) {
  const i = args.indexOf(name);
  return i >= 0 ? parseInt(args[i + 1], 10) : fallback;
}
const onlyLevel = argValue("--level", 0);
const seed = argValue("--seed", 7);

const wasm = readFileSync(join(root, "target", "wasm32-unknown-unknown", "release", "handfast.wasm"));
const { instance } = await WebAssembly.instantiate(wasm, {});
const ex = instance.exports;
const mem = ex.memory;
if (!mem) {
  console.error("wasm module did not export memory — cannot drive the core");
  process.exit(1);
}
const decoder = new TextDecoder();
const readText = (ptr, len) => decoder.decode(new Uint8Array(mem.buffer, ptr, len).slice());
const palette = JSON.parse(readText(ex.hf_palette_ptr(), ex.hf_palette_len()));

function parseFrame(text) {
  const ops = [];
  for (const line of text.split("\n")) {
    if (!line) continue;
    const parts = line.split(" ");
    const kind = parts[0];
    if (kind === "t") {
      ops.push(["t", ...parts.slice(1, 5).map(Number), parts.slice(5).join(" ")]);
    } else {
      ops.push([kind, ...parts.slice(1).map(Number)]);
    }
  }
  return { palette, ops };
}

function encode(framesDir, outPath) {
  const attempt = (crf) => {
    execFileSync("/usr/bin/ffmpeg", [
      "-y", "-framerate", "10", "-i", join(framesDir, "f%03d.png"),
      "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", String(crf),
      "-preset", "medium", "-movflags", "+faststart", outPath,
    ], { stdio: "pipe" });
    return statSync(outPath).size;
  };
  let size = attempt(24);
  if (size > 4 * 1024 * 1024) size = attempt(30);
  return size;
}

const framesDir = join(root, "tools", ".frames");
for (const level of LEVELS) {
  if (onlyLevel && level !== onlyLevel) continue;
  ex.hf_start_demo(seed, level - 1);
  rmSync(framesDir, { recursive: true, force: true });
  mkdirSync(framesDir, { recursive: true });
  for (let frame = 0; frame < CAPTURE_FRAMES; frame++) {
    for (let s = 0; s < FPS_STEPS; s++) ex.hf_step();
    const drawn = parseFrame(readText(ex.hf_frame_ptr(), ex.hf_frame_len()));
    const px = rasterise(drawn, 480, 480);
    writePNG(join(framesDir, `f${String(frame).padStart(3, "0")}.png`), px, 480, 480);
  }
  const outPath = join(root, "docs", "previews", `level-${level}-${SLUG}.mp4`);
  mkdirSync(dirname(outPath), { recursive: true });
  const size = encode(framesDir, outPath);
  console.log(`level ${level}: ${outPath} ${(size / 1024).toFixed(0)} KiB`);
  if (size > 4 * 1024 * 1024) {
    console.error(`level ${level} video exceeds 4 MB`);
    process.exit(1);
  }
}
rmSync(framesDir, { recursive: true, force: true });
