import * as fs from 'fs';

import cfg from './Config';
import srs from '../controller/SRS';

class Database {
    public readonly cards: Cards = new Cards();
    public readonly topics: Topics = new Topics();

    public constructor() {
        this.load();
    }

    public save(path: string = null) {
        const filePath: string = path === null ? cfg.getDatabasePath() : cfg.getBackupPath();
        fs.writeFileSync(filePath, this.toJSON());
    }

    private load() {
        if (!fs.existsSync(cfg.getDatabasePath())) {
            return;
        }

        const buffer: Buffer = fs.readFileSync(cfg.getDatabasePath());
        const json: any = JSON.parse(buffer.toString());

        json.cards.forEach((c: CardJSON) => {
            const card: Card = new Card(c.topicId);
            card.id = c.id;
            card.front = c.front;
            card.back = c.back;
            card.dueDate = new Date(c.dueDate);
            card.dueDays = c.dueDays;
            this.cards.add(card);
        });

        json.topics.forEach((t: TopicJSON) => {
            const topic: Topic = new Topic(t.name);
            topic.id = t.id;
            this.topics.add(topic);
        });
    }

    private toJSON(): string {
        return JSON.stringify({
            cards: this.cards.toJSON(),
            topics: this.topics.toJSON()
        }, null, 2);
    }
}

abstract class Table<T extends Entity> implements ISerialize<EntityJSON> {
    public idCounter: number = 1;
    private items: T[] = [];

    protected abstract create(field: string|number): T;
    abstract toJSON(): EntityJSON[];

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

    public add(item: T) {
        if (item.id === undefined) {
            throw new Error(`Missing id on item ${item} to be added.`);
        }

        if (item.id >= this.idCounter) {
            this.idCounter = item.id + 1;
        }

        this.items.push(item);
    }

    public delete(id: number) {
        const index = this.getIndex(id);
        this.items.splice(index, 1);
    }

    public size(): number {
        return this.items.length;
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
        srs.init(card);
        return card;
    }

    public getByTopic(topicId: number): readonly Card[] {
        return this.getAll().filter((card: Card) => card.topicId === topicId);
    }

    public toJSON(): CardJSON[] {
        return this.getAll().map(c => c.toJSON());
    }
}

class Topics extends Table<Topic> {
    protected create(name: string): Topic {
        return new Topic(name);
    }

    public toJSON(): TopicJSON[] {
        return this.getAll().map(t => t.toJSON());
    }
}

abstract class Entity implements ISerialize<EntityJSON> {
    abstract toJSON(): EntityJSON;
    public id: number;
}

export class Card extends Entity {
    public front: string;
    public back: string;
    public dueDate: Date;
    public dueDays: number;
    public topicId: number;

    public constructor(topicId: number) {
        super();
        this.topicId = topicId;
    }

    public toJSON(): CardJSON {
        return {
            id: this.id,
            front: this.front,
            back: this.back,
            dueDate: this.dueDate.toLocaleDateString(),
            dueDays: this.dueDays,
            topicId: this.topicId
        }
    }
}

export class Topic extends Entity {
    public name: string;

    public constructor(name: string) {
        super();
        this.name = name;
    }

    public toJSON(): TopicJSON {
        return { id: this.id, name: this.name }
    }
}

interface ISerialize<T extends EntityJSON> {
    toJSON(): T|T[]
}

interface EntityJSON {
    id: number;
}

interface CardJSON extends EntityJSON {
    front: string;
    back: string;
    dueDate: string;
    dueDays: number;
    topicId: number;
}

interface TopicJSON extends EntityJSON {
    name: string;
}

export default new Database();