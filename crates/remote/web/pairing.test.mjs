import { test } from "node:test";
import assert from "node:assert/strict";
import { consumeInvitation } from "./pairing.js";
test("QR invitation is consumed and removed from the address before pairing", () => {
  const invite = "ab".repeat(24);
  const location = new URL(
    "http://192.168.1.2:8123/?view=tablet#pair=" + invite,
  );
  let clean;
  assert.equal(
    consumeInvitation(location, {
      replaceState(_a, _b, url) {
        clean = url;
      },
    }),
    invite,
  );
  assert.equal(clean, "/?view=tablet");
});
test("manual URL and malformed QR do not start an automatic pairing", () => {
  for (const hash of [
    "",
    "#pair=123456",
    "#pair=" + "a".repeat(49),
    "#pair=%3Cscript%3E",
  ]) {
    assert.equal(
      consumeInvitation(new URL("http://192.168.1.2/" + hash), {
        replaceState() {},
      }),
      null,
    );
  }
});
