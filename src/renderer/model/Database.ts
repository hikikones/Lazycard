import * as fs from 'fs';

import cfg from './Config';

class Database {
    public readonly cards: Cards = new Cards();
    public readonly topics: Topics = new Topics();

    public constructor() {
        this.load();
    }

    save(path: string = null) {
        const filePath: string = path === null ? cfg.getDatabasePath() : cfg.getBackupPath();
        fs.writeFileSync(filePath, this.toJSON());
    }

    load() {
        if (!fs.existsSync(cfg.getDatabasePath())) {
            return;
        }

        const buffer: Buffer = fs.readFileSync(cfg.getDatabasePath());
        const json: any = JSON.parse(buffer.toString());

        json.cards.forEach((c: Card) => {
            const card: Card = this.cards.new(c.topicId);
            card.id = c.id;
            card.front = c.front;
            card.back = c.back;
            this.cards.add(card);
        });

        json.topics.forEach((t: Topic) => {
            const topic: Topic = this.topics.new(t.name);
            topic.id = t.id;
            this.topics.add(topic);
        });
    }

    private toJSON(): string {
        const json: string = JSON.stringify({
            cards: this.cards.getAll(),
            topics: this.topics.getAll()
        }, null, 2);
        return json;
    }
}

abstract class Table<T extends Entity> {
    public idCounter: number = 1;
    private items: T[] = [];

    abstract new(field: string|number): T;

    get(id: number): T {
        return this.items[this.getIndex(id)];
    }

    getAll(): readonly T[] {
        return this.items;
    }

    add(item: T) {
        if (typeof item.id === "undefined") {
            item.id = this.idCounter;
            this.idCounter++;
        } else if (item.id >= this.idCounter) {
            this.idCounter = item.id + 1;
        }
        
        this.items.push(item);
    }

    delete(id: number) {
        const index = this.getIndex(id);
        this.items.splice(index, 1);
    }

    size(): number {
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
    new(topicId: number): Card {
        return new Card(topicId);
    }

    getByTopic(topicId: number): readonly Card[] {
        return this.getAll().filter((card: Card) => card.topicId === topicId);
    }
}

class Topics extends Table<Topic> {
    new(name: string): Topic {
        return new Topic(name);
    }
}

abstract class Entity {
    public id: number;
}

class Card extends Entity {
    public front: string;
    public back: string;
    public topicId: number;

    public constructor(topicId: number) {
        super();
        this.topicId = topicId;
    }
}

class Topic extends Entity {
    public name: string;

    public constructor(name: string) {
        super();
        this.name = name;
    }
}

export default new Database();