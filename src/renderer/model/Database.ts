import * as fs from 'fs';
import * as path from 'path';

import cfg from './Config';
import dialog from '../controller/Dialog';
import srs from '../controller/SRS';
import demo from './demo';
import md from '../controller/Markdown';

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

    public exportToHTML(topicId: number): void {
        const path = dialog.saveFile('html', ['html']);
        if (path === undefined) return;

        const topic = this.topics.get(topicId);
        const cards = this.cards.getByTopic(topicId);
        const html: string[] = [];

        html.push('<!DOCTYPE html>');
        html.push('<html lang="en">');
        html.push('<head>');
        html.push('<meta charset="UTF-8">');
        html.push('<meta name="viewport" content="width=device-width, initial-scale=1.0">');
        html.push('<meta http-equiv="X-UA-Compatible" content="ie=edge">');
        html.push(`<title>Lazycard - ${topic.name}</title>`);
        html.push('<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css">');
        html.push('<script defer src="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.js"></script>');
        html.push('<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/themes/prism.min.css">');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/prism.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-python.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-java.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-javascript.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-typescript.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-csharp.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-css.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-c.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-cpp.min.js"></script>');
        html.push('<script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.17.1/components/prism-swift.min.js"></script>');
        html.push('<link rel="stylesheet" href="https://unpkg.com/spectre.css/dist/spectre.min.css">');
        html.push('<script>');
        html.push(`function save() { var data = { "name": "${topic.name}", "cards": [`);
        cards.forEach(c => { html.push(`{ "front": \`${c.front.replace(/\\/g,"\\\\").replace(/`/g,"\`")}\`, "back": \`${c.back.replace(/\\/g,"\\\\").replace(/`/g,"\\`")}\` },`) });
        html.push(`]};`);
        html.push(`var json = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(data, null, 2));`);
        html.push(`var link = document.createElement("a");`);
        html.push(`link.setAttribute("href", json);`);
        html.push(`link.setAttribute("download", "${topic.name}.lazytopic");`);
        html.push(`link.click();`);
        html.push(`}`);
        html.push('</script>');
        html.push('<style>');
        html.push('body {margin: 2rem auto; max-width: 1360px}');
        html.push('h1.topic {text-align: center}');
        html.push('button {margin: 1rem 0}');
        html.push('.cards {display:grid; grid-gap: 1rem; grid-template-columns: repeat(auto-fill, 400px); justify-content: center}');
        html.push('.card {padding: 1rem; box-shadow: 0 0.25rem 1rem rgba(9, 9, 10, 0.15)}');
        html.push('hr {width: 100%; border: none; border-top: 1px solid lightgray; margin: 0; margin-bottom: 1rem}');
        html.push('img {display: block; margin: 0 auto; max-width: 100%; height: auto}');
        html.push('table {width: 100%; border-spacing: 0; border: 1px solid whitesmoke}');
        html.push('table thead {background-color: whitesmoke}');
        html.push('thead th, tbody td {padding: 0.25rem; border: 1px solid whitesmoke}');
        html.push('ul, ol {margin-top: 0}');
        html.push('</style>');
        html.push('</head>');
        html.push('<body>');
        html.push(`<h1 class="topic">${topic.name}</h1>`);
        html.push(`<button class="btn btn-primary p-centered" onClick="save();">Save</button>`);
        html.push('<div class="cards">');
        cards.forEach(c => { html.push(`<div class="card">${md.parse(c.front)}<hr/>${md.parse(c.back)}</div>`) });
        html.push('</div>');
        html.push('</body>');
        html.push('</html>');

        fs.writeFileSync(path, html.join(''));
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