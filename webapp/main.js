import init, { run_app } from "./static/pkg/webapp.js";
async function main() {
  await init("/pkg/webapp_bg.wasm?CACHEBUSTER");
  run_app();
}
main();
