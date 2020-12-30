import { Application } from "spectron";
import assert from "assert";

const electronPath = require("electron"); // Require Electron from the binaries included in node_modules.
const path = require("path");

const isTestEnv = process.env.NODE_ENV === "test";

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

describe("Application launch", function () {
  let app: Application;

  beforeEach(function () {
    app = isTestEnv ? createCIApplication() : createLocalApplication();
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
      assert.strictEqual(count, 1);
      done();
    });
  });
  it("Finds the first card", function (done) {
    app.client
      .$("#card")
      .then(async (value) => {
        assert(
          (await value.getText()).indexOf(
            "Use keyboard shortcuts for quicker reviewing."
          ) !== -1
        );
        done();
      })
      .catch((e) => {
        done(e);
      });
  });
});
