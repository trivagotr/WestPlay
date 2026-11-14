export function containedPoint(
  rect,
  width,
  height,
  clientX,
  clientY,
  clamp = false,
) {
  if (!(width > 0 && height > 0 && rect.width > 0 && rect.height > 0))
    return null;
  const scale = Math.min(rect.width / width, rect.height / height);
  const w = width * scale,
    h = height * scale;
  let x = (clientX - rect.left - (rect.width - w) / 2) / w;
  let y = (clientY - rect.top - (rect.height - h) / 2) / h;
  if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
  if (!clamp && (x < 0 || x > 1 || y < 0 || y > 1)) return null;
  return { x: Math.max(0, Math.min(1, x)), y: Math.max(0, Math.min(1, y)) };
}

export function localOffer(description) {
  return {
    type: description.type,
    sdp: description.sdp
      .split("\r\n")
      .filter((line) => {
        if (!line.startsWith("a=candidate:")) return true;
        const f = line.split(/\s+/),
          ip = f[4] ?? "";
        const n = ip.split(".").map(Number);
        const privateV4 =
          n.length === 4 &&
          n.every((x) => Number.isInteger(x) && x >= 0 && x <= 255) &&
          (n[0] === 10 ||
            (n[0] === 172 && n[1] >= 16 && n[1] <= 31) ||
            (n[0] === 192 && n[1] === 168));
        return (
          f[2]?.toLowerCase() === "udp" &&
          f[7] === "host" &&
          (privateV4 || ip.endsWith(".local"))
        );
      })
      .join("\r\n"),
  };
}

export function scrollDelta(previous, current, height) {
  return Math.round(
    Math.max(-1200, Math.min(1200, (previous.y - current.y) * height)),
  );
}
