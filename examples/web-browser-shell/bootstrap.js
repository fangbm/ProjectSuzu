const status = document.querySelector("#status");
const canvas = document.querySelector("#suzu-canvas");

function resizeCanvas() {
  const scale = window.devicePixelRatio || 1;
  canvas.width = Math.floor(window.innerWidth * scale);
  canvas.height = Math.floor(window.innerHeight * scale);
}

window.addEventListener("resize", resizeCanvas);
resizeCanvas();

status.textContent =
  "Web shell ready. Build the wasm target and mount the generated Project Suzu module here.";
