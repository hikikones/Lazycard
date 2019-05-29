const { app, BrowserWindow } = require("electron");

import db from "./model/database";

import Config from "./controller/Config";

let window = null;

function createWindow() {
  const { x, y, width, height } = Config.get("windowBounds");
  window = new BrowserWindow({
    x,
    y,
    width,
    height,
    show: Config.isDev(),
    webPreferences: {
      nodeIntegration: true,
      webSecurity: !Config.isDev()
    }
  });

  window.loadURL(`file://${__dirname}/index.html`);

  if (Config.isDev()) {
    window.webContents.openDevTools();
  }

  if (!Config.isDev()) {
    window.setMenu(null);
    window.once('ready-to-show', () => {
      window.show()
    });
  }

  window.on("closed", () => {
    window = null;
  });

  window.on('resize', () => {
    const { width, height, x, y } = window.getBounds();
    Config.set('windowBounds', { width, height, x, y });
  });

  window.on('move', () => {
    const { width, height, x, y } = window.getBounds();
    Config.set('windowBounds', { width, height, x, y });
  });
}

const gotTheLock = app.requestSingleInstanceLock()

if (!gotTheLock) {
  app.quit()
} else {
  app.on('second-instance', (event, commandLine, workingDirectory) => {
    if (window) {
      if (window.isMinimized()) window.restore()
      window.focus()
    }
  })

  app.on("ready", createWindow);

  app.on("window-all-closed", () => {
    if (process.platform !== "darwin") {
      db.backup(app.quit);
    }
  });
  
  app.on("activate", () => {
    if (window === null) {
      createWindow();
    }
  });  
}