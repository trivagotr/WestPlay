// Optional integration test: install Playwright separately, build the synthetic
// example, then run this script. No installed WestPlay process is interrupted.
const fs = require("node:fs");
const path = require("node:path");
const cp = require("node:child_process");
const assert = require("node:assert/strict");
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || "playwright");
const root = path.resolve(__dirname, "../..");
const scratch = path.resolve(
  process.env.MOBILE_SMOKE_DIR || path.join(root, "work/mobile-smoke"),
);
fs.mkdirSync(scratch, { recursive: true });
const statusFile = path.join(scratch, "status.json"),
  stopFile = statusFile + ".stop",
  eventFile = path.join(scratch, "events.log");
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
(async () => {
  for (const file of [statusFile, stopFile])
    if (fs.existsSync(file)) fs.unlinkSync(file);
  const fd = fs.openSync(eventFile, "w");
  const child = cp.spawn(
    path.join(root, "target/debug/examples/synthetic.exe"),
    [statusFile],
    { windowsHide: true, stdio: ["ignore", fd, fd] },
  );
  let browser;
  try {
    for (let i = 0; i < 150 && !fs.existsSync(statusFile); i++)
      await delay(100);
    assert.ok(fs.existsSync(statusFile), "hardware smoke server must start");
    const status = JSON.parse(fs.readFileSync(statusFile, "utf8"));
    browser = await chromium.launch({
      headless: true,
      ...(process.env.CHROME_PATH
        ? { executablePath: process.env.CHROME_PATH }
        : {}),
      args: ["--no-proxy-server", "--autoplay-policy=no-user-gesture-required"],
    });
    const page = await browser.newPage({
      viewport: { width: 390, height: 844 },
      hasTouch: true,
      isMobile: true,
    });
    const errors = [],
      pairs = [];
    if (process.env.MOBILE_SMOKE_SOURCE_CLIENT === "1") {
      await page.route("**/client.js", (route) =>
        route.fulfill({ path: path.join(root, "crates/remote/web/client.js"), contentType: "text/javascript" }),
      );
    }
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("request", (req) => {
      if (req.url().endsWith("/pair")) pairs.push(req.url());
    });
    let failedOffer = false;
    await page.route("**/offer", async (route) => {
      if (failedOffer) await route.continue();
      else {
        failedOffer = true;
        await route.fulfill({
          status: 503,
          body: "Injected first-offer failure",
        });
      }
    });
    await page.goto(status.url, { waitUntil: "networkidle" });
    await page.locator("#code").fill(status.code);
    await page.locator("#connect").click();
    await page.waitForFunction(
      () => document.getElementById("connect").disabled === false,
    );
    await page.locator("#connect").click();
    try {
      await page.waitForFunction(
        () => {
          const v = document.getElementById("video");
          return v.videoWidth === 1280 && v.readyState >= 2;
        },
        {},
        { timeout: 15000 },
      );
    } catch (error) {
      console.error(
        "Video setup diagnostic:",
        await page.locator("#error").textContent(),
        await page.locator("#connection-error").textContent(),
        await page.locator("#status").textContent(),
        "pair requests",
        pairs.length,
        "page errors",
        errors,
      );
      throw error;
    }
    assert.equal(pairs.length, 1, "retry must reuse the paired token");
    const box = await page.locator("#surface").boundingBox();
    const kinds = () =>
      fs.readFileSync(eventFile, "utf8").trim().split(/\r?\n/);
    await page.touchscreen.tap(box.x + box.width / 2, box.y + 10);
    await delay(200);
    assert.ok(
      !kinds().includes("input: down"),
      "letterbox tap must not reach PC",
    );
    await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
    await delay(200);
    assert.ok(
      kinds().includes("input: down") && kinds().includes("input: up"),
      "tap must reach PC as down/up",
    );
    await page.locator("#scroll").click();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 30);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 - 30, {
      steps: 5,
    });
    await page.mouse.up();
    await delay(200);
    assert.ok(
      kinds().includes("input: wheel"),
      "scroll mode swipe must produce wheel input",
    );
    await page.locator("#keyboard").click();
    await page.locator("#text").fill("Smoke test");
    await page.locator("#text-form button[type=submit]").click();
    await delay(200);
    assert.ok(kinds().includes("input: text"), "text must reach PC");
    const forbidden = await fetch(status.url + "pair", {
      method: "POST",
      headers: {
        "content-type": "application/json",
        origin: "http://evil.test",
      },
      body: "{}",
    });
    assert.equal(forbidden.status, 403);
    await page.locator("#disconnect").click();
    await delay(200);
    assert.ok(kinds().includes("input: cancel"));
    assert.ok(await page.locator("#code").isEditable(), "disconnect must permit entering a new PC invitation");
    assert.equal(await page.locator("#code").inputValue(), "");
    assert.deepEqual(errors, []);
    console.log(
      "PASS: real hardware video decode, initial-offer retry, letterboxing, touch, scroll, text, cancel and Origin guard",
    );
  } finally {
    await browser?.close();
    fs.writeFileSync(stopFile, "stop");
    await Promise.race([
      new Promise((resolve) => child.once("exit", resolve)),
      delay(7000),
    ]);
    if (child.exitCode === null) child.kill();
    for (const file of [statusFile, stopFile])
      if (fs.existsSync(file)) fs.unlinkSync(file);
    fs.closeSync(fd);
  }
})().catch((error) => {
  console.error("FAIL:", error.message);
  process.exitCode = 1;
});
