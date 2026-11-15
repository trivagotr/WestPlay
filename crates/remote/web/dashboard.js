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
    parkingBrake: flag(v.parkingBrake),
    engine: flag(v.engine),
    electric: flag(v.electric),
    lowBeam: flag(v.lowBeam),
    highBeam: flag(v.highBeam),
    leftBlinker: flag(v.leftBlinker),
    rightBlinker: flag(v.rightBlinker),
    fuel: finite(v.fuel, 0, 10000),
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
const statusText = {
  starting: "Okuyucu hazırlanıyor",
  observing: "Yakın araçlar izleniyor",
  "sample-unavailable": "Konum doğrulanamadı",
  "ownership-changed": "Kendi aracımızın eşleşmesi değişti",
  stopped: "Tarama kapalı",
  finished: "Test tamamlandı · yeniden başlatabilirsin",
  "reader-missing": "Konvoy okuyucusu pakette bulunamadı",
  "reader-start-failed": "Okuyucu başlatılamadı",
  "build-mismatch": "Oyun sürümü bu okuyucuyla uyumlu değil",
  "image-fingerprint": "Oyun dosyası desteklenmiyor",
};
export function createDashboard({ request, getToken }) {
  const $ = (id) => document.getElementById(id),
    log = new DiagnosticLog();
  let active = false,
    generation = 0,
    timer = null,
    watch = null,
    lastAt = 0,
    lastSnapshot = {},
    observing = false;
  function put(id, value) {
    $(id).textContent = value;
  }
  function state(id, value, on = "Açık", off = "Kapalı") {
    put(id, value === null ? "—" : value ? on : off);
    $(id).dataset.state = value === null ? "unknown" : value ? "on" : "off";
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
      "leftBlinker",
      "rightBlinker",
    ])
      state(
        "state-" + key,
        v[key],
        key === "parkingBrake" ? "Çekili" : "Açık",
        key === "parkingBrake" ? "Serbest" : "Kapalı",
      );
    put("fuel-value", v.fuel === null ? "—" : Math.round(v.fuel) + " L");
    put(
      "range-value",
      v.fuelRange === null
        ? "—"
        : Math.round(v.fuelRange).toLocaleString("tr-TR") + " km",
    );
    v.wear.forEach((value, i) => {
      put("wear-" + i, value === null ? "—" : "%" + value.toFixed(1));
      $("wear-bar-" + i).style.width = (value ?? 0) + "%";
      $("wear-bar-" + i).dataset.alert = String(value !== null && value >= 10);
    });
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
    let known = 0;
    v.warnings.forEach((value, i) => {
      if (value !== null) known++;
      if (value) {
        const item = document.createElement("li");
        item.textContent = labels[i];
        list.append(item);
      }
    });
    if (!list.childElementCount) {
      const item = document.createElement("li");
      item.className = "quiet";
      item.textContent = known
        ? "Etkin uyarı yok" + (known < 7 ? " · bazı kanallar alınamıyor" : "")
        : "Uyarı verisi bekleniyor";
      list.append(item);
    }
    put(
      "coach-status",
      v.coach === false ? "Otobüste kullanılabilir" : "Yolcu verisi alınamıyor",
    );
    const c = snapshot.connected === true ? snapshot.convoy || {} : {};
    const players = c.fresh === true ? c.players || [] : [];
    put("friend-count", players.length + " araç");
    put(
      "convoy-state",
      statusText[c.status] ||
        (c.status ? "Konum verisi alınamıyor · " + c.status : "Tarama kapalı"),
    );
    observing = [
      "starting",
      "observing",
      "sample-unavailable",
      "ownership-changed",
    ].includes(c.status);
    $("convoy-start").disabled = !connected || observing;
    $("convoy-stop").disabled = !observing;
    const rows = $("friend-list");
    rows.replaceChildren();
    players.forEach((point) => {
      const li = document.createElement("li");
      const label = document.createElement("span");
      label.textContent = "Araç " + point.marker;
      const distance = document.createElement("strong");
      distance.textContent =
        point.distance >= 1000
          ? (point.distance / 1000).toFixed(1) + " km"
          : Math.round(point.distance) + " m";
      li.append(label, distance);
      rows.append(li);
    });
    put(
      "sample-count",
      Number.isFinite(c.samples)
        ? c.samples + " örnek · " + (c.gaps || 0) + " eksik/boş"
        : "Henüz ölçüm yok",
    );
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
  $("convoy-start").onclick = async () => {
    const button = $("convoy-start");
    button.disabled = true;
    try {
      await request("/convoy", { token: getToken(), enabled: true });
      log.record("convoy-start");
    } catch (error) {
      put(
        "convoy-state",
        "Başlatılamadı · bağlantıyı ve oyun sürümünü kontrol et",
      );
      log.record("convoy-error", { httpStatus: error.status });
    } finally {
      button.disabled = observing;
    }
  };
  $("convoy-stop").onclick = async () => {
    try {
      await request("/convoy", { token: getToken(), enabled: false });
      log.record("convoy-stop");
    } catch (error) {
      log.record("convoy-error", { httpStatus: error.status });
    }
  };
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
      put("convoy-state", "Log indirilemedi; bağlantıyı kontrol et");
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
