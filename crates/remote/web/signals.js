// Keep the game's lamp phase. No locally generated blink or queued replay.
export function createVehicleSignals(render, staleMs = 750) {
  let lastProbe = -Infinity,
    receivedAt = -Infinity,
    expired = true;
  return {
    accept(packet, now) {
      if (
        packet?.type !== "vehicleSignals" ||
        !Number.isFinite(packet.sentAt) ||
        packet.sentAt <= lastProbe ||
        packet.sentAt > now ||
        now - packet.sentAt > staleMs ||
        !Number.isFinite(packet.ageMs) ||
        packet.ageMs < 0 ||
        packet.ageMs + now - packet.sentAt > staleMs ||
        ![packet.left, packet.right].every(
          (v) => v === null || typeof v === "boolean",
        )
      )
        return false;
      lastProbe = packet.sentAt;
      // Include source age and full RTT conservatively; late snapshots never extend a lamp.
      receivedAt = packet.sentAt - packet.ageMs;
      expired = false;
      render(packet.left, packet.right);
      return true;
    },
    expire(now) {
      if (!expired && now - receivedAt > staleMs) {
        expired = true;
        render(null, null);
      }
    },
    reset() {
      lastProbe = receivedAt = -Infinity;
      expired = true;
      render(null, null);
    },
  };
}
