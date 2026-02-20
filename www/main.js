import init from "./pkg/spiral_wasm.js";

async function boot() {
  await init();
}

boot();
