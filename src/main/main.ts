import { app, BrowserWindow, ipcMain } from "electron";

const isDev = process.env.NODE_ENV === "development";

let mainWindow: Electron.BrowserWindow;

function createWindow() {
	mainWindow = new BrowserWindow({
		width: 1130,
		height: 768,
		show: isDev,
		webPreferences: {
			nodeIntegration: true,
			webSecurity: !isDev
		}
	});

	mainWindow.loadFile(`${__dirname}/index.html`);

	if (isDev) {
		mainWindow.webContents.openDevTools();
	} else {
		mainWindow.setMenu(null);
		mainWindow.on("ready-to-show", () => {
			mainWindow.show();
		});
	}

	mainWindow.on("close", (e: Event) => {
		if (mainWindow) {
			e.preventDefault();
			mainWindow.hide();
			mainWindow.webContents.send("app-close");
		}
	});
}

const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
	app.quit();
} else {
	app.on("second-instance", () => {
		if (mainWindow) {
			if (mainWindow.isMinimized()) {
				mainWindow.restore();
			}
			mainWindow.focus();
		}
	});

	app.on("ready", createWindow);

	app.on("activate", () => {
		if (mainWindow === null) {
			createWindow();
		}
	});

	ipcMain.on('quit', () => {
		mainWindow = null;
		app.quit();
	});
}