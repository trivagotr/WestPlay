import { test } from "node:test";
import assert from "node:assert/strict";
import { inspectFixture } from "./probe.mjs";
import { makeFixture, mutateBlock } from "./fixtures/synthetic.mjs";

test("synthetic reference converts sector coordinates, excludes own truck and never confirms players", () => {
  const report = inspectFixture(makeFixture());
  assert.equal(report.status, "synthetic-candidates");
  assert.equal(report.liveAccess, false);
  assert.equal(report.verifiedForLiveGame, false);
  assert.equal(report.excludedOwn, 1);
  assert.equal(report.candidates.length, 2);
  assert.equal(report.candidates[0].x, 3586.5);
  assert.equal(report.candidates[0].z, -4097.25);
  assert.equal(report.candidates[0].classification, "unverified-vehicle");
  assert.equal(report.onlinePlayers, undefined);
  assert.doesNotMatch(JSON.stringify(report), /0x3000|0x4000|0x5000/);
});
test("live-tagged and unsupported-reference inputs are rejected before memory-shaped data is read", () => {
  const live = {
    schema: "westplay-convoy-synthetic-v1",
    synthetic: false,
    gameVersion: "1.62.3.3",
    get segments() {
      throw Error("must not read");
    },
  };
  assert.equal(inspectFixture(live).reason, "synthetic-only");
  const wrong = makeFixture();
  wrong.reference = "windows-1.62.3.3";
  assert.equal(inspectFixture(wrong).reason, "unsupported-reference");
});
test("single-player and disconnected frames contain no stale candidates", () => {
  for (const frameState of ["single-player", "disconnected"]) {
    const fixture = makeFixture();
    fixture.frameState = frameState;
    fixture.segments = [];
    const result = inspectFixture(fixture);
    assert.equal(result.status, "no-candidates");
    assert.deepEqual(result.candidates, []);
  }
});
test("zero or duplicate signatures refuse to guess an address", () => {
  const none = makeFixture();
  mutateBlock(none, "0x10001000", (b) => b.fill(0));
  assert.equal(inspectFixture(none).reason, "signature-not-unique");
  const duplicate = makeFixture();
  mutateBlock(duplicate, "0x10001000", (b) => b.copy(b, 64, 16, 41));
  assert.equal(inspectFixture(duplicate).reason, "signature-not-unique");
});
test("invalid array counts and unmapped pointer chains fail closed without partial candidates", () => {
  const huge = makeFixture();
  mutateBlock(huge, "0x20000000", (b) => b.writeBigUInt64LE(100000n, 0x218));
  assert.equal(inspectFixture(huge).reason, "array-bounds");
  const bad = makeFixture();
  mutateBlock(bad, "0x20001000", (b) => b.writeBigUInt64LE(0x77770000n, 16));
  const result = inspectFixture(bad);
  assert.equal(result.reason, "unmapped");
  assert.deepEqual(result.candidates, []);
});
test("NaN positions and malformed rotations never become map candidates", () => {
  const nan = makeFixture();
  mutateBlock(nan, "0x30001000", (b) => b.writeFloatLE(NaN, 0x28));
  assert.equal(inspectFixture(nan).reason, "invalid-placement");
  const rotation = makeFixture();
  mutateBlock(rotation, "0x30001000", (b) => b.writeFloatLE(0, 0x38));
  assert.equal(inspectFixture(rotation).reason, "invalid-placement");
});
test("overlapping or oversized fixture blocks are rejected before decoding", () => {
  const overlap = makeFixture();
  overlap.segments.push({ ...overlap.segments[0] });
  assert.equal(inspectFixture(overlap).reason, "overlapping-segments");
  const huge = makeFixture();
  huge.segments[0].hex = "aa".repeat(2 * 1024 * 1024 + 1);
  assert.equal(inspectFixture(huge).reason, "fixture-size");
});

test("CLI refuses live attachment arguments without creating a process reader", async () => {
  const { spawnSync } = await import("node:child_process");
  const { fileURLToPath } = await import("node:url");
  const result = spawnSync(
    process.execPath,
    [fileURLToPath(new URL("./probe.mjs", import.meta.url)), "--pid", "1"],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 2);
  const report = JSON.parse(result.stderr);
  assert.equal(report.reason, "offline-only");
  assert.equal(report.liveAccess, false);
});
test("CLI reads a BOM-prefixed synthetic JSON file and does not echo corrupt file contents", async () => {
  const fs = await import("node:fs");
  const os = await import("node:os");
  const path = await import("node:path");
  const { spawnSync } = await import("node:child_process");
  const { fileURLToPath } = await import("node:url");
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "westplay-convoy-offline-"),
  );
  const file = path.join(directory, "synthetic.json"),
    probe = fileURLToPath(new URL("./probe.mjs", import.meta.url));
  try {
    fs.writeFileSync(file, "\uFEFF" + JSON.stringify(makeFixture()));
    const result = spawnSync(process.execPath, [probe, "--fixture", file], {
      encoding: "utf8",
    });
    assert.equal(result.status, 0);
    assert.equal(JSON.parse(result.stdout).candidates.length, 2);
    fs.writeFileSync(file, "private-example-not-a-real-secret {");
    const invalid = spawnSync(process.execPath, [probe, "--fixture", file], {
      encoding: "utf8",
    });
    assert.equal(invalid.status, 1);
    assert.doesNotMatch(invalid.stdout + invalid.stderr, /private-example/);
  } finally {
    fs.unlinkSync(file);
    fs.rmdirSync(directory);
  }
});
