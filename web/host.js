// Thin host (AGENTS rule 11): initialise WASM, forward raw events, and nothing else.
const grid = document.getElementById("grid");

function gridBox() {
  const r = grid.getBoundingClientRect();
  return [Math.floor(r.width), Math.floor(r.height), window.devicePixelRatio || 1];
}

window.addEventListener("TrunkApplicationStarted", () => {
  const app = window.wasmBindings;
  const [w, h, dpr] = gridBox();
  app.start("grid", w, h, dpr);

  grid.addEventListener("keydown", (e) => {
    e.preventDefault();
    app.on_key(e.key, e.shiftKey, e.ctrlKey, e.altKey, e.metaKey, e.timeStamp);
  });

  const resized = () => {
    const [cw, ch, cdpr] = gridBox();
    app.on_resize(cw, ch, cdpr, performance.now());
  };
  new ResizeObserver(resized).observe(grid);
  window.addEventListener("resize", resized);

  grid.focus();
});
