import * as React from 'react';

import search from '../controller/Search';
import { Card } from '../model/Database';

export default class SearchInput extends React.Component<IProps> {
    private input = React.createRef<HTMLInputElement>();

    public constructor(props: IProps) {
        super(props);
    }

    public query = (cards: Card[]): Card[] => {
        return search.query(this.input.current.value, cards);
    }

    public isEmpty = (): boolean => {
        return this.input.current.value === "";
    }

    private clear = (): void => {
        this.input.current.value = "";
        this.props.onInput();
    }

    public render() {
        return (
            <div className="search-container">
                <i className="search-icon material-icons">search</i>
                <input
                    ref={this.input}
                    className="search"
                    placeholder="Search..."
                    type="text"
                    onInput={this.props.onInput}
                />
                <i onClick={this.clear} className="clear-icon material-icons">close</i>
            </div>
        );
    }
}

interface IProps {
    onInput(): void
}