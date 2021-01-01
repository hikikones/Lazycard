import { Application } from "spectron";
import assert from "assert";

const electronPath = require("electron"); // Require Electron from the binaries included in node_modules.
const path = require("path");

const isCI = process.env.CI === "true";

function createLocalApplication() {
  return new Application({
    path: electronPath,
    args: [path.join(__dirname, "../build/main.js")],
    startTimeout: 30000,
    waitTimeout: 60000,
  });
}

function createCIApplication() {
  if (!process.env.SELENIUM_HOST || !process.env.SELENIUM_HOST) {
    throw new Error("Selenium host address and port must be defined");
  }
  return new Application({
    host: process.env.SELENIUM_HOST,
    port: parseInt(process.env.SELENIUM_PORT, 10),
    path: electronPath,
    args: [path.join(__dirname, "../build/main.js")],
    startTimeout: 30000,
    waitTimeout: 60000,
  });
}

async function resetData() {
  const config = await import("../src/renderer/model/Config");
  config.default.resetData();
}

describe("Application launch", function () {
  let app: Application;

  beforeEach(function () {
    resetData();
    app = isCI ? createCIApplication() : createLocalApplication();
    return app.start();
  });

  afterEach(function () {
    if (app && app.isRunning()) {
      return app.stop();
    }
  });

  it("shows an initial window", function (done) {
    app.client.getWindowCount().then(function (count: number) {
      // Note that getWindowCount() will return 2 if `dev tools` are opened.
      assert(count > 0);
      done();
    });
  });
  it("Finds the first card on the reviews pane", function (done) {
    app.client
      .$("#card")
      .then(async (value) => {
        const cardText = await value.getText();
        assert(cardText.length > 0);
        done();
      })
      .catch((e) => {
        done(e);
      });
  });
  it("Navigates to the cards pane", async function () {
    try {
      const { client } = app;

      const nav = await (await client.$("#cards-nav")).click();

      const value = await app.client.$("#cards-pane-heading");
      const cardText = await value.getText();
      assert(cardText === "All Cards");
      return;
    } catch (e) {
      return e;
    }
  });
});
