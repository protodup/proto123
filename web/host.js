// Thin host (AGENTS rule 11): initialise WASM, forward raw events, update #live.
const grid = document.getElementById("grid");
const live = document.getElementById("live");

function gridBox() {
  const r = grid.getBoundingClientRect();
  return [Math.floor(r.width), Math.floor(r.height), window.devicePixelRatio || 1];
}

function modBits(e) {
  return (e.shiftKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.altKey ? 4 : 0) | (e.metaKey ? 8 : 0);
}

function setLive(msg) {
  if (msg) live.textContent = msg;
}

window.addEventListener("TrunkApplicationStarted", () => {
  const app = window.wasmBindings;
  const [w, h, dpr] = gridBox();
  setLive(app.start("grid", w, h, dpr));

  grid.addEventListener("keydown", (e) => {
    e.preventDefault();
    setLive(app.on_key(e.key, e.shiftKey, e.ctrlKey, e.altKey, e.metaKey, e.timeStamp));
  });

  const forwardPointer = (kind, e) => {
    const r = grid.getBoundingClientRect();
    setLive(app.on_pointer(kind, e.clientX - r.left, e.clientY - r.top, e.button, e.buttons, modBits(e), e.timeStamp));
  };

  grid.addEventListener("pointerdown", (e) => {
    grid.focus();
    if (grid.setPointerCapture) grid.setPointerCapture(e.pointerId);
    forwardPointer(0, e);
  });
  grid.addEventListener("pointermove", (e) => forwardPointer(1, e));
  grid.addEventListener("pointerup", (e) => forwardPointer(2, e));
  grid.addEventListener("pointerleave", (e) => forwardPointer(3, e));
  grid.addEventListener("wheel", (e) => {
    e.preventDefault();
    setLive(app.on_wheel(e.deltaX, e.deltaY, modBits(e), e.timeStamp));
  }, { passive: false });

  const resized = () => {
    const [cw, ch, cdpr] = gridBox();
    setLive(app.on_resize(cw, ch, cdpr, performance.now()));
  };
  new ResizeObserver(resized).observe(grid);
  window.addEventListener("resize", resized);

  grid.focus();
});
