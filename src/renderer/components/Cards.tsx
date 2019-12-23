import * as React from 'react';

import { Card as CardEntity } from '../model/Database';
import search from '../controller/Search';

import Card from './Card';
import Button from './Button';

export default class Cards extends React.Component<IProps, IState> {
    private readonly searchInput = React.createRef<HTMLInputElement>();

    public constructor(props: IProps) {
        super(props);
        this.state = {
            showBack: false,
            showModal: false,
            query: ""
        }
    }

    private toggleAnswer = (): void => {
        this.setState({ showBack: !this.state.showBack });
    }

    private search = (): void => {
        this.setState({ query: this.searchInput.current.value });
    }

    private clearSearch = (): void => {
        this.searchInput.current.value = "";
        this.setState({ query: "" });
    }

    private isSearchEmpty = (): boolean => {
        return this.state.query === "";
    }

    public render() {
        if (this.props.cards.length === 0) return null;

        let cards = this.props.cards;
        if (!this.isSearchEmpty()) {
            cards = search.query(this.state.query, cards)
        }

        return (
            <div>
                <h2>Cards</h2>

                <section>
                    <Button
                        name="Show answer"
                        icon={this.state.showBack ? "check_box" : "check_box_outline_blank"}
                        action={this.toggleAnswer}
                    />
                    <div className="search-container">
                        <i className="search-icon material-icons">search</i>
                        <input
                            ref={this.searchInput}
                            className="search"
                            placeholder="Search..."
                            type="text"
                            onInput={this.search}
                        />
                        <i onClick={this.clearSearch} className="clear-icon material-icons">close</i>
                    </div>
                </section>

                <section className="cards">
                    {cards.map(c =>
                        <Card
                            key={c.id}
                            card={c}
                            showBack={this.state.showBack}
                            onDelete={this.props.onCardChange}
                        />
                    )}
                </section>
            </div>
        );
    }
}

interface IProps {
    cards: readonly CardEntity[]
    onCardChange(): void
}

interface IState {
    showBack: boolean
    showModal: boolean
    query: string
}