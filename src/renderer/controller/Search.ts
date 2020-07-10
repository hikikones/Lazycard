import { Card } from '../model/Database';
import Fuse from 'fuse.js';

class Search {
    private readonly options = {
        shouldSort: true,
        tokenize: true,
        matchAllTokens: true,
        findAllMatches: true,
        threshold: 0.4,
        location: 0,
        distance: 100,
        maxPatternLength: 32,
        minMatchCharLength: 1,
        keys: ["front", "back"]
    };

    public query(query: string, cards: Card[]): Card[] {
        const fuse = new Fuse(cards, this.options);
        const results = fuse.search(query.trimRight());
        return results.map(c => c.item);
    }
}

export default new Search();