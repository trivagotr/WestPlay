import { test } from "node:test";
import assert from "node:assert/strict";
import { videoPoint } from "./gps-overlay.js";
test("markers use exactly the fill and contain image geometry", () => {
  assert.deepEqual(
    videoPoint({ width: 1000, height: 1000 }, 1600, 900, 0.25, 0.25, "fill"),
    { x: 250, y: 250 },
  );
  assert.deepEqual(
    videoPoint({ width: 1000, height: 1000 }, 1600, 900, 0.25, 0.25, "contain"),
    { x: 250, y: 359.375 },
  );
  assert.equal(
    videoPoint({ width: 100, height: 100 }, 0, 0, 0.5, 0.5, "fill"),
    null,
  );
});
