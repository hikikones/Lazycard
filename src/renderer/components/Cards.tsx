import * as React from 'react';

import { Card } from '../model/Database';
import search from '../controller/Search';

import CardSelectable from './CardSelectable';
import Button from './Button';

export default class Cards extends React.Component<IProps, IState> {
    private readonly searchInput = React.createRef<HTMLInputElement>();

    public constructor(props: IProps) {
        super(props);
        this.state = {
            showBack: false,
            showModal: false,
            query: "",
            selected: this.props.cards.filter(c => c.selected).length
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

    private select = (): void => {
        this.setState({ selected: this.state.selected + 1 });
        this.props.onCardChange();
    }

    private deselect = (): void => {
        this.setState({ selected: this.state.selected - 1 });
        this.props.onCardChange();
    }

    private toggleSelectAll = () => {
        if (this.isAllSelected()) {
            this.props.cards.forEach(c => c.selected = false);
            this.setState({ selected: 0 });
        } else {
            this.props.cards.forEach(c => c.selected = true);
            this.setState({ selected: this.props.cards.length });
        }
        this.props.onCardChange();
    }

    private isAllSelected = (): boolean => {
        return this.state.selected === this.props.cards.length;
    }

    private delete = (card: Card): void => {
        if (card.selected) {
            this.setState({ selected: this.state.selected - 1 });
        }
        this.props.onCardChange();
    }

    public render() {
        if (this.props.cards.length === 0) return null;

        let cards = [...this.props.cards];
        if (!this.isSearchEmpty()) {
            cards = search.query(this.state.query, cards)
        } else {
            cards.sort((a, b) => b.id - a.id);
        }

        return (
            <div>
                <h2>Cards</h2>

                <p>Selected: {this.state.selected}</p>

                <section>
                    <Button
                        name="Show answer"
                        icon={this.state.showBack ? "check_box" : "check_box_outline_blank"}
                        action={this.toggleAnswer}
                    />
                    <Button
                        name="Select all"
                        icon={this.isAllSelected() ? "check_box" : "check_box_outline_blank"}
                        action={this.toggleSelectAll}
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
                        <CardSelectable
                            key={c.id}
                            card={c}
                            showBack={this.state.showBack}
                            onDelete={this.delete}
                            onSelect={this.select}
                            onDeselect={this.deselect}
                        />
                    )}
                </section>
            </div>
        );
    }
}

interface IProps {
    cards: readonly Card[]
    onCardChange(): void
}

interface IState {
    showBack: boolean
    showModal: boolean
    query: string
    selected: number
}