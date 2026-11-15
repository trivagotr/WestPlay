export function videoPoint(rect, width, height, x, y, fit = "contain") {
  if (
    ![rect.width, rect.height, width, height, x, y].every(Number.isFinite) ||
    Math.min(rect.width, rect.height, width, height) <= 0
  )
    return null;
  const scale = Math.min(rect.width / width, rect.height / height),
    w = fit === "fill" ? rect.width : width * scale,
    h = fit === "fill" ? rect.height : height * scale;
  return { x: (rect.width - w) / 2 + x * w, y: (rect.height - h) / 2 + y * h };
}
function valid(p) {
  if (
    !p ||
    !Number.isSafeInteger(p.id) ||
    !Array.isArray(p.markers) ||
    p.markers.length > 128
  )
    return false;
  if (p.clip === null) return p.markers.length === 0;
  if (
    !Array.isArray(p.clip) ||
    p.clip.length !== 4 ||
    !p.clip.every((v) => Number.isFinite(v) && v >= 0 && v <= 1)
  )
    return false;
  const [x, y, w, h] = p.clip;
  if (w <= 0 || h <= 0 || x + w > 1.000001 || y + h > 1.000001) return false;
  return p.markers.every(
    (m) =>
      Number.isInteger(m.marker) &&
      m.marker > 0 &&
      m.marker <= 128 &&
      Number.isFinite(m.x) &&
      Number.isFinite(m.y) &&
      m.x >= x &&
      m.x <= x + w &&
      m.y >= y &&
      m.y <= y + h &&
      Number.isFinite(m.distance) &&
      m.distance >= 0,
  );
}
export function createGpsOverlay({ surface, video, getFit, svg, status }) {
  const ns = "http://www.w3.org/2000/svg";
  let projection = null,
    lastAt = 0,
    lastId = -1;
  function message(text) {
    if (status) status.textContent = text;
  }
  function clear(reset = false) {
    projection = null;
    svg.replaceChildren();
    if (reset) lastId = -1;
    message("GPS konumu bekleniyor");
  }
  function draw() {
    svg.replaceChildren();
    if (!projection) return;
    if (!projection.clip) {
      message("Arkadaşlar için GPS ekranını aç");
      return;
    }
    const rect = surface.getBoundingClientRect(),
      point = (x, y) =>
        videoPoint(rect, video.videoWidth, video.videoHeight, x, y, getFit());
    const [x, y, w, h] = projection.clip,
      a = point(x, y),
      b = point(x + w, y + h);
    if (!a || !b) return;
    svg.setAttribute("viewBox", "0 0 " + rect.width + " " + rect.height);
    const defs = document.createElementNS(ns, "defs"),
      clip = document.createElementNS(ns, "clipPath"),
      clipRect = document.createElementNS(ns, "rect");
    clip.id = "gps-map-clip";
    clipRect.setAttribute("x", a.x);
    clipRect.setAttribute("y", a.y);
    clipRect.setAttribute("width", b.x - a.x);
    clipRect.setAttribute("height", b.y - a.y);
    clip.append(clipRect);
    defs.append(clip);
    svg.append(defs);
    const group = document.createElementNS(ns, "g");
    group.setAttribute("clip-path", "url(#gps-map-clip)");
    for (const marker of projection.markers) {
      const at = point(marker.x, marker.y);
      if (!at) continue;
      const circle = document.createElementNS(ns, "circle");
      circle.classList.add("gps-friend-marker");
      circle.setAttribute("cx", at.x);
      circle.setAttribute("cy", at.y);
      circle.setAttribute("r", "6");
      const title = document.createElementNS(ns, "title");
      title.textContent =
        "Arkadaş aracı " +
        marker.marker +
        " · " +
        Math.round(marker.distance) +
        " m";
      circle.append(title);
      const text = document.createElementNS(ns, "text");
      text.classList.add("gps-friend-label");
      text.setAttribute("x", at.x + 10);
      text.setAttribute("y", at.y + 4);
      text.textContent = marker.marker;
      group.append(circle, text);
    }
    svg.append(group);
    message("GPS üzerinde " + projection.markers.length + " araç");
  }
  function receive(value) {
    if (!value) {
      clear();
      return;
    }
    if (!valid(value)) {
      clear();
      return;
    }
    if (value.id <= lastId) return;
    lastId = value.id;
    projection = value;
    lastAt = performance.now();
    draw();
  }
  const resize = new ResizeObserver(draw);
  resize.observe(surface);
  video.addEventListener("loadedmetadata", draw);
  const timer = setInterval(() => {
    if (projection && performance.now() - lastAt > 350) clear();
  }, 100);
  return {
    receive,
    clear,
    redraw: draw,
    destroy() {
      clearInterval(timer);
      resize.disconnect();
      video.removeEventListener("loadedmetadata", draw);
      clear();
    },
  };
}
