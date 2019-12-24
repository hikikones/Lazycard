import * as fs from 'fs';
import * as path from 'path';

import cfg from './Config';
import dialog from '../controller/Dialog';
import srs from '../controller/SRS';
import demo from './demo';

class Database {
    public readonly cards: Cards = new Cards();
    public readonly topics: Topics = new Topics();

    public constructor() {
        this.parse(this.read());
    }

    public save(path?: string): void {
        const file = path || cfg.getDatabasePath();
        const dbFile = this.toJSON(this.read(file));
        const dbMemory = this.toJSON();

        if (dbFile === dbMemory) return;

        fs.writeFileSync(file, dbMemory);
        this.backup(dbMemory);
    }

    private backup(data: string): void {
        const files: string[] = [];
        fs.readdirSync(cfg.getBackupPath()).forEach(file => {
            files.push(file);
        });
        files.sort().reverse();
        if (files.length >= cfg.getBackupAmount()) {
            try {
                fs.unlinkSync(path.join(cfg.getBackupPath(), files.pop()));
            } catch(err) {}
        }
        fs.writeFileSync(path.join(cfg.getBackupPath(), `${Date.now()}.lazycard`), data);
    }

    public export(topicId: number): void {
        const path = dialog.saveFile('lazytopic', ['lazytopic']);
        if (path === undefined) return;

        const topic = this.topics.get(topicId).export();
        const cards = this.cards.getByTopic(topicId);
        cards.forEach(c => topic.cards.push(c.export()));
        fs.writeFileSync(path, JSON.stringify(topic, null, 2));
    }

    public import(): void {
        const lazytopic = dialog.openFile('lazytopic', ['lazytopic']);
        if (lazytopic === undefined) return;

        const buffer = fs.readFileSync(lazytopic);
        const json: TopicExport = JSON.parse(buffer.toString());

        const topic = this.topics.exists(json.name)
            ? this.topics.getByName(json.name)
            : this.topics.new(json.name);
        json.cards.forEach(c => {
            if (!this.cards.exists(c.front)) {
                const card = this.cards.new(topic.id);
                card.front = c.front;
                card.back = c.back;
                srs.today(card);
            }
        });
    }

    public restore(dbFile: string): void {
        this.cards.reset();
        this.topics.reset();
        this.parse(this.read(dbFile));
    }

    private read(file?: string): IDatabase {
        if (!fs.existsSync(file || cfg.getDatabasePath())) {
            return demo();
        }

        const buffer: Buffer = fs.readFileSync(file || cfg.getDatabasePath());
        const data: IDatabase = JSON.parse(buffer.toString());
        return data;
    }

    private parse(data: IDatabase): void {
        data.cards.forEach(c => {
            const card: Card = new Card(c.topicId);
            card.id = c.id;
            card.front = c.front;
            card.back = c.back;
            card.dueDate = new Date(c.dueDate);
            card.dueDays = c.dueDays;
            card.attempts = c.attempts;
            card.successes = c.successes;
            this.cards.add(card);
        });

        data.topics.forEach(t => {
            const topic: Topic = new Topic(t.name);
            topic.id = t.id;
            this.topics.add(topic);
        });
    }

    private toJSON(data?: IDatabase): string {
        return JSON.stringify({
            cards: data === undefined ? this.cards.getAll().map(c => c.serialize()) : data.cards,
            topics: data === undefined ? this.topics.getAll().map(t => t.serialize()) : data.topics
        }, null, 2);
    }
}

abstract class Table<T extends Entity<EntityData, EntityExport>> {
    public idCounter: number = 1;
    private items: T[] = [];

    protected abstract create(field: string|number): T;
    public abstract exists(field: string): boolean;

    public new(field: string|number): T {
        const item = this.create(field);
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
    protected create(topicId: number): Card {
        const card = new Card(topicId);
        card.attempts = 0;
        card.successes = 0;
        srs.tomorrow(card);
        return card;
    }

    public exists(front: string): boolean {
        for (let c of this.getAll()) {
            if (c.front === front) return true;
        }
        return false;
    }

    public getDue(topicId?: number): Card[] {
        const now = new Date(Date.now());

        if (topicId === undefined) {
            return this.getAll().filter((card: Card) => card.dueDate <= now);
        }

        return this.getAll().filter((card: Card) => card.dueDate <= now && card.topicId === topicId);
    }

    public getByTopic(topicId: number): Card[] {
        return this.getAll().filter((card: Card) => card.topicId === topicId);
    }
}

class Topics extends Table<Topic> {
    protected create(name: string): Topic {
        return new Topic(name);
    }

    public exists(name: string): boolean {
        for (let t of this.getAll()) {
            if (t.name === name) return true;
        }
        return false;
    }

    public getByName(name: string): Topic {
        for (let t of this.getAll()) {
            if (t.name === name) return t;
        }
        return null;
    }
}

abstract class Entity<E extends EntityData, F extends EntityExport> {
    public id: number;
    abstract serialize(): E;
    abstract export(): F;
}

export class Card extends Entity<CardData, CardExport> {
    public front: string;
    public back: string;
    public dueDate: Date;
    public dueDays: number;
    public attempts: number;
    public successes: number;
    public topicId: number;

    public selected: boolean = false;

    public constructor(topicId: number) {
        super();
        this.topicId = topicId;
    }

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
            successes: this.successes,
            topicId: this.topicId
        }
    }

    public export(): CardExport {
        return { front: this.front, back: this.back }
    }
}

export class Topic extends Entity<TopicData, TopicExport> {
    public name: string;

    public constructor(name: string) {
        super();
        this.name = name;
    }

    public serialize(): TopicData {
        return { id: this.id, name: this.name }
    }

    public export(): TopicExport {
        return { name: this.name, cards: [] }
    }
}

export interface IDatabase {
    cards: CardData[]
    topics: TopicData[]
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
    topicId: number
}

interface TopicData extends EntityData {
    name: string
}

interface EntityExport {}

interface TopicExport extends EntityExport {
    name: string
    cards: CardExport[]
}

interface CardExport extends EntityExport {
    front: string
    back: string
}

export default new Database();