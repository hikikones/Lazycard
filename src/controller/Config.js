const electron = require("electron");
const path = require("path");
const fs = require("fs");

const isDev = process.env.NODE_ENV === "development";
const userDataPath = isDev
  ? path.join((electron.app || electron.remote.app).getPath("userData"), "dev")
  : path.join(
      (electron.app || electron.remote.app).getPath("userData"),
      "user"
    );

function createFolder(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

createFolder(userDataPath);

const imagesPath = path.join(userDataPath, "images");

createFolder(imagesPath);

const backupsPath = path.join(userDataPath, "backups");

createFolder(backupsPath);

const defaultConfig = {
  windowBounds: {
    width: 800,
    height: 600
  },
  backup: path.join(userDataPath, "backup.zip")
};

function getConfigFile() {
  return path.join(userDataPath, "config.json");
}

function parseConfigFile() {
  try {
    return JSON.parse(fs.readFileSync(getConfigFile()));
  } catch (error) {
    return defaultConfig;
  }
}

function writeConfigFile(data) {
  fs.writeFileSync(getConfigFile(), JSON.stringify(data));
}

class Config {
  // TODO - Main and renderer process both create an instance. This should not happen.
  get(key) {
    const config = parseConfigFile();
    return config[key];
  }

  set(key, val) {
    const config = parseConfigFile();
    config[key] = val;
    writeConfigFile(config);
  }

  getUserDataPath() {
    return userDataPath;
  }

  getImagesPath() {
    return imagesPath;
  }

  getBackupsPath() {
    return backupsPath;
  }

  isDev() {
    return isDev;
  }
}

export default new Config();
