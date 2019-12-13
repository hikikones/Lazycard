import * as fs from 'fs';
import * as path from 'path';
import { app, remote } from "electron";

class Config {
    private readonly userDataPath: string;
    private dbPath: string;

    public constructor() {
        const isDev: boolean = process.env.NODE_ENV === "development";
        const dir: string = isDev ? "dev" : "user";
        this.userDataPath = path.join((app || remote.app).getPath("userData"), dir);
        this.createDir(this.getUserDataPath());

        this.parse(this.load());
    }

    public getDatabasePath(): string {
        return this.dbPath;
    }

    public setDatabasePath(newPath: string): void {
        this.dbPath = newPath;
    }

    public save(): void {
        const config: IConfig = {
            database: this.getDatabasePath()
        }
        fs.writeFileSync(this.getConfigPath(), JSON.stringify(config, null, 2));
    }

    private load(): IConfig {
        const configFile = this.getConfigPath();
        if (!fs.existsSync(configFile)) {
            return this.default();
        }

        const buffer: Buffer = fs.readFileSync(configFile);
        return JSON.parse(buffer.toString());
    }

    private parse(config: IConfig): void {
        this.setDatabasePath(config.database);
    }

    private default(): IConfig {
        return { database: path.join(this.getUserDataPath(), "database.lazycard") }
    }

    private getConfigPath(): string {
        return path.join(this.getUserDataPath(), 'config.json');
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

interface IConfig {
    database: string
}

export default new Config();