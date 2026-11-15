import { test } from "node:test";
import assert from "node:assert/strict";
import { containedPoint, localOffer, scrollDelta } from "./geometry.js";
test("portrait letterboxing never becomes a screen tap", () => {
  const rect = { left: 10, top: 20, width: 360, height: 640 };
  assert.equal(containedPoint(rect, 1280, 720, 190, 30), null);
  assert.deepEqual(containedPoint(rect, 1280, 720, 190, 340), {
    x: 0.5,
    y: 0.5,
  });
  assert.deepEqual(containedPoint(rect, 1280, 720, 1000, 1000, true), {
    x: 1,
    y: 1,
  });
  assert.equal(containedPoint(rect, 0, 0, 190, 340), null);
});
test("an upward finger swipe scrolls content down without a held mouse button", () => {
  assert.equal(scrollDelta({ y: 0.6 }, { y: 0.4 }, 720), 144);
  assert.equal(scrollDelta({ y: 0.4 }, { y: 0.6 }, 720), -144);
  assert.equal(scrollDelta({ y: 1 }, { y: 0 }, 4000), 1200);
});
test("SDP leaves only private IPv4 or mDNS UDP host candidates", () => {
  const sdp =
    "v=0\r\na=candidate:1 1 udp 1 192.168.1.2 123 typ host\r\na=candidate:2 1 udp 1 8.8.8.8 456 typ srflx\r\na=candidate:3 1 udp 1 foo.local 789 typ host\r\na=candidate:4 1 udp 1 10.0.0.1 123 typ relay\r\na=candidate:5 1 tcp 1 10.0.0.1 9 typ host\r\n";
  const result = localOffer({ type: "offer", sdp });
  assert.match(result.sdp, /192\.168\.1\.2/);
  assert.match(result.sdp, /foo\.local/);
  assert.doesNotMatch(result.sdp, /8\.8\.8\.8|typ relay| tcp /);
});

test("fill mode maps the whole stretched surface without letterbox gaps", () => {
  const rect = { left: 10, top: 20, width: 900, height: 300 };
  assert.deepEqual(containedPoint(rect, 1280, 720, 10, 20, false, "fill"), {
    x: 0,
    y: 0,
  });
  assert.deepEqual(containedPoint(rect, 1280, 720, 910, 320, false, "fill"), {
    x: 1,
    y: 1,
  });
  assert.deepEqual(containedPoint(rect, 1280, 720, 235, 245, false, "fill"), {
    x: 0.25,
    y: 0.75,
  });
  assert.equal(containedPoint(rect, 1280, 720, 0, 100, false, "fill"), null);
  assert.deepEqual(containedPoint(rect, 1280, 720, 1000, -20, true, "fill"), {
    x: 1,
    y: 0,
  });
});
test("portrait fill and contain use different touch regions and require a video frame", () => {
  const rect = { left: 0, top: 0, width: 360, height: 640 };
  assert.deepEqual(containedPoint(rect, 1280, 720, 90, 160, false, "fill"), {
    x: 0.25,
    y: 0.25,
  });
  assert.equal(
    containedPoint(rect, 1280, 720, 90, 160, false, "contain"),
    null,
  );
  assert.equal(containedPoint(rect, 0, 0, 90, 160, false, "fill"), null);
});
