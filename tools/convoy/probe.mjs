/** Offline research only. No process access, Windows APIs, network or game hooks. */
import { readFileSync, statSync } from "node:fs";
import { resolve, extname } from "node:path";
import { pathToFileURL } from "node:url";
import { makeFixture } from "./fixtures/synthetic.mjs";

const REFERENCE = "windows-1.60-unverified";
const SOURCE_COMMIT = "c925bd965a96c58b73eaea9ca033fe0f0dc623a7";
const SIGNATURE =
  "48 8b d9 48 8b 0d ? ? ? ? 48 85 c9 74 ? 48 8b 83 ? ? ? ? 48 85 c0"
    .split(" ")
    .map((v) => (v === "?" ? null : Number.parseInt(v, 16)));
const MAX_ADDRESS = 0x7fffffffffffn;
const baseReport = () => ({
  mode: "offline-only",
  liveAccess: false,
  verifiedForLiveGame: false,
  reference: REFERENCE,
  sourceCommit: SOURCE_COMMIT,
  candidates: [],
  excludedOwn: 0,
});
class Rejected extends Error {
  constructor(reason) {
    super(reason);
    this.reason = reason;
  }
}
function refuse(reason) {
  throw new Rejected(reason);
}
function address(value) {
  if (typeof value !== "string" || !/^0x[0-9a-f]{1,16}$/i.test(value))
    refuse("invalid-address");
  const at = BigInt(value);
  if (at < 0x10000n || at > MAX_ADDRESS) refuse("invalid-address");
  return at;
}
class FixtureMemory {
  constructor(segments) {
    if (!Array.isArray(segments) || !segments.length || segments.length > 64)
      refuse("fixture-format");
    let total = 0;
    this.blocks = segments
      .map((s) => {
        if (
          !s ||
          typeof s.hex !== "string" ||
          s.hex.length % 2 ||
          !s.hex.length
        )
          refuse("fixture-format");
        const size = s.hex.length / 2;
        total += size;
        if (size > 2 * 1024 * 1024 || total > 4 * 1024 * 1024)
          refuse("fixture-size");
        if (!/^[0-9a-f]+$/i.test(s.hex)) refuse("fixture-format");
        const at = address(s.address),
          end = at + BigInt(size);
        if (end > MAX_ADDRESS) refuse("invalid-address");
        return { at, end, bytes: Buffer.from(s.hex, "hex") };
      })
      .sort((a, b) => (a.at < b.at ? -1 : a.at > b.at ? 1 : 0));
    for (let i = 1; i < this.blocks.length; i++)
      if (this.blocks[i].at < this.blocks[i - 1].end)
        refuse("overlapping-segments");
  }
  read(at, size) {
    if (
      typeof at !== "bigint" ||
      at < 0x10000n ||
      at > MAX_ADDRESS ||
      !Number.isSafeInteger(size) ||
      size < 1 ||
      size > 2 * 1024 * 1024
    )
      refuse("invalid-address");
    const end = at + BigInt(size),
      block = this.blocks.find((b) => at >= b.at && end <= b.end);
    if (!block) refuse("unmapped");
    const offset = Number(at - block.at);
    return block.bytes.subarray(offset, offset + size);
  }
  u64(at) {
    return this.read(at, 8).readBigUInt64LE();
  }
  pointer(at) {
    const v = this.u64(at);
    if (v === 0n) refuse("null-pointer");
    if (v > MAX_ADDRESS || v < 0x10000n || v % 8n) refuse("invalid-address");
    return v;
  }
}
function placement(bytes) {
  const local = [0, 4, 8].map((i) => bytes.readFloatLE(i));
  const q = [16, 20, 24, 28].map((i) => bytes.readFloatLE(i));
  if (![...local, ...q].every(Number.isFinite)) refuse("invalid-placement");
  const norm = q.reduce((sum, v) => sum + v * v, 0);
  const x = local[0] + bytes.readInt16LE(12) * 512,
    z = local[2] + bytes.readInt16LE(14) * 512;
  if (
    Math.abs(norm - 1) > 0.02 ||
    Math.abs(x) > 17000000 ||
    Math.abs(z) > 17000000 ||
    Math.abs(local[1]) > 100000
  )
    refuse("invalid-placement");
  return {
    x,
    y: local[1],
    z,
    rotation: { w: q[0], x: q[1], y: q[2], z: q[3] },
  };
}
export function inspectFixture(input) {
  try {
    // No known current game builds are approved. This API only accepts fabricated fixtures.
    if (
      !input ||
      input.schema !== "westplay-convoy-synthetic-v1" ||
      input.synthetic !== true
    )
      return { ...baseReport(), status: "refused", reason: "synthetic-only" };
    if (input.reference !== REFERENCE)
      return {
        ...baseReport(),
        status: "refused",
        reason: "unsupported-reference",
      };
    if (["single-player", "disconnected"].includes(input.frameState))
      return { ...baseReport(), status: "no-candidates" };
    if (input.frameState !== "convoy") refuse("fixture-format");
    const memory = new FixtureMemory(input.segments),
      textAddress = address(input.textAddress),
      own = address(input.ownObject);
    if (
      !Number.isSafeInteger(input.textLength) ||
      input.textLength < SIGNATURE.length ||
      input.textLength > 256 * 1024
    )
      refuse("fixture-size");
    const text = memory.read(textAddress, input.textLength),
      matches = [];
    for (let i = 0; i <= text.length - SIGNATURE.length; i++) {
      if (SIGNATURE.every((v, j) => v === null || v === text[i + j]))
        matches.push(i);
      if (matches.length > 1) break;
    }
    if (matches.length !== 1)
      return {
        ...baseReport(),
        status: "refused",
        reason: "signature-not-unique",
        signatureMatches: matches.length,
      };
    const operand = matches[0] + 6;
    const globalSlot =
      textAddress + BigInt(operand + 4) + BigInt(text.readInt32LE(operand));
    const traffic = memory.pointer(globalSlot),
      array = traffic + 0x208n;
    const count = memory.u64(array + 16n),
      capacity = memory.u64(array + 24n);
    if (count > 128n || capacity < count || capacity > 4096n)
      refuse("array-bounds");
    const candidates = [];
    let excludedOwn = 0;
    if (count > 0n) {
      const list = memory.pointer(array + 8n);
      // Validate the whole pointer array before any element-specific reads.
      memory.read(list, Number(count) * 8);
      for (let i = 0; i < Number(count); i++) {
        const vehicle = memory.pointer(list + BigInt(i * 8)),
          base = memory.pointer(vehicle + 0x208n),
          object = memory.pointer(base + 8n);
        if (object === own) {
          excludedOwn++;
          continue;
        }
        candidates.push({
          ordinal: i,
          classification: "unverified-vehicle",
          ...placement(memory.read(vehicle + 0x28n, 32)),
        });
      }
    }
    return {
      ...baseReport(),
      status: "synthetic-candidates",
      excludedOwn,
      candidates,
    };
  } catch (error) {
    return {
      ...baseReport(),
      status: "invalid-fixture",
      reason: error instanceof Rejected ? error.reason : "fixture-format",
    };
  }
}
function main(args) {
  let input;
  try {
    if (args.length === 1 && args[0] === "--demo") input = makeFixture();
    else if (
      args.length === 2 &&
      args[0] === "--fixture" &&
      extname(args[1]).toLowerCase() === ".json"
    ) {
      if (statSync(args[1]).size > 10 * 1024 * 1024) refuse("fixture-size");
      input = JSON.parse(readFileSync(args[1], "utf8").replace(/^\uFEFF/, ""));
    } else {
      console.error(
        JSON.stringify({
          ...baseReport(),
          status: "refused",
          reason: "offline-only",
          usage: "node probe.mjs --demo | --fixture synthetic.json",
        }),
      );
      return 2;
    }
    const report = inspectFixture(input);
    console.log(JSON.stringify(report, null, 2));
    return ["synthetic-candidates", "no-candidates"].includes(report.status)
      ? 0
      : 1;
  } catch (error) {
    console.error(
      JSON.stringify({
        ...baseReport(),
        status: "refused",
        reason: error instanceof Rejected ? error.reason : "fixture-file",
      }),
    );
    return 1;
  }
}
if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
)
  process.exitCode = main(process.argv.slice(2));
