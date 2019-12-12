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

        json.cards.forEach((c: CardData) => {
            const card: Card = new Card(c.topicId);
            card.id = c.id;
            card.front = c.front;
            card.back = c.back;
            card.dueDate = new Date(c.dueDate);
            card.dueDays = c.dueDays;
            this.cards.add(card);
        });

        json.topics.forEach((t: TopicData) => {
            const topic: Topic = new Topic(t.name);
            topic.id = t.id;
            this.topics.add(topic);
        });
    }

    private toJSON(): string {
        return JSON.stringify({
            cards: this.cards.getAll().map(c => c.serialize()),
            topics: this.topics.getAll().map(t => t.serialize())
        }, null, 2);
    }
}

abstract class Table<T extends Entity<EntityData>> {
    public idCounter: number = 1;
    private items: T[] = [];

    protected abstract create(field: string|number): T;

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
}

class Topics extends Table<Topic> {
    protected create(name: string): Topic {
        return new Topic(name);
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
}

export class Topic extends Entity<TopicData> {
    public name: string;

    public constructor(name: string) {
        super();
        this.name = name;
    }

    public serialize(): TopicData {
        return { id: this.id, name: this.name }
    }
}

interface EntityData {
    id: number;
}

interface CardData extends EntityData {
    front: string;
    back: string;
    dueDate: string;
    dueDays: number;
    topicId: number;
}

interface TopicData extends EntityData {
    name: string;
}

export default new Database();