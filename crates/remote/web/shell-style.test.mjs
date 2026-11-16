import { test } from "node:test";
import assert from "node:assert/strict";
import { applyShellStyle } from "./shell-style.js";
test("browser uses the cabin appearance and bounded wallpaper colors", () => {
  const colors = {};
  const root = {
    dataset: {},
    style: {
      setProperty(k, v) {
        colors[k] = v;
      },
    },
  };
  assert.equal(
    applyShellStyle(root, {
      appearance: "light",
      palette: ["#aabbcc", "#112233", "#ddeeff"],
    }),
    true,
  );
  assert.equal(root.dataset.appearance, "light");
  assert.equal(colors["--cabin-wallpaper-center"], "#112233");
  assert.equal(
    applyShellStyle(root, {
      appearance: "dark",
      palette: ["url(x)", "#112233", "#ddeeff"],
    }),
    false,
  );
  assert.equal(root.dataset.appearance, "light");
  assert.equal(applyShellStyle(root, undefined), false);
  assert.equal(root.dataset.appearance, "light");
});
