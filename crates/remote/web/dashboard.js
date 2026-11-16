import {
  icon,
  applyIndicator,
  mountSymbols,
  passengerView,
  applyLaneIndicator,
} from "./instruments.js";
const finite = (value, min = 0, max = Infinity) =>
  typeof value === "number" &&
  Number.isFinite(value) &&
  value >= min &&
  value <= max
    ? value
    : null;
const flag = (value) => (typeof value === "boolean" ? value : null);
export function vehicleView(snapshot = {}) {
  const v =
    snapshot.connected === true && finite(snapshot.ageMs, 0, 2000) !== null
      ? snapshot.vehicle || {}
      : {};
  return {
    name: typeof v.name === "string" ? v.name.slice(0, 100) : "",
    coach: flag(v.coach),
    paused: flag(v.paused),
    cruise: flag(v.cruise),
    laneAssist: flag(v.laneAssist),
    coachTelemetry:
      v.coachTelemetry && typeof v.coachTelemetry === "object"
        ? v.coachTelemetry
        : null,
    parkingBrake: flag(v.parkingBrake),
    engine: flag(v.engine),
    electric: flag(v.electric),
    lowBeam: flag(v.lowBeam),
    highBeam: flag(v.highBeam),
    leftBlinker: flag(v.leftBlinker),
    rightBlinker: flag(v.rightBlinker),
    speedKph: finite(v.speedKph, 0, 1000),
    cruiseSpeedKph: finite(v.cruiseSpeedKph, 0, 300),
    fuelConsumption: finite(v.fuelConsumption, 0, 1000),
    totalDamage:
      finite(v.totalDamage, 0, 1) === null ? null : v.totalDamage * 100,
    cargoDamage:
      finite(v.cargoDamage, 0, 1) === null ? null : v.cargoDamage * 100,
    job: {
      active: flag(v.job?.active),
      destination:
        typeof v.job?.destination === "string"
          ? v.job.destination
              .replace(
                /[\u0000-\u001f\u007f-\u009f\u202a-\u202e\u2066-\u2069]/gu,
                "",
              )
              .slice(0, 120)
          : "",
      scheduledArrivalMinutes: finite(
        v.job?.scheduledArrivalMinutes,
        0,
        4294967295,
      ),
      gameTimeMinutes: finite(v.job?.gameTimeMinutes, 0, 4294967295),
      delayMinutes: finite(v.job?.delayMinutes, -10000000, 10000000),
      passengerCount: Number.isInteger(v.job?.passengerCount)
        ? finite(v.job.passengerCount, 0, 1000)
        : null,
    },
    fuelRange: finite(v.fuelRange, 0, 100000),
    wear: Array.from({ length: 5 }, (_, i) => {
      const n = finite(v.wear?.[i], 0, 1);
      return n === null ? null : n * 100;
    }),
    warnings: Array.from({ length: 7 }, (_, i) => flag(v.warnings?.[i])),
  };
}
export class DiagnosticLog {
  constructor(limit = 512) {
    this.limit = limit;
    this.rows = [];
  }
  record(event, details = {}) {
    const safe = {};
    for (const key of [
      "connected",
      "visible",
      "samples",
      "gaps",
      "status",
      "ageMs",
      "httpStatus",
      "width",
      "height",
    ]) {
      const v = details[key];
      if (
        typeof v === "boolean" ||
        (typeof v === "number" && Number.isFinite(v)) ||
        (typeof v === "string" && /^[a-z0-9-]{1,64}$/.test(v))
      )
        safe[key] = v;
    }
    this.rows.push({ time: new Date().toISOString(), event, details: safe });
    if (this.rows.length > this.limit) this.rows.shift();
  }
  export() {
    return this.rows.slice();
  }
}
export function gameClock(minutes) {
  if (finite(minutes, 0, 4294967295) === null) return "—";
  const whole = Math.floor(minutes);
  return (
    ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"][
      Math.floor(whole / 1440) % 7
    ] +
    " " +
    String(Math.floor((whole % 1440) / 60)).padStart(2, "0") +
    ":" +
    String(whole % 60).padStart(2, "0")
  );
}
export function delayLabel(minutes) {
  if (finite(minutes, -10000000, 10000000) === null) return "—";
  if (minutes <= 0) return "Zamanında";
  const m = Math.floor(minutes);
  return (
    "+" + (m >= 60 ? Math.floor(m / 60) + " sa. " : "") + (m % 60) + " dk."
  );
}
export function journeyView(v) {
  const passenger = passengerView(v);
  const cargo = v.coach === false;
  return {
    quality: {
      label: cargo ? "YÜK HASARI" : "ORTALAMA MEMNUNİYET",
      percent: cargo
        ? v.job.active === false
          ? null
          : v.cargoDamage
        : v.job.active === false ? null : passenger.percent,
      kind: cargo ? "cargo" : "coach",
    },
    status:
      v.coach === null
        ? "Araç bağlantısı bekleniyor"
        : cargo
          ? v.job.active === false
            ? "Aktif yük işi bekleniyor"
            : v.cargoDamage === null
              ? "Yük verisi bekleniyor"
              : "Yük telemetrisi canlı"
          : {
              live: "Yolcu telemetrisi canlı",
              plugin: "Yolcu bağlantısı bekleniyor",
              unsupported: "Yolcu kanalı desteklenmiyor",
              waiting: "Aktif yolcu işi bekleniyor",
              vehicle: "Araç bağlantısı bekleniyor",
            }[passenger.state] || "Yolcu verisi bekleniyor",
  };
}
export function createDashboard({ request, getToken }) {
  const $ = (id) => document.getElementById(id),
    log = new DiagnosticLog();
  let active = false,
    generation = 0,
    timer = null,
    watch = null,
    lastAt = 0,
    lastSnapshot = {};
  mountSymbols(document);
  function put(id, value) {
    $(id).textContent = value;
  }
  function state(id, value, on = "Açık", off = "Kapalı") {
    put(id, value === null ? "Veri yok" : value ? on : off);
    const element = $(id),
      panel = element.closest("[data-indicator]");
    if (panel)
      applyIndicator(
        panel,
        panel.dataset.indicator,
        value,
        panel.dataset.label,
        on,
        off,
      );
  }
  function render(snapshot = {}) {
    lastSnapshot = snapshot;
    const v = vehicleView(snapshot);
    const connected =
      snapshot.connected === true &&
      finite(snapshot.ageMs, 0, 2000) !== null &&
      snapshot.vehicle != null;
    put("vehicle-name", v.name || "Araç bağlantısı bekleniyor");
    put(
      "vehicle-kind",
      v.coach === null
        ? "OYUN VERİLERİ"
        : v.coach
          ? "COACHES · OTOBÜS"
          : "ÇEKİCİ · ARAÇ DURUMU",
    );
    put(
      "data-state",
      connected
        ? v.paused
          ? "Oyun duraklatıldı"
          : "Veri canlı"
        : "Veri bekleniyor",
    );
    $("data-state").dataset.live = String(connected);
    for (const key of [
      "cruise",
      "parkingBrake",
      "engine",
      "electric",
      "lowBeam",
      "highBeam",
    ])
      state(
        "state-" + key,
        v[key],
        key === "parkingBrake" ? "Çekili" : "Açık",
        key === "parkingBrake" ? "Serbest" : "Kapalı",
      );
    applyLaneIndicator(
      $("state-laneAssist").closest("[data-indicator]"),
      v.laneAssist,
      v.speedKph,
    );
    put(
      "state-laneAssist",
      v.speedKph === null || v.speedKph < 65
        ? "65 km/sa altında"
        : v.laneAssist === true
          ? "Etkin"
          : "Hazır",
    );
    put(
      "cruise-value",
      v.cruise === true && v.cruiseSpeedKph !== null && v.cruiseSpeedKph > 0
        ? Math.round(v.cruiseSpeedKph) + " km/sa"
        : "",
    );
    put(
      "fuel-value",
      v.fuelConsumption === null
        ? "—"
        : v.fuelConsumption.toLocaleString("tr-TR", {
            minimumFractionDigits: 1,
            maximumFractionDigits: 1,
          }) + " L/100 km",
    );
    put(
      "range-value",
      v.fuelRange === null
        ? "—"
        : Math.round(v.fuelRange).toLocaleString("tr-TR") + " km",
    );
    put(
      "damage-value",
      v.totalDamage === null ? "—" : "%" + v.totalDamage.toFixed(1),
    );
    $("damage-bar").style.width = (v.totalDamage ?? 0) + "%";
    $("damage-meter").dataset.alert = String(
      v.totalDamage !== null && v.totalDamage >= 10,
    );
    if (v.totalDamage !== null)
      $("damage-meter").setAttribute("aria-valuenow", String(v.totalDamage));
    else $("damage-meter").removeAttribute("aria-valuenow");
    const labels = [
      "Yakıt düşük",
      "AdBlue düşük",
      "Fren havası düşük",
      "Fren havası kritik",
      "Yağ basıncı düşük",
      "Motor sıcaklığı yüksek",
      "Akü gerilimi düşük",
    ];
    const list = $("warning-list");
    list.replaceChildren();
    const warningIcons = [
      "fuel",
      "adblue",
      "air",
      "air",
      "oil",
      "temperature",
      "battery",
    ];
    v.warnings.forEach((value, i) => {
      const li = document.createElement("li");
      li.className = "telltale warning-telltale";
      li.setAttribute("role", "img");
      li.tabIndex = 0;
      const glyph = document.createElement("span");
      glyph.className = "telltale-symbol";
      glyph.innerHTML = icon(warningIcons[i]);
      li.append(glyph);
      applyIndicator(
        li,
        i === 3 ? "airEmergency" : warningIcons[i],
        value,
        labels[i],
        "Uyarı",
        "Normal",
      );
      list.append(li);
    });
    const activeWarnings = v.warnings.filter((v) => v === true).length,
      knownWarnings = v.warnings.filter((v) => v !== null).length;
    put(
      "warning-note",
      activeWarnings
        ? activeWarnings + " etkin uyarı"
        : knownWarnings
          ? "Etkin uyarı yok" +
            (knownWarnings < 7 ? " · bazı kanallar bekleniyor" : "")
          : "Uyarı verisi bekleniyor",
    );
    const trip = journeyView(v),
      quality = trip.quality;
    put("trip-title", v.coach === false ? "Yük / teslimat" : "Yolculuk");
    put("trip-quality-caption", quality.label);
    put(
      "trip-quality-value",
      quality.percent === null ? "—" : "%" + Math.round(quality.percent),
    );
    $("trip-quality").dataset.kind = quality.kind;
    $("trip-quality").dataset.alert = String(
      quality.kind === "cargo" && quality.percent > 0,
    );
    $("trip-quality-bar").style.width = (quality.percent ?? 0) + "%";
    if (quality.percent !== null)
      $("trip-quality-meter").setAttribute(
        "aria-valuenow",
        String(quality.percent),
      );
    else $("trip-quality-meter").removeAttribute("aria-valuenow");
    $("trip-quality-meter").setAttribute("aria-label", quality.label);
    $("trip-passengers").hidden = v.coach !== true;
    put(
      "trip-passenger-count",
      v.job.passengerCount === null
        ? "—"
        : v.job.passengerCount.toLocaleString("tr-TR"),
    );
    put("trip-destination", v.job.destination || "—");
    put(
      "trip-schedule-caption",
      v.coach === false ? "Teslim sonu" : "Planlanan varış",
    );
    put("trip-schedule", gameClock(v.job.scheduledArrivalMinutes));
    put("trip-delay", delayLabel(v.job.delayMinutes));
    $("trip-delay-row").dataset.late = String(v.job.delayMinutes > 0);
    put("trip-status", trip.status + (v.coach === true && (!v.job.destination || v.job.passengerCount === null || v.job.scheduledArrivalMinutes === null) ? " · Sefer ayrıntıları bekleniyor" : ""));
  }
  async function poll(epoch) {
    if (!active || epoch !== generation) return;
    try {
      const s = await request("/dashboard", { token: getToken() });
      if (!active || epoch !== generation) return;
      lastAt = performance.now();
      render(s);
      log.record("snapshot", {
        connected: s.connected,
        ageMs: s.ageMs,
        visible: s.convoy?.players?.length || 0,
        samples: s.convoy?.samples,
        gaps: s.convoy?.gaps,
        status: s.convoy?.status,
      });
    } catch (error) {
      if (epoch !== generation) return;
      render();
      log.record("snapshot-error", { httpStatus: error.status });
      if (error.status === 401) {
        stop();
        put("data-state", "Eşleşme sona erdi");
        return;
      }
    }
    if (active && epoch === generation)
      timer = setTimeout(() => poll(epoch), 500);
  }
  function start() {
    stop();
    active = true;
    const epoch = ++generation;
    lastAt = performance.now();
    log.record("connected", { width: innerWidth, height: innerHeight });
    poll(epoch);
    watch = setInterval(() => {
      if (performance.now() - lastAt > 2000) render();
    }, 250);
  }
  function stop() {
    active = false;
    generation++;
    clearTimeout(timer);
    clearInterval(watch);
    render();
  }
  $("diagnostics").onclick = async () => {
    const button = $("diagnostics");
    button.disabled = true;
    try {
      const native = await request("/diagnostics", { token: getToken() });
      const data = {
        ...native,
        browser: log.export(),
        summary: {
          connected: lastSnapshot.connected === true,
          viewport: { width: innerWidth, height: innerHeight },
        },
      };
      const url = URL.createObjectURL(
        new Blob([JSON.stringify(data, null, 2)], { type: "application/json" }),
      );
      const link = document.createElement("a");
      link.href = url;
      link.download =
        "WestPlay-web-log-" +
        new Date().toISOString().replace(/[:.]/g, "-") +
        ".json";
      link.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    } catch (error) {
      put("connection-error", "Log indirilemedi; bağlantıyı kontrol et");
      log.record("export-error", { httpStatus: error.status });
    } finally {
      button.disabled = false;
    }
  };
  $("focus-mode").onclick = () => {
    const focused = $("display").classList.toggle("focus-mode");
    $("focus-mode").setAttribute("aria-pressed", String(focused));
    $("focus-mode").textContent = focused
      ? "Bilgileri göster"
      : "Bilgileri gizle";
  };
  render();
  return { start, stop, render };
}
