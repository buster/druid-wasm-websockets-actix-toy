import init, { wasm_main } from "../pkg/toylib.js";

async function run() {
  await init();
  wasm_main();
}

run();
