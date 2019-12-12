import { app, BrowserWindow, ipcMain } from "electron";

const isDev = process.env.NODE_ENV === "development";

let mainWindow: Electron.BrowserWindow;

function createWindow() {
	mainWindow = new BrowserWindow({
		width: 1024,
		height: 768,
		webPreferences: {
			nodeIntegration: true
		}
	});

	mainWindow.loadFile("index.html");

	if (isDev) {
		mainWindow.webContents.openDevTools();
	}

	mainWindow.on("close", (e: Event) => {
		if (mainWindow) {
			e.preventDefault();
			mainWindow.hide();
			mainWindow.webContents.send("app-close");
		}
	});
}

app.on("ready", createWindow);

app.on("activate", () => {
	if (mainWindow === null) {
		createWindow();
	}
});

ipcMain.on('quit', (e: Event) => {
	mainWindow = null;
	app.quit();
});