import * as fs from 'fs';
import * as path from 'path';
import { app, remote } from "electron";

class Config {
    private userDataPath: string;
    private backupPath: string = null;

    public constructor() {
        const isDev: boolean = process.env.NODE_ENV === "development";
        const dir: string = isDev ? "dev" : "user";
        this.userDataPath = path.join((app || remote.app).getPath("userData"), dir);
        this.createDir(this.getUserDataPath());
    }

    getDatabasePath(): string {
        return path.join(this.getUserDataPath(), "database.lazycard");
    }

    getBackupPath(): string {
        if (this.backupPath !== null) {
            return this.backupPath + ".lazycard";
        }
        return null;
    }

    private getUserDataPath(): string {
        return this.userDataPath;
    }

    private createDir(dir: string) {
        if (!fs.existsSync(dir)) {
            fs.mkdirSync(dir, { recursive: true });
        }
    }

}

export default new Config();