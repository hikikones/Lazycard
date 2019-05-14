const { app, BrowserWindow } = require("electron");

import Config from "./controller/Config";

let window;

function createWindow() {
  const { width, height } = Config.get("windowBounds");
  window = new BrowserWindow({
    width,
    height,
    webPreferences: {
      nodeIntegration: true
    }
  });

  const indexUrl = Config.isDev()
    ? "http://localhost:9000/"
    : `file://${__dirname}/index.html`;
  window.loadURL(indexUrl);

  if (Config.isDev()) {
    window.webContents.openDevTools();
  }

  window.on("closed", () => {
    window = null;
  });
}

app.on("ready", createWindow);

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  if (window === null) {
    createWindow();
  }
});
