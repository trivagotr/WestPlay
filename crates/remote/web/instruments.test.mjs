import { test } from "node:test";
import assert from "node:assert/strict";
import { passengerView, indicator, icon } from "./instruments.js";
test("real satisfaction stays zero when zero, unsupported/unknown do not become happy", () => {
  assert.equal(
    passengerView({
      coach: true,
      coachTelemetry: { supported: true, satisfaction: 0 },
    }).percent,
    0,
  );
  assert.equal(
    passengerView({
      coach: true,
      coachTelemetry: { supported: true, satisfaction: 87.5 },
    }).percent,
    87.5,
  );
  assert.equal(
    passengerView({
      coach: true,
      coachTelemetry: { supported: true, satisfaction: null },
    }).state,
    "waiting",
  );
  assert.equal(
    passengerView({ coach: true, coachTelemetry: { supported: false } }).state,
    "unsupported",
  );
  assert.equal(passengerView({ coach: true }).state, "plugin");
  assert.equal(passengerView({ coach: false }).state, "not-coach");
});
test("dashboard lights use actual states and automotive colors; unknown lane is not engaged", () => {
  assert.deepEqual(indicator("parkingBrake", true), {
    state: "on",
    tone: "red",
  });
  assert.equal(indicator("highBeam", true).tone, "blue");
  assert.equal(indicator("lowBeam", true).tone, "green");
  assert.equal(indicator("cruise", false).state, "off");
  assert.equal(indicator("laneAssist", null).state, "unknown");
  for (const key of [
    "cruise",
    "laneAssist",
    "parkingBrake",
    "engine",
    "electric",
    "lowBeam",
    "highBeam",
    "leftBlinker",
    "rightBlinker",
    "fuel",
    "range",
    "oil",
    "battery",
    "temperature",
    "air",
    "adblue",
    "passengers",
    "transmission",
    "body",
    "chassis",
    "wheels",
  ]) {
    assert.ok(icon(key).includes("<svg"));
    assert.ok(!icon(key).includes("undefined"));
  }
});
