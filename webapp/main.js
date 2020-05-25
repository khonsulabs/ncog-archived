import init, { run_app } from "/pkg/webapp.js";
async function main() {
  await init("/pkg/webapp_bg.wasm");
  run_app();
}
main();
