const electron = require("electron");
const path = require("path");
const fs = require("fs");

function parseDataFile(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath));
  } catch (error) {
    return {
      windowBounds: {
        x: 747,
        y: 403,
        width: 1600,
        height: 1200
      }
    };
  }
}

class Config {
  constructor() {
    this.path = (electron.app || electron.remote.app).getPath("userData");
    this.configFile = path.join(this.path, "config.json");

    this.data = parseDataFile(this.configFile);
  }

  get(key) {
    return this.data[key];
  }

  set(key, val) {
    this.data[key] = val;
    fs.writeFileSync(this.configFile, JSON.stringify(this.data));
  }
}

export default new Config();
