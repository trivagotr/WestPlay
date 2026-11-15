import { test } from "node:test";
import assert from "node:assert/strict";
import { vehicleView, relativePoints, DiagnosticLog } from "./dashboard.js";
test("absent and expired fields stay unknown rather than off or zero", () => {
  assert.equal(vehicleView({ connected: false }).cruise, null);
  assert.equal(
    vehicleView({ connected: true, ageMs: 2001, vehicle: { cruise: true } })
      .cruise,
    null,
  );
  let v = vehicleView({
    connected: true,
    ageMs: 0,
    vehicle: {
      cruise: false,
      parkingBrake: true,
      fuel: 0,
      wear: [0, 0.2, null],
      warnings: [false, null, true],
    },
  });
  assert.equal(v.cruise, false);
  assert.equal(v.parkingBrake, true);
  assert.equal(v.fuel, 0);
  assert.deepEqual(v.wear, [0, 20, null, null, null]);
  assert.deepEqual(v.warnings, [false, null, true, null, null, null, null]);
});
test("relative plot has north up, adapts extent, and rejects invalid positions", () => {
  const v = relativePoints([{ marker: 1, dx: 30, dz: -40, distance: 50 }]);
  assert.equal(v.points.length, 1);
  assert.equal(v.extent, 100);
  assert.equal(v.points[0].x, 62.6);
  assert.equal(v.points[0].y, 33.2);
  assert.equal(relativePoints([{ dx: NaN, dz: 2 }]).points.length, 0);
});
test("export ring is bounded and drops credentials payloads and absolute coordinates", () => {
  const log = new DiagnosticLog(3);
  for (let i = 0; i < 5; i++)
    log.record("snapshot", {
      connected: true,
      visible: i,
      token: "secret",
      x: 12000,
      name: "private",
    });
  const rows = log.export();
  assert.equal(rows.length, 3);
  assert.equal(rows[2].details.visible, 4);
  assert.ok(!JSON.stringify(rows).includes("secret"));
  assert.ok(!JSON.stringify(rows).includes("12000"));
});
