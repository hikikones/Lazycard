const katex = require('@iktakahiro/markdown-it-katex');
const prism = require('markdown-it-prism');

class Markdown {
    private readonly md = require('markdown-it')({
        html: true,
        linkify: true,
        typographer: true
    });

    public constructor() {
        this.md.use(katex);
        this.md.use(prism);
    }

    public parse(markdown: string): string {
        return this.md.render(markdown);
    }
}

export default new Markdown();