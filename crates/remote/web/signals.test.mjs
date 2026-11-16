import test from "node:test";
import assert from "node:assert/strict";
import { createVehicleSignals } from "./signals.js";
test("lamps follow exact game states without a synthetic blink", () => {
  const seen = [];
  const s = createVehicleSignals((l, r) => seen.push([l, r]));
  assert.equal(
    s.accept(
      {
        type: "vehicleSignals",
        ageMs: 0,
        sentAt: 100,
        left: true,
        right: false,
      },
      120,
    ),
    true,
  );
  s.expire(400);
  assert.deepEqual(seen.at(-1), [true, false]);
  s.accept(
    { type: "vehicleSignals", ageMs: 0, sentAt: 450, left: false, right: true },
    460,
  );
  assert.deepEqual(seen.at(-1), [false, true]);
});
test("stale or out-of-order packets never replay a lamp", () => {
  const seen = [];
  const s = createVehicleSignals((l, r) => seen.push([l, r]));
  s.accept(
    { type: "vehicleSignals", ageMs: 0, sentAt: 100, left: true, right: false },
    120,
  );
  assert.equal(
    s.accept(
      {
        type: "vehicleSignals",
        ageMs: 0,
        sentAt: 90,
        left: false,
        right: true,
      },
      130,
    ),
    false,
  );
  s.expire(871);
  assert.deepEqual(seen.at(-1), [null, null]);
  assert.equal(
    s.accept(
      {
        type: "vehicleSignals",
        ageMs: 0,
        sentAt: 110,
        left: true,
        right: true,
      },
      900,
    ),
    false,
  );
  assert.deepEqual(seen.at(-1), [null, null]);
});
test("disconnect invalidates samples and malformed or future data is ignored", () => {
  const seen = [];
  const s = createVehicleSignals((l, r) => seen.push([l, r]));
  assert.equal(
    s.accept(
      {
        type: "vehicleSignals",
        ageMs: 0,
        sentAt: 200,
        left: true,
        right: false,
      },
      100,
    ),
    false,
  );
  assert.equal(
    s.accept(
      { type: "vehicleSignals", ageMs: 0, sentAt: 50, left: 1, right: false },
      100,
    ),
    false,
  );
  s.accept(
    { type: "vehicleSignals", ageMs: 0, sentAt: 50, left: true, right: false },
    100,
  );
  s.reset();
  assert.deepEqual(seen.at(-1), [null, null]);
});
test("source age and transport delay share the stale budget", () => {
  const seen = [];
  const s = createVehicleSignals((l, r) => seen.push([l, r]));
  assert.equal(
    s.accept(
      {
        type: "vehicleSignals",
        ageMs: 600,
        sentAt: 1000,
        left: true,
        right: false,
      },
      1100,
    ),
    true,
  );
  s.expire(1151);
  assert.deepEqual(seen.at(-1), [null, null]);
  assert.equal(
    s.accept(
      {
        type: "vehicleSignals",
        ageMs: 700,
        sentAt: 1200,
        left: true,
        right: false,
      },
      1300,
    ),
    false,
  );
});
