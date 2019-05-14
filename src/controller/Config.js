const electron = require("electron");
const path = require("path");
const fs = require("fs");

const isDev = process.env.NODE_ENV === "development";
const userDataPath = isDev
  ? path.join((electron.app || electron.remote.app).getPath("userData"), "dev")
  : (electron.app || electron.remote.app).getPath("userData");

if (!fs.existsSync(userDataPath)) {
  fs.mkdirSync(userDataPath);
}

const defaultConfig = {
  windowBounds: {
    width: 1600,
    height: 1200
  }
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

const config = parseConfigFile();

function writeConfigFile() {
  fs.writeFileSync(getConfigFile(), JSON.stringify(config));
}

class Config {
  get(key) {
    return config[key];
  }

  set(key, val) {
    config[key] = val;
    writeConfigFile();
  }

  isDev() {
    return isDev;
  }
}

export default new Config();
