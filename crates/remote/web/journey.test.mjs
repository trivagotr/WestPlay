import { test } from "node:test";
import assert from "node:assert/strict";
import {
  vehicleView,
  journeyView,
  gameClock,
  delayLabel,
} from "./dashboard.js";
import { laneView } from "./instruments.js";
const live = (v) => vehicleView({ connected: true, ageMs: 0, vehicle: v });
test("truck replaces old passenger quality with cargo damage; empty job is unknown", () => {
  let v = live({
    coach: false,
    cargoDamage: 0,
    coachTelemetry: { supported: true, satisfaction: 99 },
    job: { active: true },
  });
  assert.deepEqual(journeyView(v).quality, {
    label: "YÜK HASARI",
    percent: 0,
    kind: "cargo",
  });
  assert.equal(
    journeyView(
      live({ coach: false, cargoDamage: 0.25, job: { active: false } }),
    ).quality.percent,
    null,
  );
  assert.equal(
    journeyView(
      live({
        coach: true,
        coachTelemetry: { supported: true, satisfaction: 52 },
      }),
    ).quality.percent,
    52,
  );
});
test("new dashboard fields validate fractions, counts, consumption and freshness", () => {
  const v = live({
    totalDamage: 0.12,
    fuelConsumption: 28.4,
    cruise: true,
    cruiseSpeedKph: 85,
    job: {
      passengerCount: 33,
      scheduledArrivalMinutes: 9035,
      delayMinutes: 255,
      destination: "Keflavík (IS)",
    },
  });
  assert.equal(v.totalDamage, 12);
  assert.equal(v.fuelConsumption, 28.4);
  assert.equal(v.cruiseSpeedKph, 85);
  assert.equal(v.job.passengerCount, 33);
  assert.equal(gameClock(v.job.scheduledArrivalMinutes), "Paz 06:35");
  assert.equal(delayLabel(255), "+4 sa. 15 dk.");
  assert.equal(delayLabel(null), "—");
  assert.equal(
    vehicleView({
      connected: true,
      ageMs: 2001,
      vehicle: { totalDamage: 0.12 },
    }).totalDamage,
    null,
  );
  assert.equal(live({ totalDamage: 2 }).totalDamage, null);
  assert.equal(live({ job: { passengerCount: 2.5 } }).job.passengerCount, null);
});
test("lane readiness is white at65, green only actual active above threshold", () => {
  assert.equal(laneView(null, 64.99).state, "unknown");
  assert.deepEqual(laneView(false, 65), { state: "ready", tone: "white" });
  assert.equal(laneView(null, 65).state, "ready");
  assert.deepEqual(laneView(true, 65), { state: "on", tone: "green" });
  assert.equal(laneView(true, null).state, "unknown");
});

test("ended bus job cannot hold the prior satisfaction",()=>{ assert.equal(journeyView(vehicleView({connected:true,ageMs:0,vehicle:{coach:true,coachTelemetry:{supported:true,satisfaction:52},job:{active:false}}})).quality.percent,null); });
