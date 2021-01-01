import * as fs from "fs";
import * as path from "path";
import { app, remote } from "electron";

class Config {
  private readonly userDataPath: string;
  private readonly backupPath: string;
  private dbPath: string;
  private backupAmount: number;
  private theme: string;

  public constructor(injectedUserDataPath: string) {
    this.userDataPath = injectedUserDataPath;
    this.backupPath = path.join(this.userDataPath, "backup");
    this.createDir(this.getUserDataPath());
    this.createDir(this.backupPath);

    this.parse(this.load());
  }

  public resetData(): void {
    fs.unlinkSync(this.userDataPath);
    fs.unlinkSync(this.dbPath);
  }

  public getDatabasePath(): string {
    return this.dbPath;
  }

  public getDatabaseDir(): string {
    return path.dirname(this.dbPath);
  }

  public setDatabasePath(newPath: string): void {
    this.dbPath = newPath;
  }

  public getBackupDir(): string {
    return this.backupPath;
  }

  public getBackupAmount(): number {
    return this.backupAmount;
  }

  public setBackupAmount(amount: number): void {
    this.backupAmount = amount;
  }

  public getTheme(): string {
    return this.theme;
  }

  public setTheme(theme: string): void {
    this.theme = theme;
  }

  public getStaticDir(): string {
    return ".";
  }

  public save(): void {
    const config: IConfig = {
      database: this.getDatabasePath(),
      backupAmount: this.backupAmount,
      theme: this.theme,
    };
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
    this.backupAmount = config.backupAmount;
    this.theme = config.theme;
  }

  private default(): IConfig {
    return {
      database: path.join(this.getUserDataPath(), "database.lazycard"),
      backupAmount: 20,
      theme: "light",
    };
  }

  private getConfigPath(): string {
    return path.join(this.getUserDataPath(), "config.json");
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
  database: string;
  backupAmount: number;
  theme: string;
}

function getUserDataPathByEnv(env: string) {
  const isDev: boolean = process.env.NODE_ENV === "development";
  const dir: string = isDev ? "dev" : "user";

  switch (env) {
    case "development":
      return path.join((app || remote.app).getPath("userData"), dir);
    case "test":
      return __dirname + "/tmp";
    default:
      throw new Error("Unknown Environment");
  }
}
export default new Config(getUserDataPathByEnv(process.env.NODE_ENV));
