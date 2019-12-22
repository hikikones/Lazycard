import * as fs from 'fs';
import * as path from 'path';

import cfg from './Config';
import dialog from '../controller/Dialog';
import srs from '../controller/SRS';

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
            return this.demo();
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

    private demo(): IDatabase {
        const now = new Date(Date.now()).toLocaleDateString();
        return {
            topics: [{id: 1, name: "Demo"}],
            cards: [
                {
                    id: 1,
                    front: "Format cards with **Markdown**.",
                    back: "* _italic_\n* ~~strikethrough~~\n* `inline code`",
                    dueDate: now,
                    dueDays: 0,
                    topicId: 1
                },
                {
                    id: 2,
                    front: "Write mathematics with **KaTeX**.",
                    back: "$$ \\int_a^b x^2 \\ \\mathrm dx $$",
                    dueDate: now,
                    dueDays: 0,
                    topicId: 1
                },
                {
                    id: 3,
                    front: "Render code blocks with **syntax highlighting**.",
                    back: "```python\ndef add(a, b):\n    return a + b\n```",
                    dueDate: now,
                    dueDays: 0,
                    topicId: 1
                },
                {
                    id: 4,
                    front: "Add **tables** to your cards.",
                    back: "Tables| Are | Cool\n:--- | :---: | ---:\nleft | center | right\n1 | 2 | 3",
                    dueDate: now,
                    dueDays: 0,
                    topicId: 1
                },
                {
                    id: 5,
                    front: "Display **images** by copy and pasting.",
                    back: "![](data:image/png;base64,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)\n\nBut be careful, they are encoded in `base64` so performance drops considerably if they are too big. Stick with small images.",
                    dueDate: now,
                    dueDays: 0,
                    topicId: 1
                }
            ]
        }
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
    public topicId: number;

    public constructor(topicId: number) {
        super();
        this.topicId = topicId;
    }

    public serialize(): CardData {
        return {
            id: this.id,
            front: this.front,
            back: this.back,
            dueDate: this.dueDate.toLocaleDateString(),
            dueDays: this.dueDays,
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

interface IDatabase {
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