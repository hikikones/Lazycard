const electron = require("electron");
const path = require("path");
const fs = require("fs");

const isDev = process.env.NODE_ENV === "development";
const userDataPath = isDev
  ? path.join((electron.app || electron.remote.app).getPath("userData"), "dev")
  : (electron.app || electron.remote.app).getPath("userData");

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
    if (!fs.existsSync(userDataPath)){
      fs.mkdirSync(userDataPath);
    }
    this.data = parseDataFile(this.getConfigFile());
    this.set("isDev", isDev, false);

    console.log("isDev: " + isDev);
    console.log(userDataPath);
  }

  get(key) {
    return this.data[key];
  }

  set(key, val, write = true) {
    this.data[key] = val;
    if (write) {
      fs.writeFileSync(this.getConfigFile(), JSON.stringify(this.data));
    }
  }

  getConfigFile() {
    return path.join(userDataPath, "config.json");
  }

}

export default new Config();
