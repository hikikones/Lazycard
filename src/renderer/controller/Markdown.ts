import * as MarkdownIt from 'markdown-it';

const katex = require('@iktakahiro/markdown-it-katex');
const prism = require('markdown-it-prism');
const token = require('markdown-it-modify-token');

class Markdown {
    private readonly md = new MarkdownIt();

    public constructor() {
        this.md.use(katex);
        this.md.use(prism);
        this.md.use(token); // TODO: modify image token for database retrieval
    }

    public parse(markdown: string): string {
        return this.md.render(markdown);
    }
}

export default new Markdown();