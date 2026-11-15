import { containedPoint, localOffer, scrollDelta } from "./geometry.js";
const $ = (id) => document.getElementById(id);
let pc = null,
  channel = null,
  token = null,
  heartbeat = null,
  pointer = null,
  pendingMove = null,
  moving = false,
  pingAt = 0,
  lastPong = 0;
let fitMode = "fill";
try {
  if (localStorage.getItem("westplay.videoFit") === "contain")
    fitMode = "contain";
} catch {
  /* Browser storage is optional. */
}
function applyFit() {
  $("video").style.objectFit = fitMode;
  $("fit").setAttribute("aria-pressed", String(fitMode === "fill"));
  $("fit").textContent = `Görüntü: ${fitMode === "fill" ? "Doldur" : "Sığdır"}`;
}
applyFit();
$("fit").onclick = () => {
  cancel();
  scrollPoint = null;
  fitMode = fitMode === "fill" ? "contain" : "fill";
  applyFit();
  try {
    localStorage.setItem("westplay.videoFit", fitMode);
  } catch {}
};
let scrollMode = false,
  scrollPoint = null;
function status(message) {
  $("status").textContent = message;
}
function problem(message) {
  $("connection-error").textContent = message;
}
function send(input) {
  if (channel?.readyState !== "open") return false;
  if (channel.bufferedAmount > 16384) {
    close();
    problem(
      "Bağlantı yavaşladı. Gecikmiş komutları göndermemek için kontrol durduruldu.",
    );
    return false;
  }
  channel.send(typeof input === "string" ? input : JSON.stringify(input));
  return true;
}
function cancel() {
  pendingMove = null;
  pointer = null;
  send({ type: "cancel" });
}
function close() {
  clearInterval(heartbeat);
  heartbeat = null;
  if (channel?.readyState === "open")
    channel.send(JSON.stringify({ type: "cancel" }));
  channel?.close();
  channel = null;
  if (pc) {
    pc.onconnectionstatechange = null;
    pc.close();
    pc = null;
  }
  pointer = null;
  pendingMove = null;
  $("video").srcObject = null;
  $("waiting").hidden = false;
  status("Bağlantı kapalı");
  $("latency").textContent = "";
}
async function request(path, body) {
  const response = await fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(16000),
  });
  if (!response.ok) {
    const error = Error(
      (await response.text()).slice(0, 300) ||
        `Bağlantı hatası (${response.status})`,
    );
    error.status = response.status;
    throw error;
  }
  return response.json();
}
async function connect() {
  close();
  problem("");
  status("Bağlanıyor…");
  const connection = new RTCPeerConnection({
    iceServers: [],
    bundlePolicy: "max-bundle",
  });
  pc = connection;
  try {
    connection.addTransceiver("video", { direction: "recvonly" });
    channel = connection.createDataChannel("control", { ordered: true });
    channel.onopen = () => {
      if (pc !== connection) return;
      lastPong = performance.now();
      heartbeat = setInterval(() => {
        if (performance.now() - lastPong > 5000) {
          close();
          problem("Bağlantı yanıt vermiyor. Yeniden bağlanmayı dene.");
          return;
        }
        pingAt = performance.now();
        send("ping");
      }, 1000);
      status("Canlı · yerel ağ");
    };
    channel.onmessage = (event) => {
      if (event.data === "pong") {
        lastPong = performance.now();
        $("latency").textContent = `RTT ${Math.round(lastPong - pingAt)} ms`;
      }
    };
    channel.onclose = () => {
      if (pc === connection) {
        cancel();
        status("Kontrol bağlantısı kapandı");
      }
    };
    connection.ontrack = (event) => {
      if (pc === connection) {
        $("video").srcObject = new MediaStream([event.track]);
        $("video")
          .play()
          .catch(() => {});
      }
    };
    connection.onconnectionstatechange = () => {
      if (pc !== connection) return;
      if (
        ["failed", "disconnected", "closed"].includes(
          connection.connectionState,
        )
      ) {
        close();
        problem(
          "Bağlantı kesildi. PC ve telefonun aynı ağda olduğunu kontrol edip yeniden bağlan.",
        );
      }
    };
    const offer = await connection.createOffer();
    await connection.setLocalDescription(offer);
    if (connection.iceGatheringState !== "complete")
      await new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          connection.removeEventListener("icegatheringstatechange", changed);
          reject(Error("Yerel ağ bağlantısı hazırlanamadı."));
        }, 8000);
        function changed() {
          if (connection.iceGatheringState === "complete") {
            clearTimeout(timeout);
            connection.removeEventListener("icegatheringstatechange", changed);
            resolve();
          }
        }
        connection.addEventListener("icegatheringstatechange", changed);
        changed();
      });
    // Closing an old peer is asynchronous on the PC. Retry only its temporary
    // ownership conflict; never replay input or retry invalid credentials.
    const offerBody = {
      token,
      description: localOffer(connection.localDescription),
    };
    const retryUntil = performance.now() + 10000;
    let answer;
    for (;;) {
      if (pc !== connection) return;
      try {
        answer = await request("/offer", offerBody);
        break;
      } catch (error) {
        if (pc !== connection) return;
        if (error.status !== 409 || performance.now() >= retryUntil)
          throw error;
        status("Önceki bağlantı kapatılıyor…");
        await new Promise((resolve) => setTimeout(resolve, 500));
      }
    }
    if (pc !== connection) return;
    await connection.setRemoteDescription(answer);
    if (pc !== connection) return;
    $("pairing").hidden = true;
    $("display").hidden = false;
    document.body.classList.add("viewing");
  } catch (error) {
    // A newer attempt owns the global connection; old failures must not close it.
    if (pc !== connection) return;
    throw error;
  }
}
$("pair-form").onsubmit = async (event) => {
  event.preventDefault();
  $("connect").disabled = true;
  $("error").textContent = "";
  try {
    if (!token)
      ({ token } = await request("/pair", { code: $("code").value.trim() }));
    await connect();
  } catch (e) {
    close();
    if (e.status === 401) token = null;
    $("error").textContent = e.message;
  } finally {
    $("connect").disabled = false;
    $("code").readOnly = !!token;
    $("connect").textContent = token ? "Tekrar bağlan" : "Bağlan ↗";
  }
};
$("reconnect").onclick = async () => {
  try {
    await connect();
  } catch (e) {
    close();
    problem(e.message);
  }
};
$("disconnect").onclick = () => {
  close();
  token = null;
  $("code").readOnly = false;
  $("code").value = "";
  $("connect").textContent = "Bağlan ↗";
  $("pairing").hidden = false;
  $("display").hidden = true;
  document.body.classList.remove("viewing");
  $("error").textContent = "Yeni eşleşme için PC’de mobil ekranı kapatıp aç.";
};
$("video").onloadeddata = () => {
  $("waiting").hidden = true;
};
const surface = $("surface");
function point(event, clamp = false) {
  return containedPoint(
    surface.getBoundingClientRect(),
    $("video").videoWidth,
    $("video").videoHeight,
    event.clientX,
    event.clientY,
    clamp,
    fitMode,
  );
}
function flushMove() {
  if (!pendingMove) return;
  const move = pendingMove;
  pendingMove = null;
  if (scrollMode) {
    const delta = scrollPoint
      ? scrollDelta(scrollPoint, move, $("video").videoHeight)
      : 0;
    scrollPoint = move;
    if (delta) send({ type: "wheel", ...move, delta });
  } else send({ type: "move", ...move });
}
surface.onpointerdown = (event) => {
  if (pointer !== null || channel?.readyState !== "open") return;
  const p = point(event);
  if (!p) return;
  event.preventDefault();
  pointer = event.pointerId;
  surface.setPointerCapture(pointer);
  scrollPoint = p;
  if (!scrollMode) send({ type: "down", ...p });
};
surface.onpointermove = (event) => {
  if (event.pointerId !== pointer) return;
  pendingMove = point(event, true);
  if (!moving) {
    moving = true;
    requestAnimationFrame(() => {
      moving = false;
      flushMove();
    });
  }
};
surface.onpointerup = (event) => {
  if (event.pointerId !== pointer) return;
  flushMove();
  const p = point(event, true);
  if (!scrollMode) {
    if (p) send({ type: "up", ...p });
    else cancel();
  }
  pointer = null;
};
surface.onpointercancel = cancel;
surface.onlostpointercapture = () => {
  if (pointer !== null) cancel();
};
surface.addEventListener(
  "wheel",
  (event) => {
    const p = point(event);
    if (!p) return;
    event.preventDefault();
    send({
      type: "wheel",
      ...p,
      delta: Math.max(-1200, Math.min(1200, event.deltaY)),
    });
  },
  { passive: false },
);
surface.oncontextmenu = (event) => event.preventDefault();
$("scroll").onclick = () => {
  cancel();
  scrollMode = !scrollMode;
  scrollPoint = null;
  $("scroll").setAttribute("aria-pressed", String(scrollMode));
  $("scroll").textContent = `Kaydırma: ${scrollMode ? "açık" : "kapalı"}`;
};
$("keyboard").onclick = () => {
  cancel();
  $("text-form").hidden = !$("text-form").hidden;
  if (!$("text-form").hidden) $("text").focus();
};
$("text-form").onsubmit = (event) => {
  event.preventDefault();
  const text = $("text").value;
  if (new TextEncoder().encode(text).length > 1024) {
    problem("Metin çok uzun; daha kısa parçalar halinde gönder.");
    return;
  }
  if (send({ type: "text", text })) $("text").value = "";
};
$("backspace").onclick = () => send({ type: "key", key: "backspace" });
$("enter").onclick = () => send({ type: "key", key: "enter" });
$("full").onclick = async () => {
  try {
    if (document.fullscreenElement) await document.exitFullscreen();
    else await $("display").requestFullscreen();
  } catch {
    problem(
      "Bu tarayıcı tam ekranı desteklemiyor. Telefonu yatay kullanabilirsin.",
    );
  }
};
document.addEventListener("visibilitychange", () => {
  if (document.hidden) close();
});
window.addEventListener("pagehide", close);
window.addEventListener("blur", cancel);
