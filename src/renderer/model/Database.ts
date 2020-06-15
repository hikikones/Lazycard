import * as fs from 'fs';
import * as path from 'path';

import cfg from './Config';
import srs from '../controller/SRS';

// TODO: Add hash values to cards for easy comparisons

class Database {
    public readonly cards: Cards = new Cards();
    private version: string;

    public constructor() {
        this.parse(this.read());
    }

    public save(path?: string): void {
        const file = path || cfg.getDatabasePath();
        const dbFile = this.toJSON(this.read(file));
        const dbMemory = this.toJSON();

        if (dbFile === dbMemory) return;

        try { fs.writeFileSync(file, dbMemory); }
        catch (err) {}
        this.backup(dbMemory);
    }

    private backup(data: string): void {
        const files: string[] = [];
        try { fs.readdirSync(cfg.getBackupDir()).forEach(file => { files.push(file); }); }
        catch (err) { return; }
        files.sort().reverse();
        if (files.length >= cfg.getBackupAmount()) {
            try { fs.unlinkSync(path.join(cfg.getBackupDir(), files.pop())); }
            catch (err) {}
        }
        fs.writeFileSync(path.join(cfg.getBackupDir(), `${Date.now()}.lazycard`), data);
    }

    public restore(dbFile: string): void {
        this.cards.reset();
        this.parse(this.read(dbFile));
    }

    private read(file?: string): IDatabase {
        if (!fs.existsSync(file || cfg.getDatabasePath())) {
            return this.empty();
        }

        const buffer: Buffer = fs.readFileSync(file || cfg.getDatabasePath());
        const data: IDatabase = JSON.parse(buffer.toString());
        return data;
    }

    private parse(data: IDatabase): void {
        this.version = data.version;
        data.cards.forEach(c => {
            const card: Card = new Card();
            card.id = c.id;
            card.front = c.front;
            card.back = c.back;
            card.dueDate = new Date(c.dueDate);
            card.dueDays = c.dueDays;
            card.attempts = c.attempts;
            card.successes = c.successes;
            this.cards.add(card);
        });
    }

    private toJSON(data?: IDatabase): string {
        return JSON.stringify({
            version: data === undefined ? this.version : data.version,
            cards: data === undefined ? this.cards.getAll().map(c => c.serialize()) : data.cards
        }, null, 2);
    }

    private empty(): IDatabase {
        return {
            version: "2.0.0",
            cards: new Array<CardData>()
        }
    }
}

abstract class Table<T extends Entity<EntityData>> {
    public idCounter: number = 1;
    private items: T[] = [];

    protected abstract create(): T;
    public abstract exists(field: string): boolean;

    public new(): T {
        const item = this.create();
        item.id = this.idCounter;
        this.idCounter++;
        this.items.push(item);
        return item;
    }

    public get(id: number): T {
        return this.items[this.getIndex(id)];
    }

    public getAll(): readonly T[] {
        return this.items;
    }

    public add(item: T): void {
        if (item.id === undefined) {
            throw new Error(`Missing id on item ${item} to be added.`);
        }

        if (item.id >= this.idCounter) {
            this.idCounter = item.id + 1;
        }

        this.items.push(item);
    }

    public delete(id: number): void {
        const index = this.getIndex(id);
        this.items.splice(index, 1);
    }

    public size(): number {
        return this.items.length;
    }

    public reset(): void {
        this.idCounter = 1;
        this.items = [];
    }

    private getIndex(id: number): number {
        for (let i = 0; i < this.size(); i++) {
            if (this.items[i].id === id) return i;
        }
        return -1;
    }
}

class Cards extends Table<Card> {
    protected create(): Card {
        const card = new Card();
        card.attempts = 0;
        card.successes = 0;
        srs.today(card);
        return card;
    }

    public exists(front: string): boolean {
        for (let c of this.getAll()) {
            if (c.front === front) return true;
        }
        return false;
    }

    public getDue(): Card[] {
        const now = new Date(Date.now());
        return this.getAll().filter((card: Card) => card.dueDate <= now);
    }
}

abstract class Entity<E extends EntityData> {
    public id: number;
    abstract serialize(): E;
}

export class Card extends Entity<CardData> {
    public front: string;
    public back: string;
    public dueDate: Date;
    public dueDays: number;
    public attempts: number;
    public successes: number;

    public review(success: boolean): void {
        this.attempts++;
        if (success) this.successes++;
        srs.schedule(this, success);
    }

    public retentionRate(): number {
        if (this.attempts === 0) return 0;
        return this.successes / this.attempts;
    }

    public serialize(): CardData {
        return {
            id: this.id,
            front: this.front,
            back: this.back,
            dueDate: this.dueDate.toLocaleDateString(),
            dueDays: this.dueDays,
            attempts: this.attempts,
            successes: this.successes
        }
    }
}

interface EntityData {
    id: number
}

interface CardData extends EntityData {
    front: string
    back: string
    dueDate: string
    dueDays: number
    attempts: number
    successes: number
}

interface IDatabase {
    version: string
    cards: CardData[]
}

export default new Database();