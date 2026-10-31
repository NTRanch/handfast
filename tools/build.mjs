// build.mjs — assemble the single-file jam entry.
// cargo build (wasm32, release, pinned toolchain) -> base64 -> inject
// into web/shell.html -> web/index.html. Fails loudly on any step.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const cargo = join(homedir(), ".cargo", "bin", "cargo");

execFileSync(cargo, ["build", "--release", "--target", "wasm32-unknown-unknown"], {
  cwd: root,
  stdio: "inherit",
  env: { ...process.env, RUSTC_WRAPPER: "" },
});

const wasmPath = join(root, "target", "wasm32-unknown-unknown", "release", "handfast.wasm");
const wasm = readFileSync(wasmPath);
const b64 = wasm.toString("base64");
const shell = readFileSync(join(root, "web", "shell.html"), "utf8");
if (!shell.includes("%%WASM_B64%%")) {
  console.error("shell.html is missing the %%WASM_B64%% placeholder");
  process.exit(1);
}
const out = shell.replace("%%WASM_B64%%", b64);
writeFileSync(join(root, "web", "index.html"), out);
console.log(`web/index.html: ${(out.length / 1024).toFixed(0)} KiB total, wasm ${wasm.length} bytes`);
