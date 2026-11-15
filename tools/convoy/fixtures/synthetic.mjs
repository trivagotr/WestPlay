// Entirely generated data. These are not game-memory bytes or verified game offsets.
export function makeFixture() {
  const blocks = [];
  const add = (base, length) => {
    const data = Buffer.alloc(length);
    blocks.push({ base, data });
    return data;
  };
  const text = add(0x10001000, 128);
  const pattern =
    "48 8b d9 48 8b 0d 00 00 00 00 48 85 c9 74 00 48 8b 83 00 00 00 00 48 85 c0";
  Buffer.from(pattern.replaceAll(" ", ""), "hex").copy(text, 16);
  text.writeInt32LE(0x10003000 - (0x10001000 + 16 + 10), 22);
  add(0x10003000, 8).writeBigUInt64LE(0x20000000n);
  const traffic = add(0x20000000, 0x240);
  traffic.writeBigUInt64LE(0x20001000n, 0x210);
  traffic.writeBigUInt64LE(3n, 0x218);
  traffic.writeBigUInt64LE(3n, 0x220);
  const list = add(0x20001000, 24);
  for (let i = 0; i < 3; i++) {
    const vehicle = 0x30000000 + i * 0x1000,
      baseObject = 0x40000000 + i * 0x1000;
    list.writeBigUInt64LE(BigInt(vehicle), i * 8);
    const bytes = add(vehicle, 0x220);
    bytes.writeFloatLE(i + 1.5, 0x28);
    bytes.writeFloatLE(2, 0x2c);
    bytes.writeFloatLE(-1.25, 0x30);
    bytes.writeInt16LE(7, 0x34);
    bytes.writeInt16LE(-8, 0x36);
    bytes.writeFloatLE(1, 0x38);
    bytes.writeBigUInt64LE(BigInt(baseObject), 0x208);
    add(baseObject, 16).writeBigUInt64LE(BigInt(0x50000000 + i * 0x1000), 8);
  }
  return {
    schema: "westplay-convoy-synthetic-v1",
    synthetic: true,
    reference: "windows-1.60-unverified",
    frameState: "convoy",
    textAddress: "0x10001000",
    textLength: 128,
    ownObject: "0x50000000",
    segments: blocks.map(({ base, data }) => ({
      address: "0x" + base.toString(16),
      hex: data.toString("hex"),
    })),
  };
}
export function mutateBlock(fixture, address, change) {
  const block = fixture.segments.find((s) => s.address === address);
  if (!block) throw Error("Missing synthetic block");
  const bytes = Buffer.from(block.hex, "hex");
  change(bytes);
  block.hex = bytes.toString("hex");
}
