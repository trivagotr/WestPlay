import { test } from "node:test";
import assert from "node:assert/strict";
import { vehicleView, convoyView, DiagnosticLog } from "./dashboard.js";
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

test("automatic convoy view clears stale positions and preserves only bounded display names", () => {
  const sample = {fresh:true, status:"observing",players:[
    {marker:1,distance:45,name:"Alice <img src=x>"},
    {marker:2,distance:80},
    {marker:3,distance:90,name:"\u202EAl\u0000i"},
    {marker:4,distance:NaN,name:"invalid"},
  ]};
  assert.deepEqual(convoyView(sample).players.map(p=>p.name), ["Alice <img src=x>","İsim bekleniyor","Ali"]);
  assert.equal(convoyView({...sample,fresh:false}).players.length,0);
  assert.equal(convoyView({}).status,"Otomatik bağlantı bekleniyor");
  assert.equal(convoyView({...sample,players:[{distance:1,name:"a".repeat(1000)}]}).players[0].name.length,80);
});
