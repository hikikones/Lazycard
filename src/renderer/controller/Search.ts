import { Card } from '../model/Database';
import * as Fuse from 'fuse.js';

class Search {
    private readonly options: {};

    public constructor() {
        this.options = {
            shouldSort: true,
            tokenize: true,
            matchAllTokens: true,
            findAllMatches: true,
            threshold: 0.0,
            location: 0,
            distance: 100,
            maxPatternLength: 32,
            minMatchCharLength: 1,
            keys: ["front", "back"]
        };
    }

    public query(query: string, cards: readonly Card[]): Card[] {
        const fuse = new Fuse(cards, this.options);
        // TODO: ignore chars like _*~ etc due to markdown
        return fuse.search(query.trimRight());
    }
}

export default new Search();