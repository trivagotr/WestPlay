// Original vector telltales inspired by common vehicle dashboard conventions.
const paths = {
  cruise:
    '<path d="M3 18a9 9 0 1 1 18 0M6 10l2 1m4-5v3m6 1-2 1M12 15l4-4"/><path d="M7 20h10m-2-3 3 3-3 3"/>',
  laneAssist:
    '<path d="M5 2 1 22M19 2l4 20M8 12l1-5h6l1 5M7 12h10v7H7ZM8 19v2m8-2v2M9 15h1m4 0h1"/>',
  parkingBrake:
    '<circle cx="12" cy="12" r="7"/><path d="M3 5a12 12 0 0 0 0 14M21 5a12 12 0 0 1 0 14M10 16V8h3a2.5 2.5 0 0 1 0 5h-3"/>',
  engine: '<path d="M6 7h11v3h4v8h-4v2H6v-3H3v-7h3ZM9 4h6M12 4v3M1 11v5"/>',
  electric: '<path d="M12 2v9M6 5a9 9 0 1 0 12 0"/>',
  lowBeam: '<path d="M14 5v14c10 0 10-14 0-14ZM3 5l7 3M3 10l7 3M3 15l7 3"/>',
  highBeam: '<path d="M14 5v14c10 0 10-14 0-14ZM2 5h8M2 10h8M2 15h8M2 19h8"/>',
  leftBlinker:
    '<path d="M2 12 11 4v5h11v6H11v5Z" fill="currentColor" stroke="none"/>',
  fuel: '<path d="M4 21V4h10v17M3 21h12M6 6h6v5H6ZM14 8h2l3 4v6a2 2 0 0 0 4 0v-7l-4-5m1 3h3"/>',
  range: '<path d="m7 2-4 20M17 2l4 20M12 3v4m0 4v3m0 4v3"/>',
  oil: '<path d="M3 11h12v8H5l-2-5H1v-4h2m3-3h6M9 7v4m6 1 5-5 2 2-5 7"/><path d="M21 16s-2 2-2 3a2 2 0 0 0 4 0c0-1-2-3-2-3Z"/>',
  battery: '<path d="M3 7h18v14H3ZM6 4h3v3m6-3h3v3M6 12h4m7-2v5m-2-2h4"/>',
  temperature:
    '<path d="M10 15V4a2 2 0 0 1 4 0v11a4 4 0 1 1-4 0ZM12 7v10M3 20l2-1 2 1m10 0 2-1 3 1M17 8h4m-4 4h4"/>',
  air: '<circle cx="12" cy="12" r="7"/><path d="M3 5a12 12 0 0 0 0 14M21 5a12 12 0 0 1 0 14M12 7v6m0 3v1"/>',
  adblue:
    '<path d="M12 2S5 10 5 14a7 7 0 0 0 14 0c0-4-7-12-7-12ZM8 16l3 2 5-6"/>',
  passengers:
    '<circle cx="9" cy="5" r="2.5"/><path d="M5 21v-7h7l3 7M8 9v6h7m-1-8v7h5M2 9v9h10"/>',
  transmission:
    '<path d="m10 3 1-2h2l1 2 3 1 2-1 2 2-1 2 1 3 2 1v2l-2 1-1 3 1 2-2 2-2-1-3 1-1 2h-2l-1-2-3-1-2 1-2-2 1-2-1-3-2-1v-2l2-1 1-3-1-2 2-2 2 1Z"/><circle cx="12" cy="12" r="4"/>',
  body: '<path d="M3 17V5h17v12ZM6 8h4v5H6Zm8 0h3v5h-3"/><circle cx="6" cy="19" r="2"/><circle cx="18" cy="19" r="2"/>',
  chassis:
    '<path d="M4 5v14M20 5v14M4 8h16M4 16h16M4 8l16 8M4 16l16-8M1 6h3M20 6h3M1 18h3M20 18h3"/>',
  wheels:
    '<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="4"/><path d="m12 3 1 5m7 9-5-3m-11 3 5-3"/>',
};
paths.rightBlinker =
  '<g transform="translate(24 0) scale(-1 1)">' + paths.leftBlinker + "</g>";
export function icon(key) {
  const path = paths[key] || paths.engine;
  return (
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
    path +
    "</svg>"
  );
}
export function indicator(key, value) {
  const colors = {
    parkingBrake: "red",
    highBeam: "blue",
    electric: "white",
    oil: "red",
    battery: "red",
    temperature: "red",
    airEmergency: "red",
    fuel: "amber",
    adblue: "amber",
    air: "amber",
  };
  return {
    state: typeof value === "boolean" ? (value ? "on" : "off") : "unknown",
    tone: colors[key] || "green",
  };
}
export function applyIndicator(
  element,
  key,
  value,
  label,
  on = "Açık",
  off = "Kapalı",
) {
  const view = indicator(key, value);
  element.dataset.state = view.state;
  element.dataset.tone = view.tone;
  const text =
    label + ": " + (view.state === "unknown" ? "Veri yok" : value ? on : off);
  element.setAttribute("aria-label", text);
  element.title = text;
}
export function laneView(active, speedKph) {
  if (
    typeof speedKph !== "number" ||
    !Number.isFinite(speedKph) ||
    speedKph < 65
  )
    return { state: "unknown", tone: "white" };
  return active === true
    ? { state: "on", tone: "green" }
    : { state: "ready", tone: "white" };
}
export function applyLaneIndicator(element, active, speedKph) {
  const view = laneView(active, speedKph);
  element.dataset.state = view.state;
  element.dataset.tone = view.tone;
  const label =
    "Şerit desteği: " +
    (view.state === "unknown"
      ? "65 km/sa altında veya hız verisi yok"
      : view.state === "on"
        ? "Etkin"
        : active === null
          ? "65 km/sa eşiği uygun; etkinlik verisi alınamıyor"
          : "Hazır");
  element.setAttribute("aria-label", label);
  element.title = label;
}
export function passengerView(vehicle = {}) {
  if (vehicle.coach === false) return { state: "not-coach", percent: null };
  if (vehicle.coach !== true) return { state: "vehicle", percent: null };
  const data = vehicle.coachTelemetry;
  if (!data) return { state: "plugin", percent: null };
  if (data.supported !== true) return { state: "unsupported", percent: null };
  const p = data.satisfaction;
  if (typeof p !== "number" || !Number.isFinite(p) || p < 0 || p > 100)
    return { state: "waiting", percent: null };
  return { state: "live", percent: p };
}
export function mountSymbols(root) {
  for (const slot of root.querySelectorAll("[data-symbol]"))
    slot.innerHTML = icon(slot.dataset.symbol);
}
